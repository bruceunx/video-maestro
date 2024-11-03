use tauri::Manager;
use tauri_plugin_shell::ShellExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command(rename_all = "snake_case")]
async fn run_yt(app: tauri::AppHandle, url: &str) -> Result<String, String> {
    println!("url: {}", url);

    let cache_dir = app.path().cache_dir().unwrap();
    println!("cache dir: {}", cache_dir.to_str().unwrap());

    let temp_path = cache_dir.join("newscenter").join("temp.wav");
    let temp_path_str = temp_path.to_str().unwrap();

    println!("temp path_str: {}", &temp_path_str);
    let command = app.shell().sidecar("yt").unwrap().args([
        "--proxy",
        "socks5://127.0.0.1:1095",
        "--force-overwrites",
        "-x",
        "-f",
        "\"worstaudio[ext=webm]\"",
        "--extract-audio",
        "--audio-format",
        "wav",
        "--postprocessor-args",
        "\"-ar 16000 -ac 1\"",
        "-o",
        temp_path_str,
        url,
    ]);

    println!("run here");

    match command.output().await {
        Ok(output) => {
            if output.status.success() {
                println!("yt successful");
                Ok("yt success".to_string())
            } else {
                let err_message = String::from_utf8_lossy(&output.stderr).to_string();
                println!("error from command output {}", &err_message);
                Err(err_message.to_string())
            }
        }
        Err(e) => {
            println!("error from run command {}", e);
            Err(e.to_string())
        }
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
