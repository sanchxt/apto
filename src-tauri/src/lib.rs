use chrono::Utc;
use log::{error, info};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State, Theme, Wry};
use thiserror::Error;
use window_vibrancy::{apply_acrylic, clear_acrylic};

mod models;
use models::Habit;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("testt {}", name)
}

#[tauri::command]
fn set_acrylic_effect(window: tauri::WebviewWindow, enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if enable {
            apply_acrylic(&window, Some((100, 100, 100, 50)))
                .map_err(|e| format!("Failed to apply acrylic: {}", e))?;
        } else {
            clear_acrylic(&window).map_err(|e| format!("Failed to clear acrylic: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
fn get_system_theme() -> String {
    match dark_light::detect() {
        dark_light::Mode::Dark => "dark".to_string(),
        dark_light::Mode::Light => "light".to_string(),
        dark_light::Mode::Default => "system".to_string(),
    }
}

// Store the current theme preference
static mut CURRENT_THEME: Option<String> = None;

#[tauri::command]
fn set_theme(window: tauri::WebviewWindow, theme: &str) -> Result<(), String> {
    // store the theme preference
    unsafe {
        CURRENT_THEME = Some(theme.to_string());
    }

    match theme {
        "light" => window
            .set_theme(Some(Theme::Light))
            .map_err(|e| e.to_string())?,
        "dark" => window
            .set_theme(Some(Theme::Dark))
            .map_err(|e| e.to_string())?,
        "system" => {
            // for system theme, we need to detect the current OS theme
            let system_theme = match dark_light::detect() {
                dark_light::Mode::Dark => Theme::Dark,
                dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
            };
            window
                .set_theme(Some(system_theme))
                .map_err(|e| e.to_string())?;
        }
        _ => return Err("Invalid theme".to_string()),
    }

    // emit theme changed event
    let _ = window.emit("tauri://theme-changed", theme);

    Ok(())
}

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
}

impl Serialize for DbError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Function to initialize the database connection and create tables
fn initialize_database(app_handle: &tauri::AppHandle<Wry>) -> Result<Connection, DbError> {
    // Get the path to the app's data directory (Tauri v2 API)
    let app_data_dir = app_handle.path().app_data_dir()?; // This now returns Result<PathBuf, Error>
                                                          // The '?' handles the Result directly.
                                                          // If Ok(path), 'app_data_dir' becomes PathBuf.
                                                          // If Err(e), the function returns Err(DbError::from(e)).

    // Create the data directory if it doesn't exist
    fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("apto_habits.db"); // Name your database file
    info!("Database path: {:?}", db_path); // Log the path

    // Open the connection
    let conn = Connection::open(&db_path)?;

    // Enable foreign key support (good practice)
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    // Create the 'habits' table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habits (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            name            TEXT NOT NULL,
            description     TEXT,
            frequency       TEXT NOT NULL,
            created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        )",
        [], // No parameters needed for this query
    )?;

    // Create the 'habit_completions' table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS habit_completions (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            habit_id        INTEGER NOT NULL,
            completed_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
            notes           TEXT,
            FOREIGN KEY (habit_id) REFERENCES habits (id) ON DELETE CASCADE
        )",
        [],
    )?;

    info!("Database initialized successfully.");
    Ok(conn)
}

#[tauri::command]
async fn add_habit(
    name: String,
    description: Option<String>,
    frequency: String,
    db_state: State<'_, DbState>, // Access the managed state
) -> Result<i64, String> {
    // Return the ID of the new habit or an error string
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?; // Lock the mutex

    let now = Utc::now().to_rfc3339(); // Use RFC3339 format compatible with SQLite function

    match conn.execute(
        "INSERT INTO habits (name, description, frequency, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, description, frequency, now],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            info!("Added habit '{}' with ID: {}", name, id);
            Ok(id)
        }
        Err(e) => {
            error!("Failed to add habit '{}': {}", name, e);
            Err(format!("Failed to add habit: {}", e)) // Convert error to String for frontend
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let db_conn =
                initialize_database(&app.handle()).expect("Failed to initialize database");
            app.manage(DbState(Mutex::new(db_conn)));

            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            apply_acrylic(&window, Some((100, 100, 100, 50))).expect("Failed to apply acrylic");

            // Set the initial theme to system (which means detect OS theme)
            let system_theme = match dark_light::detect() {
                dark_light::Mode::Dark => Theme::Dark,
                dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
            };

            window
                .set_theme(Some(system_theme))
                .expect("Failed to set theme");

            // emit initial theme event with the actual theme name
            let system_theme_name = match dark_light::detect() {
                dark_light::Mode::Dark => "dark",
                dark_light::Mode::Light | dark_light::Mode::Default => "light",
            };

            // emit initial theme event
            let _ = window.emit("tauri://theme-changed", system_theme_name);

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            set_acrylic_effect,
            get_system_theme,
            set_theme,
            add_habit
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
