use anyhow::Result;
use dotenv::dotenv;
use reqwest::multipart::{Form, Part};
use reqwest::StatusCode;
use serde::Deserialize;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    text: String,
    // segments: Vec<Segment>,
}

// #[derive(Debug, Deserialize)]
// struct Segment {
//     text: String,
//     start: f64,
//     end: f64,
// }

async fn transcribe_audio(
    api_key: &str,
    audio_path: &str,
    audio_model: &str,
    api_url: &str,
) -> Result<TranscriptionResponse> {
    let client = reqwest::Client::new();

    let path = Path::new(audio_path);
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;

    let file_part = Part::bytes(buffer)
        .file_name(path.file_name().unwrap().to_string_lossy().to_string())
        .mime_str("audio/wav")?;
    let form = Form::new()
        .text("model", audio_model.to_string())
        .text("response_format", "verbose_json")
        .part("file", file_part);

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    match status {
        StatusCode::OK => {
            let transcription = response.json::<TranscriptionResponse>().await?;
            Ok(transcription)
        }
        _ => {
            let error_text = response.text().await?;
            anyhow::bail!("API error {} - {}", status, error_text);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let api_key = std::env::var("GROQ_API_KEY").expect("API KEY is missing!");
    let api_url = std::env::var("GROQ_AUDIO_URL").expect("AUDIO URL is missing!");
    let model_name = std::env::var("AUDIO_MODEL").expect("AUDIO MODEL is missing!");

    let audio_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("cache")
        .join("temp_000.wav");
    let audio_path_str = audio_path.to_str().unwrap();

    match transcribe_audio(&api_key, audio_path_str, &model_name, &api_url).await {
        Ok(response) => {
            println!("Transcription result: {:?}", response.text);
        }
        Err(e) => eprintln!("Err {}", e),
    }

    Ok(())
}
