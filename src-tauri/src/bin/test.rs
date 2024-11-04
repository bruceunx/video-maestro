use newscenter_lib::webvtt;

#[tokio::main]
async fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let vtt_path = current_dir
        .parent()
        .unwrap()
        .join("cache")
        .join("test.zh.vtt");

    let content = webvtt::extract_vtt_chunks(&vtt_path).await;
    if let Ok(chunks) = content {
        for chunk in chunks {
            println!("{}, length: {}", chunk, chunk.len());
        }
    }
}
