use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, CONTENT_TYPE, USER_AGENT},
    Client, Proxy,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::Write, path::PathBuf};

pub struct YoutubeAudio {
    client: Client,
}

#[derive(Serialize)]
struct ClientInfo {
    client_name: String,
    client_version: String,
    device_make: String,
    device_model: String,
    os_name: String,
    os_version: String,
    android_sdk_version: String,
}

#[derive(Serialize)]
struct RequestContext {
    client: ClientInfo,
}

#[derive(Serialize)]
struct RequestBody {
    context: RequestContext,
    video_id: String,
    #[serde(rename = "contentCheckOk")]
    content_check_ok: String,
}

impl ClientInfo {
    pub fn default() -> Self {
        Self {
            client_name: "ANDROID_VR".to_string(),
            client_version: "1.60.19".to_string(),
            device_make: "Oculus".to_string(),
            device_model: "Quest 3".to_string(),
            os_name: "Android".to_string(),
            os_version: "12L".to_string(),
            android_sdk_version: "32".to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseBody {
    streaming_data: StreamingData,
    captions: Option<Captions>,
    video_details: VideoDetail,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoDetail {
    // video_id: String,
    title: String,
    length_seconds: String,
    keywords: Option<Vec<String>>,
    short_description: Option<String>,
    thumbnail: ThumbNail,
}

#[derive(Deserialize)]
struct ThumbNail {
    thumbnails: Vec<ThumbNailItem>,
}

#[derive(Deserialize)]
struct ThumbNailItem {
    url: String,
    // width: u32,
    // height: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Captions {
    player_captions_tracklist_renderer: PlayerCaptionsTracklistRenderer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerCaptionsTracklistRenderer {
    caption_tracks: Vec<CaptionItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CaptionItem {
    base_url: String,
    vss_id: String,
    // language_code: String,
}

#[derive(Deserialize)]
struct Format {
    #[serde(rename = "mimeType")]
    mime_type: String,
    bitrate: u32,
    url: String,
    #[serde(rename = "contentLength")]
    content_length: String,
}

#[derive(Deserialize)]
struct StreamingData {
    formats: Option<Vec<Format>>,
    #[serde(rename = "adaptiveFormats")]
    adaptive_formats: Option<Vec<Format>>,
}

// export the struct
pub struct AudioData {
    pub title: String,
    pub length: u64,
    pub keywords: Option<Vec<String>>,
    pub description: Option<String>,
    pub caption_lang: Option<String>,
    pub caption_url: Option<String>,
    pub audio_url: String,
    pub audio_length: u64,
    pub thumbnail_url: String,
}

fn extract_id(url: &str) -> Option<String> {
    let re =
        Regex::new(r"(?:v=|\/v\/|youtu\.be\/|\/embed\/|\/shorts\/)([A-Za-z0-9_-]{11})").unwrap();

    if let Some(captures) = re.captures(url) {
        return captures.get(1).map(|m| m.as_str().to_string());
    }

    None
}

// pub fn extract_xml_captions(content: &str) -> Option<Vec<String>> {
//     todo!()
// }

impl YoutubeAudio {
    pub fn new(proxy: Option<&str>) -> Self {
        let client_builder = Client::builder();
        let client = match proxy {
            Some(proxy_str) => match Proxy::https(proxy_str) {
                Ok(ok_proxy) => client_builder.proxy(ok_proxy).build().unwrap(),
                Err(_) => client_builder.build().unwrap(),
            },
            None => client_builder.build().unwrap(),
        };
        Self { client }
    }

    pub async fn get_video_info(&self, url: &str) -> Option<AudioData> {
        let video_id = match extract_id(url) {
            Some(_id) => _id,
            None => return None,
        };
        let mut headers = HeaderMap::new();

        headers.insert(USER_AGENT, HeaderValue::from_static("com.google.android.apps.youtube.vr.oculus/1.60.19 (Linux; U; Android 12L; eureka-user Build/SQ3A.220605.009.A1) gzip"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let request_body = RequestBody {
            context: RequestContext {
                client: ClientInfo::default(),
            },
            video_id: video_id.to_string(),
            content_check_ok: "true".to_string(),
        };

        let response = self
            .client
            .post("https://www.youtube.com/youtubei/v1/player?prettyPrint=false")
            .headers(headers)
            .json(&request_body)
            .send()
            .await
            .ok()?;

        let response_data: ResponseBody = match response.json().await {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };

        let mut all_formats = Vec::new();
        if let Some(formats) = response_data.streaming_data.formats {
            all_formats.extend(formats);
        }
        if let Some(adaptive_formats) = response_data.streaming_data.adaptive_formats {
            all_formats.extend(adaptive_formats);
        }

        let (audio_url, audio_length) = match all_formats
            .into_iter()
            .filter(|format| format.mime_type.starts_with("audio"))
            .min_by_key(|format| format.bitrate)
        {
            Some(format) => (
                format.url,
                format
                    .content_length
                    .parse::<u64>()
                    .ok()
                    .or(Some(0))
                    .unwrap(),
            ),
            None => return None,
        };

        let thumbnail_url = response_data.video_details.thumbnail.thumbnails[0]
            .url
            .clone();

        let (caption_url, caption_lang) = match response_data.captions {
            Some(captions) => {
                let caption = &captions.player_captions_tracklist_renderer.caption_tracks[0];

                (Some(caption.base_url.clone()), Some(caption.vss_id.clone()))
            }
            None => (None, None),
        };

        Some(AudioData {
            title: response_data.video_details.title,
            length: response_data
                .video_details
                .length_seconds
                .parse::<u64>()
                .unwrap(),
            keywords: response_data.video_details.keywords,
            description: response_data.video_details.short_description,
            caption_lang,
            caption_url,
            audio_url,
            audio_length,
            thumbnail_url,
        })
    }

    pub async fn download_caption(&self, caption_url: &str) -> Option<Vec<String>> {
        todo!()
    }

    pub async fn download_audio(
        &self,
        audio_url: &str,
        file_size: u64,
        file_path: PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-us,en"));
        let mut file = File::create(file_path)?;
        let mut downloaded = 0;
        let default_range_size = 1024 * 1024 * 9;
        while downloaded < file_size {
            let stop_pos = (downloaded + default_range_size).min(file_size) - 1;
            let chunk_reponse = self
                .client
                .get(format!("{}?range={}-{}", audio_url, downloaded, stop_pos))
                .headers(headers.clone())
                .send()
                .await?;

            let chunk = chunk_reponse.bytes().await?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::{env, str::FromStr};

    #[tokio::test]
    async fn check_response_body_works() {
        dotenv().ok();
        let proxy = env::var("PROXY").ok();
        let youtube_client = YoutubeAudio::new(proxy.as_deref());
        let url = "https://www.youtube.com/watch?v=Q0cvzaPJJas&ab_channel=TJDeVries";
        let video_data = youtube_client.get_video_info(url).await;
        assert!(video_data.is_some());
        let video = video_data.unwrap();
        assert_eq!(video.caption_lang.unwrap(), "a.en".to_string());
    }

    #[tokio::test]
    async fn check_download_audio_works() {
        dotenv().ok();
        let proxy = env::var("PROXY").ok();
        let youtube_client = YoutubeAudio::new(proxy.as_deref());
        let url = "https://www.youtube.com/watch?v=Q0cvzaPJJas&ab_channel=TJDeVries";
        let video_data = youtube_client.get_video_info(url).await;
        assert!(video_data.is_some());

        let video = video_data.unwrap();

        let audio_url = &video.audio_url;
        let audio_length = video.audio_length;
        let file_path = PathBuf::from_str("./sample.webm").unwrap();
        let download = youtube_client
            .download_audio(audio_url, audio_length, file_path)
            .await;

        assert!(download.is_ok());
    }

    #[test]
    fn extract_id_works() {
        let test_cases = vec![
            (
                "https://youtu.be/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Short YouTube URL
            (
                "https://www.youtube.com/watch?v=FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Standard YouTube URL
            (
                "https://youtube.com/watch?v=FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // YouTube without 'www'
            (
                "https://www.youtube.com/v/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // /v/ style URL
            (
                "https://www.youtube.com/embed/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Embedded URL
            (
                "https://www.youtube.com/shorts/FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Shorts URL
            (
                "https://youtu.be/FdeioVndUhs?t=30",
                Some("FdeioVndUhs".to_string()),
            ), // URL with timestamp
            (
                "https://www.youtube.com/watch?v=FdeioVndUhs&feature=share",
                Some("FdeioVndUhs".to_string()),
            ), // With additional params
            (
                "https://www.youtube.com/watch?v=FdeioVndUhs&list=PL123",
                Some("FdeioVndUhs".to_string()),
            ), // With playlist
            (
                "https://www.youtube.com/watch?list=PL123&v=FdeioVndUhs",
                Some("FdeioVndUhs".to_string()),
            ), // Playlist first
            ("https://invalid.url/FdeioVndUhs", None), // Invalid URL
            ("https://www.youtube.com/", None),        // Homepage
            ("https://youtu.be/", None),               // Invalid Short URL
        ];

        for (input, expected) in test_cases {
            let video_id = extract_id(input);
            assert_eq!(video_id, expected);
        }
    }
}
