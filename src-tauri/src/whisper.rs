use anyhow::Result;
use futures_util::StreamExt;
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Proxy, StatusCode};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use tauri::Manager;
use tauri::{Emitter, State};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;

use super::db::{self, DataBase};
use super::setting;
use super::utils;

// define the transcription struct with only text in my interest
#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    text: String,
    segments: Vec<Segment>,
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

pub async fn create_client(app: &tauri::AppHandle) -> Result<Client> {
    let client = match setting::get_proxy(app) {
        Some(proxy_url) => Client::builder().proxy(Proxy::https(proxy_url)?).build()?,
        None => Client::builder().build()?,
    };
    Ok(client)
}

// System Prompt: summarize with mindmap?
//
//
#[derive(Debug, Deserialize, Serialize)]
pub struct Segment {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

//
//
//
fn get_system_prompt(language: &str) -> String {
    let prompt = match language {
    "es" => "Eres un resumidor multilingüe avanzado. Tu tarea es resumir el contenido proporcionado en español. Si hay una descripción disponible, úsala como contexto para mejorar el resumen. Para contenido corto, proporciona un resumen breve que capture los puntos clave. Para contenido largo, crea un resumen más detallado. Si el contenido no está en español, tradúcelo antes de resumir.",

    "fr" => "Vous êtes un résumé multilingue avancé. Votre tâche est de résumer le contenu fourni en français. Si une description est disponible, utilisez-la comme contexte pour améliorer le résumé. Pour un contenu court, fournissez un bref résumé capturant les points clés. Pour un contenu long, créez un résumé plus détaillé. Si le contenu n'est pas en français, traduisez-le avant de le résumer.",

    "de" => "Sie sind ein fortgeschrittener mehrsprachiger Zusammenfasser. Ihre Aufgabe ist es, den bereitgestellten Inhalt auf Deutsch zusammenzufassen. Wenn eine Beschreibung verfügbar ist, nutzen Sie diese als Kontext für eine bessere Zusammenfassung. Bei kurzem Inhalt erstellen Sie eine knappe Zusammenfassung der Kernpunkte. Bei langem Inhalt erstellen Sie eine ausführlichere Zusammenfassung. Wenn der Inhalt nicht auf Deutsch ist, übersetzen Sie ihn vor der Zusammenfassung.",

    "zh" => "您是一位高级多语言摘要工具。您的任务是用中文总结所提供的内容。如果有描述信息，请将其作为背景来改进摘要。对于简短内容，请提供捕捉要点的简明总结。对于较长内容，请创建更详细的摘要。如果内容不是中文，请先翻译再总结。",

    "zh-TW" => "您是一位高級多語言摘要工具。您的任務是用繁體中文總結所提供的內容。如果有描述資訊，請將其作為背景來改進摘要。對於簡短內容，請提供捕捉要點的簡明總結。對於較長內容，請創建更詳細的摘要。如果內容不是繁體中文，請先翻譯再總結。",

    "ar" => "أنت مُلخص متعدد اللغات متقدم. مهمتك هي تلخيص المحتوى المقدم باللغة العربية. إذا كان هناك وصف متاح، استخدمه كسياق لتحسين الملخص. للمحتوى القصير، قدم ملخصاً موجزاً يلتقط النقاط الرئيسية. للمحتوى الطويل، قم بإنشاء ملخص أكثر تفصيلاً. إذا لم يكن المحتوى باللغة العربية، قم بترجمته قبل تلخيصه.",

    "ru" => "Вы — продвинутый многоязычный резюмер. Ваша задача — создать резюме предоставленного контента на русском языке. Если доступно описание, используйте его как контекст для улучшения резюме. Для короткого контента предоставьте краткое резюме, охватывающее ключевые моменты. Для длинного контента создайте более подробное резюме. Если контент не на русском, переведите его перед резюмированием.",

    "ja" => "あなたは高度な多言語要約者です。提供されたコンテンツを日本語で要約するのがあなたの任務です。説明が利用可能な場合は、それをコンテキストとして要約の改善に使用してください。短いコンテンツの場合は、重要なポイントを捉えた簡潔な要約を提供し、長いコンテンツの場合は、より詳細な要約を作成してください。内容が日本語でない場合、翻訳してから要約してください。",

    _ => "You are an advanced multilingual summarizer. Your task is to summarize the provided content in English. If a description is available, use it as context to improve the summary. For short content, provide a brief summary capturing the key points. For longer content, create a more detailed summary. If the content is not in English, translate it before summarizing.",
};
    prompt.to_string()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn run_summary(
    app: tauri::AppHandle,
    db: State<'_, DataBase>,
    video_id: i64, // id in database
    language: String,
    auto: bool,
) -> Result<(), String> {
    let mut lang = language;
    if auto {
        lang = "auto".to_string()
    }

    let (transcripts, description) = db::get_subtitle_with_id(db, video_id)?;
    let subtitles: Vec<Segment> = serde_json::from_str(&transcripts).map_err(|e| e.to_string())?;

    let chunks = utils::transform_segments_to_chunks(&description, subtitles);

    let mut summary = Vec::new();

    app.emit("summary", "[start]".to_string())
        .map_err(|e| e.to_string())?;
    for chunk in chunks {
        summary.push(chat_stream(&app, &chunk, &lang, &description).await?)
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
    description: &str,
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

    let message = format!("short description for the whole content: {description}. and with content or partial content as following: {user_message}");

    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::System,
                content: get_system_prompt(lang),
            },
            Message {
                role: Role::User,
                content: message,
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
            anyhow::bail!("API error {} - {}", status, error_text);
        }
    }
}

pub async fn trancript(app: &tauri::AppHandle, audio_path: &Path) -> Result<Vec<Segment>, String> {
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

    let mut chunks = Vec::new();

    if !audio_path.is_dir() {
        let audio_path_str = audio_path.to_str().unwrap();
        match transcribe_audio(app, &api_key, audio_path_str, &model_name, &api_url).await {
            Ok(response) => {
                app.emit("stream", response.text.clone()).unwrap();
                chunks.extend(response.segments);
            }
            Err(e) => return Err(e.to_string()),
        };
    } else {
        let mut dir = fs::read_dir(audio_path).await.map_err(|e| e.to_string())?;

        let mut end_time = 0.0;
        while let Some(entry) = dir.next_entry().await.map_err(|e| e.to_string())? {
            let audio_path = entry.path();
            let audio_path_str = audio_path.to_str().unwrap();
            match transcribe_audio(app, &api_key, audio_path_str, &model_name, &api_url).await {
                Ok(response) => {
                    app.emit("stream", response.text.clone()).unwrap();
                    let mut current_end = 0.0;
                    for mut segment in response.segments {
                        segment.start += end_time;
                        current_end = segment.end;
                        chunks.push(segment);
                    }
                    end_time = current_end;
                }
                Err(_) => continue,
            };
        }
        remove_files_from_directory(audio_path)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(chunks)
}
