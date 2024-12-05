use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use thiserror::Error;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    id: i64,
    url: String,
    title: String,
    time_length: Option<i64>,
    transcripts: Option<String>,
    translate: Option<String>,
    summary: Option<String>,
    timestamp: Option<i64>,
}

pub fn init_db(app_handle: &AppHandle) -> Result<DataBase, DataBaseError> {
    let app_dir = app_handle.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("test.db");
    let connection = Connection::open(db_path)?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS Video (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL UNIQUE,
            title TEXT,
            time_length INTEGER,
            transcripts TEXT,
            translate TEXT,
            summary TEXT,
            timestamp INTEGER DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    Ok(DataBase(Mutex::new(connection)))
}

#[tauri::command]
pub fn create_video(db: State<DataBase>, url: String, title: String) -> Result<Video, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO Video (url, title) VALUES (?1, ?2)",
        params![url, title],
    )
    .map_err(|e| e.to_string())?;
    let db_id = db.last_insert_rowid();
    Ok(Video {
        id: db_id,
        url,
        title,
        time_length: None,
        transcripts: None,
        translate: None,
        summary: None,
        timestamp: None,
    })
}

#[tauri::command]
pub fn get_videos(db: State<DataBase>) -> Result<Vec<Video>, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare("SELECT * from Video")
        .map_err(|e| e.to_string())?;

    let video_iter = stmt
        .query_map([], |row| {
            Ok(Video {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                time_length: row.get(3).ok(),
                transcripts: row.get(4).ok(),
                translate: row.get(5).ok(),
                summary: row.get(6).ok(),
                timestamp: row.get(7).ok(),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut videos = Vec::new();
    for video in video_iter {
        videos.push(video.map_err(|e| e.to_string())?)
    }
    Ok(videos)
}

#[tauri::command]
pub fn update_video(
    db: State<DataBase>,
    id: i64,
    column: String,
    value: String,
) -> Result<(), String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;

    db.execute(
        &format!("UPDATE Video SET {} = ?1 Where id=?2", column),
        params![value, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_video(db: State<DataBase>, id: i64) -> Result<(), String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE From Video WHERE id =?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
