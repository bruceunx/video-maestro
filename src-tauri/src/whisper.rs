use anyhow::Result;
use futures_util::StreamExt;
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Proxy, StatusCode};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use tauri::Emitter;
use tauri::Manager;
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;

use super::db;

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

// after process the file remove the file
pub async fn remove_files_from_directory(dir_path: &Path) -> Result<()> {
    let mut dir = fs::read_dir(dir_path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let _path = entry.path();
        if _path.is_file() {
            fs::remove_file(_path).await?;
        }
    }
    Ok(())
}

async fn create_client() -> Result<Client> {
    let client = match std::env::var("PROXY_URL") {
        Ok(proxy_url) => Client::builder().proxy(Proxy::https(proxy_url)?).build()?,
        Err(_) => Client::builder().build()?,
    };
    Ok(client)
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
#[tauri::command(rename_all = "snake_case")]
pub async fn run_summary(
    app: tauri::AppHandle,
    video_id: i64,
    context: String,
) -> Result<(), String> {
    let chunks: Vec<String> = context.split("\n\n").map(|s| s.to_string()).collect();
    let mut summary = Vec::new();
    for chunk in chunks {
        summary.push(chat_stream(&app, &chunk).await?)
    }
    let summary_content = summary.join("\n\n");
    db::update_video(
        app.state(),
        video_id,
        "summary".to_string(),
        summary_content,
    )?;
    Ok(())
}

pub async fn chat_stream(app: &tauri::AppHandle, user_message: &str) -> Result<String, String> {
    let api_url = std::env::var("GROQ_LLM_URL").expect("AUDIO URL is missing!");
    let llm_model = std::env::var("LLM_MODEL").expect("AUDIO MODEL is missing!");
    let api_key = std::env::var("GROQ_API_KEY").expect("API KEY is missing!");

    let client = create_client().await.map_err(|e| e.to_string())?;

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
        .await
        .map_err(|e| e.to_string())?;

    let mut summary = Vec::new();

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.split("data: ") {
            let line = line.trim();
            if line.is_empty() || line == "[DONE]" {
                continue;
            }

            if let Ok(response) = serde_json::from_str::<ChatResponse>(line) {
                for choice in response.choices {
                    if let Some(content) = choice.delta.content {
                        summary.push(content.clone());
                        app.emit("stream", content).map_err(|e| e.to_string())?;
                        std::io::stdout().flush().unwrap();
                    }
                }
            }
        }
    }

    Ok(summary.join(""))
}

async fn transcribe_audio(
    api_key: &str,
    audio_path: &str,
    audio_model: &str,
    api_url: &str,
) -> Result<TranscriptionResponse> {
    let client = create_client().await?;
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
            println!("error from whisper {}", error_text);
            anyhow::bail!("API error {} - {}", status, error_text);
        }
    }
}

pub async fn trancript(audio_path: &Path) -> Result<Vec<String>> {
    let api_key = std::env::var("GROQ_API_KEY").expect("API KEY is missing!");
    let api_url = std::env::var("GROQ_AUDIO_URL").expect("AUDIO URL is missing!");
    let model_name = std::env::var("AUDIO_MODEL").expect("AUDIO MODEL is missing!");

    // let llm_api_url = std::env::var("GROQ_LLM_URL").expect("AUDIO URL is missing!");
    // let llm_model_name = std::env::var("LLM_MODEL").expect("AUDIO MODEL is missing!");

    let mut dir = fs::read_dir(audio_path).await?;
    let mut chunks = Vec::new();

    // app.emit("stream", "[start]")?;
    while let Some(entry) = dir.next_entry().await? {
        let audio_path = entry.path();
        let audio_path_str = audio_path.to_str().unwrap();
        match transcribe_audio(&api_key, audio_path_str, &model_name, &api_url).await {
            Ok(response) => chunks.push(response.text),
            Err(_) => continue,
        };

        // app.emit("stream", &text)?;
        // chat_stream(&app, &api_key, &text, &llm_model_name, &llm_api_url).await?;
    }
    // app.emit("stream", "[end]")?;
    remove_files_from_directory(audio_path).await?;

    Ok(chunks)
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     dotenv().ok();
//     let api_key = std::env::var("GROQ_API_KEY").expect("API KEY is missing!");
//     let api_url = std::env::var("GROQ_AUDIO_URL").expect("AUDIO URL is missing!");
//     let model_name = std::env::var("AUDIO_MODEL").expect("AUDIO MODEL is missing!");
//
//     let llm_api_url = std::env::var("GROQ_LLM_URL").expect("AUDIO URL is missing!");
//     let llm_model_name = std::env::var("LLM_MODEL").expect("AUDIO MODEL is missing!");
//
//     let audio_path = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .parent()
//         .unwrap()
//         .join("cache");
//
//     let mut dir = fs::read_dir(audio_path).await?;
//     while let Some(entry) = dir.next_entry().await? {
//         let audio_path = entry.path();
//         let audio_path_str = audio_path.to_str().unwrap();
//         let text = match transcribe_audio(&api_key, audio_path_str, &model_name, &api_url).await {
//             Ok(response) => response.text,
//             Err(e) => anyhow::bail!("Error from transcribe_audio {}", e),
//         };
//
//         chat_stream(&api_key, &text, &llm_model_name, &llm_api_url).await?;
//     }
//
//     Ok(())
// }
