use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, BufReader};

fn parse_timestamp(timestamp: &str) -> Option<Duration> {
    let parts: Vec<&str> = timestamp.split(&['.', ':'][..]).collect();
    if parts.len() < 3 {
        return None;
    }
    let hour: u64 = parts[0].parse().ok()?;
    let minute: u64 = parts[1].parse().ok()?;
    let second: u64 = parts[2].parse().ok()?;
    let millis: u64 = parts.get(4).and_then(|&ms| ms.parse().ok()).unwrap_or(0);
    Some(Duration::from_secs(hour * 3600 + minute * 60 + second) + Duration::from_millis(millis))
}

pub async fn extract_vtt_chunks(vtt_file: &Path) -> Result<Vec<String>, String> {
    let interval = Duration::from_secs(500);
    let mut last_split_duration = Duration::ZERO;

    let mut chunks = Vec::new();
    let mut text_parse = Vec::new();
    let file = File::open(vtt_file).await.map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut is_inblock: bool = false;

    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        if line.contains("-->") {
            if let Some((start, _)) = line.split_once(" --> ") {
                is_inblock = true;
                if let Some(start_time) = parse_timestamp(start) {
                    if start_time >= last_split_duration + interval {
                        last_split_duration = start_time;
                        chunks.push(text_parse.join(" "));
                        text_parse.clear();
                    }
                }
            }
        } else if !line.is_empty() & is_inblock {
            text_parse.push(line);
        } else if line.is_empty() {
            is_inblock = false;
        }
    }
    if text_parse.len() > 0 {
        chunks.push(text_parse.join(" "))
    }

    Ok(chunks)
}

pub async fn run_yt_vtt(app: &tauri::AppHandle, url: &str, lang: &str) -> Result<PathBuf, String> {
    println!("run yt download vtt");
    let cache_dir = app.path().cache_dir().unwrap();
    let vtt_dir = cache_dir.join("newscenter").join("vtt");
    if !vtt_dir.is_dir() {
        fs::create_dir(&vtt_dir)
            .await
            .expect("create vtt fold failed");
    }

    let temp_path = vtt_dir.join("temp");
    let temp_path_str = temp_path.to_str().unwrap();

    let mut args = Vec::new();
    if let Ok(proxy_url) = std::env::var("PROXY_URL") {
        args.push("--proxy".to_string());
        args.push(proxy_url);
    }
    let standard_args = vec!["--skip-download", "--write-subs", "--sub-lang", lang, "-o"];
    args.extend(standard_args.into_iter().map(String::from));
    args.push(temp_path_str.to_string());
    args.push(url.to_string());

    let command = app
        .shell()
        .sidecar("ytdown")
        .expect("should find the ytdown!")
        .args(args);

    match command.output().await {
        Ok(output) => {
            if output.status.success() {
                let vtt_path = vtt_dir.join(format!("temp.{}.vtt", lang));
                Ok(vtt_path)
                // match handle_summarize(app, &vtt_path).await {
                //     Ok(_) => Ok("success: finished".to_string()),
                //     Err(_) => Err("error from summarizing".to_string()),
                // }
            } else {
                let err_message = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!("error: {}", err_message))
            }
        }
        Err(e) => Err(format!("error: {}", e)),
    }
}

pub async fn get_sub_lang(app: &tauri::AppHandle, url: &str) -> Option<String> {
    println!("get_sub_lang");
    let mut args = Vec::new();
    if let Ok(proxy_url) = std::env::var("PROXY_URL") {
        args.push("--proxy".to_string());
        args.push(proxy_url);
    }
    args.push("--list-subs".to_string());
    args.push(url.to_string());
    let command = app
        .shell()
        .sidecar("ytdown")
        .expect("should find the ytdown!")
        .args(args);

    let mut lang_attention = false;

    match command.output().await {
        Ok(output) => {
            if output.status.success() {
                let output_str = std::str::from_utf8(&output.stdout).unwrap();

                if output_str.contains("has no subtitles") {
                    return None;
                }
                for line in output_str.lines() {
                    if line.is_empty() {
                        continue;
                    }
                    if line.contains("Available automatic captions") {
                        if !lang_attention {
                            return None;
                        }
                    }
                    if line.starts_with("Language") {
                        lang_attention = true;
                        continue;
                    }
                    if lang_attention {
                        if let Some((lang, _)) = line.split_once(" ") {
                            return Some(lang.to_string());
                        }
                        break;
                    }
                }
                return None;
            } else {
                return None;
            }
        }
        Err(_) => return None,
    }
}
