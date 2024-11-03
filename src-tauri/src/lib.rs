use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tokio::fs;
mod whisper;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

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
    let cache_dir = app.path().cache_dir().unwrap();

    let temp_path = cache_dir.join("newscenter").join("temp.wav");
    let temp_path_str = temp_path.to_str().unwrap();

    let command = app
        .shell()
        .sidecar("ytdown")
        .expect("should find the ytdown!")
        .args([
            "--proxy",
            "socks5://127.0.0.1:1095",
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
            temp_path_str,
            url,
        ]);

    match command.output().await {
        Ok(output) => {
            if output.status.success() {
                Ok(run_ffmpeg(app).await)
            } else {
                let err_message = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!("error: {}", err_message))
            }
        }
        Err(e) => Err(format!("error: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, run_yt])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
