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
pub struct User {
    id: i64,
    username: String,
    password: String,
}

pub fn init_db(app_handle: &AppHandle) -> Result<DataBase, DataBaseError> {
    let app_dir = app_handle.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("test.db");
    let connection = Connection::open(db_path)?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    Ok(DataBase(Mutex::new(connection)))
}

#[tauri::command]
pub fn create_user(
    db: State<DataBase>,
    username: String,
    password: String,
) -> Result<User, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO User (username, password) value (?1, ?2)",
        params![username, password],
    )
    .map_err(|e| e.to_string())?;
    let db_id = db.last_insert_rowid();
    Ok(User {
        id: db_id,
        username,
        password,
    })
}
