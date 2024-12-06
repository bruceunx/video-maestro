pub mod webvtt;
use dotenv::dotenv;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tokio::fs;
mod db;
mod setting;
mod whisper;

async fn run_ffmpeg(app: tauri::AppHandle) -> String {
    println!("run ffmpeg");
    let cache_dir = app.path().cache_dir().unwrap();
    let temp_path = cache_dir.join("newscenter").join("temp.wav");
    if !temp_path.is_file() {
        return "Error: temp wav not exit".to_string();
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
    println!("{}", temp_path_str);

    let command = app
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
        ]);

    match command.output().await {
        Ok(output) => {
            if output.status.success() {
                fs::remove_file(temp_path)
                    .await
                    .expect("cannot remove the temp file");

                match whisper::trancript_summary(app, &split_fold).await {
                    Ok(_) => "success: summary finished".to_string(),
                    Err(_) => "error: summary failed".to_string(),
                }
            } else {
                "error: ffmpeg error".to_string()
            }
        }
        Err(e) => format!("error from run command, {}", e),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn run_yt(app: tauri::AppHandle, url: &str) -> Result<String, String> {
    println!("run yt");

    if let Some(lang) = webvtt::get_sub_lang(&app, url).await {
        return webvtt::run_yt_vtt(app, url, &lang).await;
    }

    let cache_dir = app.path().cache_dir().unwrap();

    let temp_path = cache_dir.join("newscenter").join("temp.wav");
    let temp_path_str = temp_path.to_str().unwrap();

    let mut args = Vec::new();
    if let Ok(proxy_url) = std::env::var("PROXY_URL") {
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

    let output = app
        .shell()
        .sidecar("ytdown")
        .expect("should find the ytdown!")
        .args(args)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(run_ffmpeg(app).await)
    } else {
        let err_message = String::from_utf8_lossy(&output.stderr).to_string();
        Err(err_message)
    }
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
            whisper::summary,
            webvtt::run_yt_vtt,
            db::create_video,
            db::get_videos,
            db::update_video,
            db::delete_video,
            setting::load_settings,
            setting::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
