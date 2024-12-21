use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use thiserror::Error;
use tube_rs::AudioData;

pub struct DataBase(Mutex<Connection>);

#[derive(Error, Debug)]
pub enum DataBaseError {
    #[error("Faile to get app dir: {0}")]
    AppDirError(#[from] tauri::Error),

    #[error("Database connection error: {0}")]
    ConnectionError(#[from] rusqlite::Error),

    #[error("Create file error: {0}")]
    CreateFileError(#[from] std::io::Error),
}

//id, video_id, title, duration, upload_date, transcripts, summary, keywords, timestamp, thumbnail_url

#[derive(Debug, Serialize, Deserialize)]
pub struct Audio {
    id: i64,
    video_id: String,
    title: String,
    duration: u64,
    upload_date: String,
    transcripts: Option<String>,
    summary: Option<String>,
    keywords: String,
    timestamp: i64,
    thumbail_url: String,
}

pub fn init_db(app_handle: &AppHandle) -> Result<DataBase, DataBaseError> {
    let app_dir = app_handle.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("test.db");
    let connection = Connection::open(db_path)?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS audio (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            video_id TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            duration INTEGER NOT NULL,
            upload_date INTEGER NOT NULL,
            keywords TEXT,
            description TEXT,
            caption_lang TEXT,
            caption_url TEXT,
            audio_url TEXT NOT NULL,
            audio_filesize INTEGER NOT NULL,
            thumbnail_url TEXT NOT NULL,
            transcripts TEXT,
            summary TEXT,
            timestamp INTEGER DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    Ok(DataBase(Mutex::new(connection)))
}

pub fn create_video(db: State<DataBase>, audio_data: AudioData) -> Result<i64, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    let keywords = match audio_data.keywords {
        Some(array) => array.join(" "),
        None => "".to_string(),
    };
    db.execute(
        "INSERT INTO audio (
            video_id, title, duration, timestamp, description,
            caption_lang, caption_url, audio_url, audio_filesize, thumbnail_url, keywords
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            audio_data.video_id,
            audio_data.title,
            audio_data.duration,
            audio_data.timestamp,
            audio_data.description,
            audio_data.caption_lang,
            audio_data.caption_url,
            audio_data.audio_url,
            audio_data.audio_filesize,
            audio_data.thumbnail_url,
            keywords,
        ],
    )
    .map_err(|e| e.to_string())?;
    let db_id = db.last_insert_rowid();
    Ok(db_id)
}

#[tauri::command]
pub fn get_videos(db: State<DataBase>) -> Result<Vec<Audio>, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare("SELECT id, video_id, title, duration, upload_date, transcripts, summary, keywords, timestamp, thumbnail_url from audio ORDER BY id DESC")
        .map_err(|e| e.to_string())?;

    let video_iter = stmt
        .query_map([], |row| {
            Ok(Audio {
                id: row.get(0)?,
                video_id: row.get(1)?,
                title: row.get(2)?,
                duration: row.get(3)?,
                upload_date: row.get(4)?,
                transcripts: row.get(5).ok(),
                summary: row.get(6).ok(),
                keywords: row.get(7)?,
                timestamp: row.get(8)?,
                thumbail_url: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut videos = Vec::new();
    for video in video_iter {
        videos.push(video.map_err(|e| e.to_string())?)
    }
    Ok(videos)
}

pub fn get_caption_with_id(
    db: State<DataBase>,
    id: i64,
) -> Result<(Option<String>, Option<String>), String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.query_row(
        "Select caption_lang, caption_url from audio Where id=?1",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map_err(|e| e.to_string())
}

pub fn get_audio_url_with_id(db: State<DataBase>, id: i64) -> Result<(String, u64), String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.query_row(
        "Select audio_url, audio_filesize from audio Where id=?1",
        params![id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map_err(|e| e.to_string())
}

pub fn update_video(
    db: State<DataBase>,
    id: i64,
    column: String,
    value: String,
) -> Result<(), String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;

    db.execute(
        &format!("UPDATE audio SET {} = ?1 Where id=?2", column),
        params![value, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_video(db: State<DataBase>, id: i64) -> Result<(), String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE From audio WHERE id =?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
