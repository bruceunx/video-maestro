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

use crate::gemini::parse_gemini;

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
struct GeminiRequest {
    model: String,
    contents: Vec<GeminiMessage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GeminiMessage {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GeminiPart {
    text: String,
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
    pub start: f64,
    pub end: f64,
    pub text: String,
}

//
//
//
fn get_system_prompt(language: &str) -> String {
    let prompt = match language {
        "zh" => {
            r#"
            你是一名专注于分析视频内容的助手。根据以下提供的视频字幕（格式为 `[时间线 - 文本]`），生成章节。如果有视频简介并且包含时间线，各章节参考时间线并保持一致。每个章节应包括以下内容：
            1. 一个编号比如1， 2 ，3。
            2. 根据该章节中最早和最晚的时间戳确定的开始时间。
            3. 对该章节内容的简洁总结，格式为 `总结：编号 时间线 内容`, 如果可以，根据内容提供相应的emoji。
            指导原则：
            - 根据主题或内容的变化将字幕分组为章节。
            - 使用文本中的逻辑过渡点识别新章节的开始位置。
            - 总结应为3-5句话，概括关键内容。
            - 可以适当使用emoji.
            并对根据各章节内容判断是否需要展开介绍，展开介绍在总结结束后。
            指导原则:
            - 要和总结的编号，时间线保持一致。
            - 展开介绍大概是总结长度的3-5倍。
            展开介绍的输出格式：
            最终输出的总结部分格式如下：
            ## 总结：
            编号. option<emoji> 开始时间 - 内容总结
            编号. option<emoji> 开始时间 - 内容总结
            ...
            ## 展开介绍:
            编号.  option<emoji> 开始时间 - 内容展开
            编号.  option<emoji> 开始时间 - 内容展开
            ...
            如果内容不是中文，请先翻译成中文。
            以下是字幕内容：
            "#
        }
        "zh-TW" => {
            r#"
        你是一名專注於分析影片內容的助手。根據以下提供的影片字幕（格式為 `[時間線 - 文字]`），產生章節。如果有影片簡介並且包含時間線，各章節參考時間線並保持一致。每個章節應包括以下內容：
        1. 一個編號比如1， 2 ，3。
        2. 根據該章節中最早和最晚的時間戳確定的開始時間。
        3. 對該章節內容的簡潔總結，格式為 `總結：編號 時間線 內容`, 如果可以，根據內容提供相應的emoji。
        最終輸出的總結部分格式如下：
        指導原則：
        - 根據主題或內容的變化將字幕分組為章節。
        - 使用文本中的邏輯過渡點識別新章節的開始位置。
        - 總結應為3-5句話，概括關鍵內容。
        - 可以適當使用emoji.
        並對根據各章節內容判斷是否需要展開介紹，展開介紹在總結結束後。
        指導原則:
        - 要和總結的編號，時間線保持一致。
        - 展開介紹大概是總結長度的3-5倍。
        展開介紹的輸出格式：
        ## 總結：
        編號. option<emoji> 開始時間 - 內容總結
        編號. option<emoji> 開始時間 - 內容總結
        ...
        ## 展開介紹:
        編號.  option<emoji> 開始時間 - 內容展開
        編號.  option<emoji> 開始時間 - 內容展開
        ...
        如果內容不是中文，請先翻譯成中文。
        以下是字幕內容：
        "#
        }
        "es" => {
            r#"
        Eres un asistente especializado en analizar contenido de video. Basándote en los subtítulos del video proporcionados a continuación (en el formato `[Marca de tiempo - Texto]`), genera capítulos. Si se proporciona una descripción del video con marcas de tiempo, consúltala y mantén la coherencia. Cada capítulo debe incluir:
        1. Un número, p. ej., 1, 2, 3.
        2. La hora de inicio determinada por las marcas de tiempo más temprana y más tardía en ese capítulo.
        3. Un resumen conciso del contenido del capítulo, con el formato `Resumen: Número Marca de tiempo Contenido`, proporcionando emojis relevantes según el contenido si es posible.
        Principios rectores:
        - Agrupa los subtítulos en capítulos según los cambios en el tema o el contenido.
        - Utiliza puntos de transición lógica en el texto para identificar el inicio de nuevos capítulos.
        - Los resúmenes deben tener de 3 a 5 oraciones, resumiendo el contenido clave.
        - Utiliza emojis cuando sea apropiado.
        Según el contenido de cada capítulo, determina si se necesita una explicación más detallada. Esta explicación detallada debe aparecer después del resumen.
        Principios rectores:
        - Mantén la coherencia con la numeración y las marcas de tiempo de los resúmenes.
        - La explicación detallada debe ser aproximadamente de 3 a 5 veces la longitud del resumen.
        La sección de resumen de la salida final debe tener el siguiente formato:
        ## Resumen:
        Número. option<emoji> Hora de inicio - Resumen del contenido
        Número. option<emoji> Hora de inicio - Resumen del contenido
        ...
        ## Explicación detallada:
        Número. option<emoji> Hora de inicio - Contenido detallado
        Número. option<emoji> Hora de inicio - Contenido detallado
        ...
        Si el contenido no está en español, tradúcelo al español primero.
        Aquí están los subtítulos:
        "#
        }
        "fr" => {
            r#"
        Vous êtes un assistant spécialisé dans l'analyse de contenu vidéo. Sur la base des sous-titres vidéo fournis ci-dessous (au format `[Horodatage - Texte]`), générez des chapitres. Si une description vidéo avec des horodatages est fournie, référez-vous y et maintenez la cohérence. Chaque chapitre doit inclure :
        1. Un numéro, par ex. : 1, 2, 3.
        2. L'heure de début déterminée par les horodatages les plus anciens et les plus récents de ce chapitre.
        3. Un résumé concis du contenu du chapitre, au format `Résumé : Numéro Horodatage Contenu`, en fournissant des emojis pertinents en fonction du contenu si possible.
        Principes directeurs :
        - Regroupez les sous-titres en chapitres en fonction des changements de sujet ou de contenu.
        - Utilisez les points de transition logique dans le texte pour identifier le début de nouveaux chapitres.
        - Les résumés doivent comporter de 3 à 5 phrases, résumant le contenu clé.
        - Utilisez des emojis lorsque cela est approprié.
        En fonction du contenu de chaque chapitre, déterminez si une explication plus détaillée est nécessaire. Cette explication détaillée doit apparaître après le résumé.
        Principes directeurs :
        - Maintenez la cohérence avec la numérotation et les horodatages des résumés.
        - L'explication détaillée doit être environ 3 à 5 fois plus longue que le résumé.
        La section de résumé de la sortie finale doit être formatée comme suit :
        ## Résumé :
        Numéro. option<emoji> Heure de début - Résumé du contenu
        Numéro. option<emoji> Heure de début - Résumé du contenu
        ...
        ## Explication détaillée :
        Numéro. option<emoji> Heure de début - Contenu détaillé
        Numéro. option<emoji> Heure de début - Contenu détaillé
        ...
        Si le contenu n'est pas en français, veuillez d'abord le traduire en français.
        Voici les sous-titres :
        "#
        }
        "de" => {
            r#"
        Du bist ein Assistent, der sich auf die Analyse von Videoinhalten spezialisiert hat. Generiere basierend auf den unten angegebenen Video-Untertiteln (im Format `[Zeitstempel - Text]`) Kapitel. Wenn eine Videobeschreibung mit Zeitstempeln vorhanden ist, beziehe dich darauf und sorge für Konsistenz. Jedes Kapitel sollte Folgendes enthalten:
        1. Eine Nummer, z. B. 1, 2, 3.
        2. Die Startzeit, die durch die frühesten und spätesten Zeitstempel in diesem Kapitel bestimmt wird.
        3. Eine prägnante Zusammenfassung des Inhalts des Kapitels im Format `Zusammenfassung: Nummer Zeitstempel Inhalt`. Füge nach Möglichkeit relevante Emojis basierend auf dem Inhalt hinzu.
        Leitprinzipien:
        - Gruppiere Untertitel basierend auf Änderungen in Thema oder Inhalt in Kapitel.
        - Verwende logische Übergangspunkte im Text, um den Beginn neuer Kapitel zu identifizieren.
        - Zusammenfassungen sollten 3-5 Sätze umfassen und den wichtigsten Inhalt zusammenfassen.
        - Verwende Emojis, wo es angebracht ist.
        Beurteile anhand des Inhalts jedes Kapitels, ob eine detailliertere Erläuterung erforderlich ist. Diese detailliertere Erläuterung sollte nach der Zusammenfassung erscheinen.
        Leitprinzipien:
        - Sorge für Konsistenz mit der Nummerierung und den Zeitstempeln der Zusammenfassungen.
        - Die detailliertere Erläuterung sollte ungefähr 3-5 Mal so lang sein wie die Zusammenfassung.
        Die Zusammenfassung im Endergebnis sollte wie folgt formatiert sein:
        ## Zusammenfassung:
        Nummer. option<emoji> Startzeit - Inhaltszusammenfassung
        Nummer. option<emoji> Startzeit - Inhaltszusammenfassung
        ...
        ## Detaillierte Erläuterung:
        Nummer. option<emoji> Startzeit - Detaillierter Inhalt
        Nummer. option<emoji> Startzeit - Detaillierter Inhalt
        ...
        Wenn der Inhalt nicht auf Deutsch ist, übersetze ihn bitte zuerst ins Deutsche.
        Hier sind die Untertitel:
        "#
        }
        "ja" => {
            r#"
        あなたは、ビデオコンテンツの分析に特化したアシスタントです。以下に示すビデオの字幕（`[タイムスタンプ - テキスト]`形式）に基づいて、章を生成してください。タイムスタンプを含むビデオの説明がある場合は、それを参照して一貫性を保ってください。各章には以下を含める必要があります。
        1. 番号（例：1、2、3）。
        2. その章の最初と最後のタイムスタンプによって決定される開始時間。
        3. 章の内容の簡潔な要約（`要約：番号 タイムスタンプ 内容`形式）。可能であれば、内容に基づいて適切な絵文字を提供します。
        指針：
        - トピックまたは内容の変更に基づいて字幕を章にグループ化します。
        - テキスト内の論理的な移行ポイントを使用して、新しい章の開始位置を特定します。
        - 要約は、主要な内容を要約した3〜5文にする必要があります。
        - 適切な場合は絵文字を使用します。
        各章の内容に基づいて、より詳細な説明が必要かどうかを判断します。この詳細な説明は、要約の後に表示される必要があります。
        指針：
        - 要約の番号とタイムスタンプとの一貫性を保ちます。
        - 詳細な説明は、要約の約3〜5倍の長さにする必要があります。
        詳細な説明の出力形式は次のとおりです。
        最終出力の要約セクションは、次の形式にする必要があります。
        ## 要約：
        番号. option<emoji> 開始時間 - 内容要約
        番号. option<emoji> 開始時間 - 内容要約
        ...
        ## 詳細な説明：
        番号. option<emoji> 開始時間 - 詳細な内容
        番号. option<emoji> 開始時間 - 詳細な内容
        ...
        コンテンツが日本語でない場合は、最初に日本語に翻訳してください。
        以下は字幕です。
        "#
        }
        _ => {
            r#"
        You are an assistant specializing in analyzing video content. Based on the video subtitles provided below (in the format `[Timestamp - Text]`), generate chapters. If a video description with timestamps is provided, refer to it and maintain consistency. Each chapter should include:
        1. A number, e.g., 1, 2, 3.
        2. The start time determined by the earliest and latest timestamps in that chapter.
        3. A concise summary of the chapter's content, formatted as `Summary: Number Timestamp Content`, providing relevant emojis based on the content if possible.
        Guiding principles:
        - Group subtitles into chapters based on changes in topic or content.
        - Use logical transition points in the text to identify the start of new chapters.
        - Summaries should be 3-5 sentences, summarizing the key content.
        - Use emojis where appropriate.
        Based on each chapter's content, determine if a more detailed explanation is needed. 
        Guiding principles:
        - Maintain consistency with the numbering and timestamps of the summaries.
        - The detailed explanation should be approximately 3-5 times the length of the summary.
        The final output's summary sections should be formatted as follows:
        ## Short Summary:
        Number. option<emoji> Start Time - Content Summary
        Number. option<emoji> Start Time - Content Summary
        ...
        ## Detailed Explanation:
        Number. option<emoji> Start Time - Detailed Content
        Number. option<emoji> Start Time - Detailed Content
        ...
        If the content is not in English, please translate it to English first.
        Here are the subtitles:
        "#
        }
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

    let content = utils::transform_segment_to_string(subtitles);

    app.emit("summary", "[start]".to_string())
        .map_err(|e| e.to_string())?;
    let summary_content = chat_stream(&app, &content, &lang, &description).await?;
    app.emit("summary", "[end]".to_string())
        .map_err(|e| e.to_string())?;

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

    let message = format!(
        "short description for the whole content: {description}. full subtitles: {user_message}"
    );

    if api_url.contains("googleapis") {
        handle_gemini_api(app, lang, message, llm_model, client, &api_url, &api_key).await
    } else {
        handle_open_api(app, lang, message, llm_model, client, &api_url, &api_key).await
    }
}

async fn handle_gemini_api(
    app: &tauri::AppHandle,
    lang: &str,
    message: String,
    llm_model: String,
    client: Client,
    api_url: &str,
    _api_key: &str,
) -> Result<String, String> {
    let contents: Vec<GeminiMessage> = vec![
        GeminiMessage {
            role: "model".to_string(),
            parts: vec![GeminiPart {
                text: get_system_prompt(lang),
            }],
        },
        GeminiMessage {
            role: "user".to_string(),
            parts: vec![GeminiPart { text: message }],
        },
    ];

    let request = GeminiRequest {
        model: llm_model,
        contents,
    };

    let response = client
        .post(api_url)
        .header("Content-Type", "application/json")
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
            if let Ok(content) = parse_gemini(line) {
                summary.push(content.clone());
                app.emit("summary", content).map_err(|e| e.to_string())?;
                std::io::stdout().flush().unwrap();
            }
        }
    }

    Ok(summary.join(""))
}

async fn handle_open_api(
    app: &tauri::AppHandle,
    lang: &str,
    message: String,
    llm_model: String,
    client: Client,
    api_url: &str,
    api_key: &str,
) -> Result<String, String> {
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
        model: llm_model,
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
            whisper_api_key: Some(api_key),
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
