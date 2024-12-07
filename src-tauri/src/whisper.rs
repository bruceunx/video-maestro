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
use super::setting;

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

async fn create_client(app: &tauri::AppHandle) -> Result<Client> {
    let client = match setting::get_proxy(app) {
        Some(proxy_url) => Client::builder().proxy(Proxy::https(proxy_url)?).build()?,
        None => Client::builder().build()?,
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
//
//
//
fn get_system_prompt(language: &str) -> String {
    match language {
        "es" => "Proporciona un resumen conciso en español. Traduce literalmente sin interpretar o modificar el significado original. Mantén la estructura y el tono del texto original lo más fielmente posible.".to_string(),
        "fr" => "Fournissez un résumé concis en français. Traduisez littéralement sans interpréter ou modifier le sens original. Conservez la structure et le ton du texte original aussi fidèlement que possible.".to_string(),
        "de" => "Liefern Sie eine prägnante Zusammenfassung auf Deutsch. Übersetzen Sie wörtlich, ohne zu interpretieren oder die ursprüngliche Bedeutung zu verändern. Bewahren Sie Struktur und Ton des Originaltextes so genau wie möglich.".to_string(),
        "zh" => "提供一个简洁的中文摘要。逐字翻译，不要解释或修改原始含义。尽可能忠实地保留原文的结构和语气。".to_string(),
        "zh-TW" => "提供一個簡潔的繁體中文摘要。逐字翻譯，不要解釋或修改原始含義。盡可能忠實地保留原文的結構和語氣。".to_string(),
        "ar" => "قدم ملخصًا موجزًا باللغة العربية. اترجم حرفيًا دون تفسير أو تعديل المعنى الأصلي. حافظ على بنية ونبرة النص الأصلي بأكبر قدر ممكن من الدقة.".to_string(),
        "ru" => "Предоставьте краткое резюме на русском языке. Переводите дословно, не интерпретируя и не изменяя исходного значения. Максимально точно сохраняйте структуру и тон оригинального текста.".to_string(),
        "ja" => "簡潔な日本語の要約を提供してください。元の意味を解釈したり変更したりせず、文字通りに翻訳してください。元のテキストの構造とトーンを可能な限り忠実に保ってください。".to_string(),
        "en" => "Provide a concise summary in English. Translate literally without interpreting or modifying the original meaning. Maintain the structure and tone of the original text as faithfully as possible.".to_string(),
        _ => "Provide a concise summary in language from the content without interpreting or modifying the original meaning. Maintain the structure and tone of the original text as faithfully as possible.".to_string(),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn run_summary(
    app: tauri::AppHandle,
    video_id: i64,
    context: String,
    language: String,
    auto: bool,
) -> Result<(), String> {
    let mut lang = language;
    if auto {
        lang = "auto".to_string()
    }

    let chunks: Vec<String> = context.split("\n\n").map(|s| s.to_string()).collect();
    let mut summary = Vec::new();

    app.emit("summary", "[start]".to_string())
        .map_err(|e| e.to_string())?;
    for chunk in chunks {
        summary.push(chat_stream(&app, &chunk, &lang).await?)
    }
    app.emit("summary", "[end]".to_string())
        .map_err(|e| e.to_string())?;

    let summary_content = summary.join("\n\n");
    db::update_video(
        app.state(),
        video_id,
        "summary".to_string(),
        summary_content,
    )?;
    Ok(())
}

pub async fn chat_stream(
    app: &tauri::AppHandle,
    user_message: &str,
    lang: &str,
) -> Result<String, String> {
    let settings_value = setting::get_settings(app);

    let (api_url, llm_model, api_key) = match settings_value {
        Some(setting::AppSettings {
            ai_url: Some(ai_url),
            ai_model_name: Some(ai_model_name),
            api_key: Some(api_key),
            ..
        }) => (ai_url, ai_model_name, api_key),
        _ => return Err("no api settings found".to_string()),
    };

    let client = create_client(app).await.map_err(|e| e.to_string())?;

    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::System,
                content: get_system_prompt(lang),
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
                        app.emit("summary", content).map_err(|e| e.to_string())?;
                        std::io::stdout().flush().unwrap();
                    }
                }
            }
        }
    }

    Ok(summary.join(""))
}

async fn transcribe_audio(
    app: &tauri::AppHandle,
    api_key: &str,
    audio_path: &str,
    audio_model: &str,
    api_url: &str,
) -> Result<TranscriptionResponse> {
    let client = create_client(app).await?;
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

pub async fn trancript(app: &tauri::AppHandle, audio_path: &Path) -> Result<Vec<String>, String> {
    let settings_value = setting::get_settings(app);

    let (api_url, model_name, api_key) = match settings_value {
        Some(setting::AppSettings {
            whisper_url: Some(whisper_url),
            whisper_model_name: Some(whisper_model_name),
            api_key: Some(api_key),
            ..
        }) => (whisper_url, whisper_model_name, api_key),
        _ => return Err("no api settings found".to_string()),
    };

    let mut dir = fs::read_dir(audio_path).await.map_err(|e| e.to_string())?;
    let mut chunks = Vec::new();

    while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
        let audio_path = entry.path();
        let audio_path_str = audio_path.to_str().unwrap();
        match transcribe_audio(app, &api_key, audio_path_str, &model_name, &api_url).await {
            Ok(response) => {
                app.emit("stream", response.text.clone()).unwrap();
                chunks.push(response.text);
            }
            Err(_) => continue,
        };
    }
    remove_files_from_directory(audio_path)
        .await
        .map_err(|e| e.to_string())?;

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
