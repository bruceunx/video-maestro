pub mod webvtt;
use dotenv::dotenv;
use tauri::{Emitter, Manager};
mod db;
mod setting;
mod whisper;
use tube_rs::{SubtitleEntry, YoutubeAudio};

fn transform_subtitles_to_chunks(subtitles: Vec<SubtitleEntry>) -> Result<Vec<String>, String> {
    let mut chunks = Vec::new();
    let mut current_text = String::new();
    for subtitle in subtitles {
        if current_text.len() + subtitle.text.len() < 2000 {
            current_text.push_str(&subtitle.text)
        } else {
            chunks.push(current_text.trim().to_string());
            current_text.clear();
        }
    }
    Ok(chunks)
}

#[tauri::command(rename_all = "snake_case")]
async fn run_yt(app: tauri::AppHandle, url: &str, input_id: i64) -> Result<(), String> {
    let mut _id = input_id;
    let youtube_audio = YoutubeAudio::new(setting::get_proxy(&app).as_deref());
    if _id == -1 {
        let audio_data = match youtube_audio.get_video_info(url).await {
            Some(data) => data,
            None => return Err("failed to parse audio info".to_string()),
        };
        _id = db::create_video(app.state(), audio_data)?;
        app.emit("state", "update video")
            .map_err(|e| e.to_string())?
    }
    match db::get_caption_with_id(app.state(), _id) {
        Ok((Some(lang), Some(url))) => {
            let subtitles = youtube_audio
                .download_caption(&url, &lang)
                .await
                .map_err(|e| e.to_string())?;
            app.emit("stream", "[start]".to_string())
                .map_err(|e| e.to_string())?;
            for subtitle in &subtitles {
                app.emit("stream", subtitle.text.clone())
                    .map_err(|e| e.to_string())?
            }
            app.emit("stream", "[end]".to_string())
                .map_err(|e| e.to_string())?;

            let chunks = transform_subtitles_to_chunks(subtitles)?;
            let transcripts = chunks.join("\n\n");
            db::update_video(app.state(), _id, "transcripts".to_string(), transcripts)?;
            return Ok(());
        }
        _ => {}
    };

    let (audio_url, audio_filesize) = db::get_audio_url_with_id(app.state(), _id)?;
    let cache_dir = app.path().cache_dir().unwrap();
    let temp_path = cache_dir.join("newscenter").join("temp.webm");
    youtube_audio
        .download_audio(&audio_url, audio_filesize, &temp_path)
        .await
        .map_err(|e| e.to_string())?;

    if audio_filesize > 25 * 1024 * 1024 {
        // split the audio
        todo!()
    }

    app.emit("stream", "[start]".to_string())
        .map_err(|e| e.to_string())?;
    let chunks = whisper::trancript(&app, &temp_path).await?;
    app.emit("stream", "[end]".to_string())
        .map_err(|e| e.to_string())?;
    let transcripts = chunks.join("\n\n");
    db::update_video(app.state(), _id, "transcripts".to_string(), transcripts)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenv().ok();
    tauri::Builder::default()
        .setup(|app| {
            setting::get_config_path(&app.handle());
            let database = db::init_db(app.handle())?;
            app.manage(database);
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            run_yt,
            whisper::run_summary,
            db::get_videos,
            db::delete_video,
            setting::load_settings,
            setting::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
