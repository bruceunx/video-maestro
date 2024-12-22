use ffmpeg_next::{self as ffmpeg};
use hound::{WavReader, WavWriter};
use std::error::Error;
use std::fs;
use std::path::Path;

pub struct WebmSplitter {
    chunk_duration: i64,
}

pub struct WavSplitter {
    chunk_duration: u32,
}

impl WebmSplitter {
    pub fn new(duration_seconds: i64) -> Self {
        Self {
            chunk_duration: duration_seconds,
        }
    }

    pub fn split(&self, input_file: &Path, output_dir: &Path) -> Result<(), ffmpeg::Error> {
        ffmpeg::init()?;
        if !output_dir.is_dir() {
            std::fs::create_dir_all(output_dir).map_err(|_| ffmpeg::Error::Other { errno: -1 })?;
        }

        let mut input_ctx = ffmpeg::format::input(input_file)?;
        let audio_stream = input_ctx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .ok_or(ffmpeg::Error::StreamNotFound)?;

        let audio_stream_index = audio_stream.index();
        let total_duration = input_ctx.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE);

        let num_chunks = (total_duration / self.chunk_duration as f64) as i64;

        let mut output_files = Vec::new();

        let input_codec_params = audio_stream.parameters();
        let time_base = audio_stream.time_base();

        for chunk_index in 0..num_chunks {
            let encoder = ffmpeg::codec::encoder::find(input_codec_params.id())
                .ok_or(ffmpeg::Error::EncoderNotFound)?;

            let start_time = chunk_index * self.chunk_duration;

            let output_filename = format!("chunk_{:03}.webm", chunk_index + 1);
            let output_path = output_dir.join(output_filename);
            output_files.push(output_path.clone());

            let mut output_ctx = ffmpeg::format::output(&output_path)?;

            let mut output_stream = output_ctx.add_stream(encoder)?;
            output_stream.set_time_base(time_base);

            output_ctx.write_header()?;

            let start_ts = start_time * ffmpeg::ffi::AV_TIME_BASE as i64;

            let end_time = ((start_time + self.chunk_duration) * ffmpeg::ffi::AV_TIME_BASE as i64)
                .min(input_ctx.duration());

            input_ctx.seek(start_ts, ..start_ts)?;

            for (stream, packet) in input_ctx.packets() {
                if stream.index() == audio_stream_index {
                    let pts = packet.pts().unwrap_or(0);
                    let current_time = pts * ffmpeg::ffi::AV_TIME_BASE as i64
                        / stream.time_base().denominator() as i64;

                    if current_time >= end_time {
                        break;
                    }

                    let mut new_packet = packet.clone();
                    new_packet.set_position(-1);
                    new_packet.set_stream(0);
                    new_packet.write_interleaved(&mut output_ctx)?;
                }
            }

            output_ctx.write_trailer()?;
        }
        Ok(())
    }
}

impl WavSplitter {
    pub fn new(duration: u32) -> Self {
        Self {
            chunk_duration: duration,
        }
    }

    pub fn split_wav(&self, input_file: &Path, output_dir: &Path) -> Result<(), Box<dyn Error>> {
        let reader = WavReader::open(input_file)?;
        let spec = reader.spec();
        let samples: Vec<i16> = reader.into_samples::<i16>().collect::<Result<_, _>>()?;

        let sample_rate = spec.sample_rate;
        let channels = spec.channels as usize;
        let chunk_size = (self.chunk_duration * sample_rate * channels as u32) as usize;

        if !output_dir.is_dir() {
            fs::create_dir_all(output_dir)?;
        }

        for (i, chunk) in samples.chunks(chunk_size).enumerate() {
            let output_filename = format!("chunk_{:03}.wav", i + 1);
            let output_file = output_dir.join(output_filename);
            let mut writer = WavWriter::create(&output_file, spec)?;
            for &sample in chunk {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;

    #[test]
    fn split_audio_works() {
        let audio_splitter = WebmSplitter::new(300);

        let input_file = PathBuf::from_str("./sample.webm").unwrap();
        let output_dir = PathBuf::from_str("output_dir").unwrap();
        let result = audio_splitter.split(&input_file, &output_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn split_wav_works() {
        let wav_splitter = WavSplitter::new(300);
        let input_file = PathBuf::from_str("./output.wav").unwrap();
        let output_dir = PathBuf::from_str("output_dir").unwrap();
        let result = wav_splitter.split_wav(&input_file, &output_dir);
        assert!(result.is_ok());
    }
}
