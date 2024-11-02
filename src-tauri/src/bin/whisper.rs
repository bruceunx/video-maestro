use anyhow::Result;
use dotenv::dotenv;
use futures_util::StreamExt;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;

// define the transcription struct with only text in my interest
#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    text: String,
    // segments: Vec<Segment>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
enum Role {
    System,
    User,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    messages: Vec<Message>,
    model: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct DeltaContent {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: DeltaContent,
    // index: i32,
    // finished_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    // id: String,
    choices: Vec<Choice>,
}

// System Prompt: summarize with mindmap?
//
//
// #[derive(Debug, Deserialize)]
// struct Segment {
//     text: String,
//     start: f64,
//     end: f64,
// }
async fn chat_stream(
    api_key: &str,
    user_message: &str,
    llm_model: &str,
    api_url: &str,
) -> Result<()> {
    let client = Client::new();
    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant that provides concise summaries. Please summarize the following content:".to_string(),
            },
            Message {
                role: Role::User,
                content: user_message.to_string(),
            },
        ],
        model: llm_model.to_string(),
        stream: true,
    };
    let response = client
        .post(api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.split("data: ") {
            let line = line.trim();
            if line.is_empty() || line == "[DONE]" {
                continue;
            }

            if let Ok(response) = serde_json::from_str::<ChatResponse>(line) {
                for choice in response.choices {
                    if let Some(content) = choice.delta.content {
                        print!("{}", content);
                        std::io::stdout().flush().unwrap();
                    }
                }
            }
        }
    }

    Ok(())
}

async fn transcribe_audio(
    api_key: &str,
    audio_path: &str,
    audio_model: &str,
    api_url: &str,
) -> Result<TranscriptionResponse> {
    let client = Client::new();

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

    let llm_api_url = std::env::var("GROQ_LLM_URL").expect("AUDIO URL is missing!");
    let llm_model_name = std::env::var("LLM_MODEL").expect("AUDIO MODEL is missing!");

    let audio_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("cache");

    let mut dir = fs::read_dir(audio_path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let audio_path = entry.path();
        let audio_path_str = audio_path.to_str().unwrap();
        let text = match transcribe_audio(&api_key, audio_path_str, &model_name, &api_url).await {
            Ok(response) => response.text,
            Err(e) => panic!("Error {}", e),
        };

        chat_stream(&api_key, &text, &llm_model_name, &llm_api_url).await?;
    }

    Ok(())
}
