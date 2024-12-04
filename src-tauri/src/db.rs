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
pub fn create_user(db: State<DataBase>, url: String, title: String) -> Result<Video, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO Video (url, title) value (?1, ?2)",
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
