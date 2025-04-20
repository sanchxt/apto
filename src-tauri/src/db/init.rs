use log::{error, info};
use rusqlite::Connection;
use serde::Serialize;
use serde_json;
use std::fs;
use std::sync::Mutex;
use tauri::{Manager, Wry};
use thiserror::Error;

pub struct DbState(pub Mutex<Connection>);
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection failed: {0}")]
    Connection(#[from] rusqlite::Error),
    #[error("Failed to access application data directory: {0}")]
    AppDataDir(String),
    #[error("Filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tauri API error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Serialize for DbError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// initialize the database connection and create tables
pub fn initialize_database(app_handle: &tauri::AppHandle<Wry>) -> Result<Connection, DbError> {
    // path to the app's data directory
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| DbError::AppDataDir(e.to_string()))?;

    // create the data directory if it doesn't exist
    fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("apto_habits.db");
    info!("Database path: {:?}", db_path);

    // open connection
    let conn = Connection::open(&db_path)?;

    // enable foreign key support
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // create the 'habits' table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habits (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL,
            description     TEXT,
            category        TEXT,
            frequency_type  TEXT NOT NULL,
            frequency_data  TEXT NOT NULL,
            target_value    REAL,
            target_unit     TEXT,
            color           TEXT,
            icon            TEXT,
            is_active       INTEGER NOT NULL DEFAULT 1,
            priority        INTEGER NOT NULL DEFAULT 2,
            start_date      TEXT NOT NULL,
            end_date        TEXT,
            created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            reminder_time   TEXT,
            current_streak  INTEGER NOT NULL DEFAULT 0,
            longest_streak  INTEGER NOT NULL DEFAULT 0,
            last_completed  TEXT
        )",
        [],
    )?;

    // create the 'habit_tags' table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habit_tags (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL UNIQUE,
            color           TEXT
        )",
        [],
    )?;

    // Create the 'habit_tag_mappings' junction table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habit_tag_mappings (
            habit_id        INTEGER NOT NULL,
            tag_id          INTEGER NOT NULL,
            PRIMARY KEY (habit_id, tag_id),
            FOREIGN KEY (habit_id) REFERENCES habits (id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES habit_tags (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // create the 'habit_completions' table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habit_completions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            habit_id        INTEGER NOT NULL,
            completed_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            value           REAL,
            notes           TEXT,
            mood            INTEGER,
            difficulty      INTEGER,
            FOREIGN KEY (habit_id) REFERENCES habits (id) ON DELETE CASCADE
        )",
        [],
    )?;

    // create the 'habit_reminders' table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habit_reminders (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            habit_id        INTEGER NOT NULL,
            time            TEXT NOT NULL,
            days            TEXT NOT NULL,
            is_enabled      INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY (habit_id) REFERENCES habits (id) ON DELETE CASCADE
        )",
        [],
    )?;

    info!("Database initialized successfully.");
    Ok(conn)
}
