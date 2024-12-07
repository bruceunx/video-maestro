pub mod webvtt;
use dotenv::dotenv;
use tauri::{Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use tokio::fs;
mod db;
mod setting;
mod whisper;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct VideoInfo {
    title: String,
    duration: u64,
    upload_date: String,
}

async fn run_ffmpeg(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    println!("run ffmpeg");
    let cache_dir = app.path().cache_dir().unwrap();
    let temp_path = cache_dir.join("newscenter").join("temp.wav");
    if !temp_path.is_file() {
        return Err("Error: temp wav not exit".to_string());
    }
    let temp_path_str = temp_path.to_str().unwrap();

    let split_fold = cache_dir.join("newscenter").join("temp");
    if !split_fold.is_dir() {
        fs::create_dir_all(&split_fold)
            .await
            .expect("cannot create a temp fold for split wav");
    }

    whisper::remove_files_from_directory(&split_fold)
        .await
        .expect("remove temp fold failed");

    let split_path = format!("{}/temp_%02d.wav", split_fold.to_str().unwrap());

    let output = app
        .shell()
        .sidecar("ffmpeg")
        .expect("ffmpeg command found")
        .args([
            "-y",
            "-i",
            temp_path_str,
            "-f",
            "segment",
            "-segment_time",
            "00:10:00",
            "-reset_timestamps",
            "1",
            "-c",
            "copy",
            &split_path,
        ])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        fs::remove_file(temp_path)
            .await
            .expect("cannot remove the temp file");

        Ok(split_fold)

        // match whisper::trancript_summary(app, &split_fold).await {
        //     Ok(_) => "success: summary finished".to_string(),
        //     Err(_) => "error: summary failed".to_string(),
        // }
    } else {
        Err("error: ffmpeg error".to_string())
    }
}

async fn get_video_metadata(app: &tauri::AppHandle, url: &str) -> Result<VideoInfo, String> {
    let mut args = Vec::new();
    if let Some(proxy_url) = setting::get_proxy(app) {
        args.push("--proxy".to_string());
        args.push(proxy_url);
    }

    args.push("--dump-json".to_string());
    args.push(url.to_string());

    let output = app
        .shell()
        .sidecar("ytdown")
        .expect("should find the ytdown!")
        .args(args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let output_str = std::str::from_utf8(&output.stdout).unwrap();
        serde_json::from_str::<VideoInfo>(&output_str).map_err(|e| e.to_string())
    } else {
        Err("can not find video info from metadata".to_string())
    }
}

async fn handle_transcripts(
    app: &tauri::AppHandle,
    video_id: i64,
    chunks: Vec<String>,
) -> Result<(), String> {
    let transcripts = chunks.join("\n\n");
    db::update_video(
        app.state(),
        video_id,
        "transcripts".to_string(),
        transcripts,
    )?;
    app.emit("stream", "[start]").map_err(|e| e.to_string())?;
    for chunk in chunks {
        app.emit("stream", chunk).map_err(|e| e.to_string())?;
    }
    app.emit("stream", "[end]").map_err(|e| e.to_string())?;
    Ok(())
}

async fn download_with_retries(
    app: &tauri::AppHandle,
    url: &str,
    max_retries: usize,
) -> Result<(), String> {
    let cache_dir = app.path().cache_dir().unwrap();
    let temp_path = cache_dir.join("newscenter").join("temp.wav");
    let temp_path_str = temp_path.to_str().unwrap();
    let mut args = Vec::new();

    if let Some(proxy_url) = setting::get_proxy(app) {
        args.push("--proxy".to_string());
        args.push(proxy_url);
    }

    let standard_args = vec![
        "--force-overwrites",
        "-x",
        "-f",
        "worstaudio[ext=webm]",
        "--extract-audio",
        "--audio-format",
        "wav",
        "--postprocessor-args",
        "-ar 16000 -ac 1",
        "-o",
    ];

    args.extend(standard_args.into_iter().map(String::from));
    args.push(temp_path_str.to_string());
    args.push(url.to_string());

    for attempt in 0..max_retries {
        match app
            .shell()
            .sidecar("ytdown")
            .expect("should find the ytdown!")
            .args(args.clone())
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    return Ok(());
                } else {
                    if attempt < max_retries - 1 {
                        tokio::time::sleep(std::time::Duration::from_secs(
                            2 * (attempt + 1) as u64,
                        ))
                        .await;
                    }
                }
            }
            Err(_) => {
                if attempt < max_retries - 1 {
                    tokio::time::sleep(std::time::Duration::from_secs(2 * (attempt + 1) as u64))
                        .await;
                }
            }
        }
    }

    // If all retries fail
    Err("Failed to download and process after maximum retries".to_string())
}

#[tauri::command(rename_all = "snake_case")]
async fn run_yt(app: tauri::AppHandle, url: &str, input_id: i64) -> Result<(), String> {
    let mut video_id = input_id;
    if video_id == -1 {
        let video_info = get_video_metadata(&app, url).await?;
        video_id = db::create_video(
            app.state(),
            url.to_string(),
            video_info.title,
            video_info.duration,
            video_info.upload_date,
        )?;
        app.emit("state", "update video")
            .map_err(|e| e.to_string())?
    }
    if let Some(lang) = webvtt::get_sub_lang(&app, url).await {
        let vtt_path = webvtt::run_yt_vtt(&app, url, &lang).await?;
        let chunks = webvtt::extract_vtt_chunks(&vtt_path).await?;
        handle_transcripts(&app, video_id, chunks).await?;
    } else {
        // most vulnerable part with yt-dlp
        download_with_retries(&app, url, 5).await?;

        let split_path = run_ffmpeg(&app).await?;
        app.emit("stream", "[start]".to_string())
            .map_err(|e| e.to_string())?;
        let chunks = whisper::trancript(&app, &split_path).await?;
        app.emit("stream", "[end]".to_string())
            .map_err(|e| e.to_string())?;
        let transcripts = chunks.join("\n\n");
        db::update_video(
            app.state(),
            video_id,
            "transcripts".to_string(),
            transcripts,
        )?;
    }

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
