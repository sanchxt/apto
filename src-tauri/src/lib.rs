use chrono::{DateTime, Datelike, NaiveDate, Utc};
use log::{error, info};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State, Theme, Wry};
use thiserror::Error;
use window_vibrancy::{apply_acrylic, clear_acrylic};

mod models;
use models::{FrequencyPattern, Habit, HabitCompletion, HabitReminder, HabitStats, HabitTag};

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

// store the current theme preference
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
            // detect the current OS theme
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
fn initialize_database(app_handle: &tauri::AppHandle<Wry>) -> Result<Connection, DbError> {
    // path to the app's data directory
    let app_data_dir = app_handle.path().app_data_dir()?;

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

// helper function to convert FrequencyPattern to database format
fn serialize_frequency(
    frequency: &FrequencyPattern,
) -> Result<(String, String), serde_json::Error> {
    let frequency_type = match frequency {
        FrequencyPattern::Daily => "daily".to_string(),
        FrequencyPattern::Weekly { .. } => "weekly".to_string(),
        FrequencyPattern::Monthly { .. } => "monthly".to_string(),
        FrequencyPattern::Interval { .. } => "interval".to_string(),
        FrequencyPattern::Custom { .. } => "custom".to_string(),
    };

    let frequency_data = match frequency {
        FrequencyPattern::Daily => "{}".to_string(),
        FrequencyPattern::Weekly { days } => serde_json::to_string(&days)?,
        FrequencyPattern::Monthly { days } => serde_json::to_string(&days)?,
        FrequencyPattern::Interval { days } => serde_json::to_string(days)?,
        FrequencyPattern::Custom { pattern } => serde_json::to_string(pattern)?,
    };

    Ok((frequency_type, frequency_data))
}

// helper function to convert database format to FrequencyPattern
fn deserialize_frequency(
    freq_type: &str,
    freq_data: &str,
) -> Result<FrequencyPattern, serde_json::Error> {
    match freq_type {
        "daily" => Ok(FrequencyPattern::Daily),
        "weekly" => {
            let days: Vec<u32> = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Weekly { days })
        }
        "monthly" => {
            let days: Vec<u32> = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Monthly { days })
        }
        "interval" => {
            let days: u32 = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Interval { days })
        }
        "custom" => {
            let pattern: String = serde_json::from_str(freq_data)?;
            Ok(FrequencyPattern::Custom { pattern })
        }
        _ => {
            // Create a generic error instead of using Error::custom
            let msg = format!("Unknown frequency type: {}", freq_type);
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )))
        }
    }
}

#[tauri::command]
async fn add_habit(
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    frequency: FrequencyPattern,
    target_value: Option<f64>,
    target_unit: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    is_active: bool,
    priority: i32,
    start_date: String,
    end_date: Option<String>,
    reminder_time: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // parse start_date
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date format: {}", e))?;

    // serialize frequency pattern
    let (freq_type, freq_data) = serialize_frequency(&frequency)
        .map_err(|e| format!("Failed to serialize frequency: {}", e))?;

    // Insert the habit
    conn.execute(
        "INSERT INTO habits (
            name, description, category, frequency_type, frequency_data,
            target_value, target_unit, color, icon, is_active, priority,
            start_date, end_date, created_at, updated_at, reminder_time,
            current_streak, longest_streak
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0, 0
        )",
        params![
            name,
            description,
            category,
            freq_type,
            freq_data,
            target_value,
            target_unit,
            color,
            icon,
            is_active as i32,
            priority,
            start_date.to_string(),
            end_date,
            now,
            now,
            reminder_time
        ],
    )
    .map_err(|e| format!("Failed to add habit: {}", e))?;

    let habit_id = conn.last_insert_rowid();

    // process tags
    for tag_name in tags {
        // try to find if tag exists
        let mut stmt = conn
            .prepare("SELECT id FROM habit_tags WHERE name = ?")
            .map_err(|e| format!("Failed to prepare tag statement: {}", e))?;

        let tag_id: Result<i64, rusqlite::Error> =
            stmt.query_row(params![tag_name], |row| row.get(0));

        let tag_id = match tag_id {
            Ok(id) => id, // tag exists
            Err(_) => {
                // tag doesn't exist, create it
                conn.execute(
                    "INSERT INTO habit_tags (name) VALUES (?)",
                    params![tag_name],
                )
                .map_err(|e| format!("Failed to create tag: {}", e))?;

                conn.last_insert_rowid()
            }
        };

        // add tag mapping
        // Handle possible constraint violations if the mapping already exists
        let result = conn.execute(
            "INSERT OR IGNORE INTO habit_tag_mappings (habit_id, tag_id) VALUES (?, ?)",
            params![habit_id, tag_id],
        );

        if let Err(e) = result {
            return Err(format!("Failed to add tag mapping: {}", e));
        }
    }

    // add reminder if specified
    if let Some(time) = reminder_time {
        // default to daily reminders
        let default_days = vec![1, 2, 3, 4, 5, 6, 7]; // all days
        let days_json = serde_json::to_string(&default_days)
            .map_err(|e| format!("Failed to serialize reminder days: {}", e))?;

        conn.execute(
            "INSERT INTO habit_reminders (habit_id, time, days, is_enabled) VALUES (?, ?, ?, 1)",
            params![habit_id, time, days_json],
        )
        .map_err(|e| format!("Failed to add reminder: {}", e))?;
    }

    info!("Added habit '{}' with ID: {}", name, habit_id);
    Ok(habit_id)
}

#[tauri::command]
async fn get_habits(db_state: State<'_, DbState>) -> Result<Vec<Habit>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut habits = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT
            id, name, description, category, frequency_type, frequency_data,
            target_value, target_unit, color, icon, is_active, priority,
            start_date, end_date, created_at, updated_at, reminder_time,
            current_streak, longest_streak, last_completed
         FROM habits",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let habit_rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let description: Option<String> = row.get(2)?;
            let category: Option<String> = row.get(3)?;
            let frequency_type: String = row.get(4)?;
            let frequency_data: String = row.get(5)?;
            let target_value: Option<f64> = row.get(6)?;
            let target_unit: Option<String> = row.get(7)?;
            let color: Option<String> = row.get(8)?;
            let icon: Option<String> = row.get(9)?;
            let is_active: i32 = row.get(10)?;
            let priority: i32 = row.get(11)?;
            let start_date: String = row.get(12)?;
            let end_date: Option<String> = row.get(13)?;
            let created_at: String = row.get(14)?;
            let updated_at: String = row.get(15)?;
            let reminder_time: Option<String> = row.get(16)?;
            let current_streak: i32 = row.get(17)?;
            let longest_streak: i32 = row.get(18)?;
            let last_completed: Option<String> = row.get(19)?;

            Ok((
                id,
                name,
                description,
                category,
                frequency_type,
                frequency_data,
                target_value,
                target_unit,
                color,
                icon,
                is_active,
                priority,
                start_date,
                end_date,
                created_at,
                updated_at,
                reminder_time,
                current_streak,
                longest_streak,
                last_completed,
            ))
        })
        .map_err(|e| format!("Failed to query habits: {}", e))?;

    for habit_result in habit_rows {
        let (
            id,
            name,
            description,
            category,
            frequency_type,
            frequency_data,
            target_value,
            target_unit,
            color,
            icon,
            is_active,
            priority,
            start_date,
            end_date,
            created_at,
            updated_at,
            reminder_time,
            current_streak,
            longest_streak,
            last_completed,
        ) = habit_result.map_err(|e| format!("Failed to process habit row: {}", e))?;

        // get tags for this habit
        let mut tags_stmt = conn
            .prepare(
                "SELECT t.name FROM habit_tags t
             JOIN habit_tag_mappings m ON t.id = m.tag_id
             WHERE m.habit_id = ?",
            )
            .map_err(|e| format!("Failed to prepare tags statement: {}", e))?;

        let tags_rows = tags_stmt
            .query_map(params![id], |row| {
                let name: String = row.get(0)?;
                Ok(name)
            })
            .map_err(|e| format!("Failed to query tags: {}", e))?;

        let mut tags = Vec::new();
        for tag_result in tags_rows {
            tags.push(tag_result.map_err(|e| format!("Failed to process tag: {}", e))?);
        }

        // parse frequency
        let frequency = deserialize_frequency(&frequency_type, &frequency_data)
            .map_err(|e| format!("Failed to deserialize frequency: {}", e))?;

        // parse dates
        let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid start date: {}", e))?;

        let end_date = match end_date {
            Some(date) => Some(
                NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                    .map_err(|e| format!("Invalid end date: {}", e))?,
            ),
            None => None,
        };

        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| format!("Invalid updated_at date: {}", e))?
            .with_timezone(&Utc);

        let last_completed = match last_completed {
            Some(date) => Some(
                DateTime::parse_from_rfc3339(&date)
                    .map_err(|e| format!("Invalid last_completed date: {}", e))?
                    .with_timezone(&Utc),
            ),
            None => None,
        };

        // create Habit struct
        let habit = Habit {
            id,
            name,
            description,
            category,
            tags,
            frequency,
            target_value,
            target_unit,
            color,
            icon,
            is_active: is_active != 0,
            priority,
            start_date,
            end_date,
            created_at,
            updated_at,
            reminder_time,
            current_streak,
            longest_streak,
            last_completed,
        };

        habits.push(habit);
    }

    Ok(habits)
}

#[tauri::command]
async fn get_habit_by_id(id: i64, db_state: State<'_, DbState>) -> Result<Habit, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let habit_data = conn
        .query_row(
            "SELECT
                id, name, description, category, frequency_type, frequency_data,
                target_value, target_unit, color, icon, is_active, priority,
                start_date, end_date, created_at, updated_at, reminder_time,
                current_streak, longest_streak, last_completed
             FROM habits WHERE id = ?",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<f64>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, i32>(10)?,
                    row.get::<_, i32>(11)?,
                    row.get::<_, String>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, String>(14)?,
                    row.get::<_, String>(15)?,
                    row.get::<_, Option<String>>(16)?,
                    row.get::<_, i32>(17)?,
                    row.get::<_, i32>(18)?,
                    row.get::<_, Option<String>>(19)?,
                ))
            },
        )
        .map_err(|e| format!("Failed to get habit: {}", e))?;

    let (
        id,
        name,
        description,
        category,
        frequency_type,
        frequency_data,
        target_value,
        target_unit,
        color,
        icon,
        is_active,
        priority,
        start_date,
        end_date,
        created_at,
        updated_at,
        reminder_time,
        current_streak,
        longest_streak,
        last_completed,
    ) = habit_data;

    // get tags for this habit
    let mut tags_stmt = conn
        .prepare(
            "SELECT t.name FROM habit_tags t
         JOIN habit_tag_mappings m ON t.id = m.tag_id
         WHERE m.habit_id = ?",
        )
        .map_err(|e| format!("Failed to prepare tags statement: {}", e))?;

    let tags_rows = tags_stmt
        .query_map(params![id], |row| {
            let name: String = row.get(0)?;
            Ok(name)
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?;

    let mut tags = Vec::new();
    for tag_result in tags_rows {
        tags.push(tag_result.map_err(|e| format!("Failed to process tag: {}", e))?);
    }

    // parse frequency
    let frequency = deserialize_frequency(&frequency_type, &frequency_data)
        .map_err(|e| format!("Failed to deserialize frequency: {}", e))?;

    // parse dates
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date: {}", e))?;

    let end_date = match end_date {
        Some(date) => Some(
            NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|e| format!("Invalid end date: {}", e))?,
        ),
        None => None,
    };

    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Invalid created_at date: {}", e))?
        .with_timezone(&Utc);

    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|e| format!("Invalid updated_at date: {}", e))?
        .with_timezone(&Utc);

    let last_completed = match last_completed {
        Some(date) => Some(
            DateTime::parse_from_rfc3339(&date)
                .map_err(|e| format!("Invalid last_completed date: {}", e))?
                .with_timezone(&Utc),
        ),
        None => None,
    };

    // create Habit struct
    let habit = Habit {
        id,
        name,
        description,
        category,
        tags,
        frequency,
        target_value,
        target_unit,
        color,
        icon,
        is_active: is_active != 0,
        priority,
        start_date,
        end_date,
        created_at,
        updated_at,
        reminder_time,
        current_streak,
        longest_streak,
        last_completed,
    };

    Ok(habit)
}

#[tauri::command]
async fn update_habit(
    id: i64,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Vec<String>,
    frequency: FrequencyPattern,
    target_value: Option<f64>,
    target_unit: Option<String>,
    color: Option<String>,
    icon: Option<String>,
    is_active: bool,
    priority: i32,
    start_date: String,
    end_date: Option<String>,
    reminder_time: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // parse start_date
    let start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date format: {}", e))?;

    // serialize frequency pattern
    let (freq_type, freq_data) = serialize_frequency(&frequency)
        .map_err(|e| format!("Failed to serialize frequency: {}", e))?;

    // update the habit
    conn.execute(
        "UPDATE habits SET
            name = ?, description = ?, category = ?, frequency_type = ?, frequency_data = ?,
            target_value = ?, target_unit = ?, color = ?, icon = ?, is_active = ?, priority = ?,
            start_date = ?, end_date = ?, updated_at = ?, reminder_time = ?
         WHERE id = ?",
        params![
            name,
            description,
            category,
            freq_type,
            freq_data,
            target_value,
            target_unit,
            color,
            icon,
            is_active as i32,
            priority,
            start_date.to_string(),
            end_date,
            now,
            reminder_time,
            id
        ],
    )
    .map_err(|e| format!("Failed to update habit: {}", e))?;

    // delete existing tag mappings for this habit
    conn.execute(
        "DELETE FROM habit_tag_mappings WHERE habit_id = ?",
        params![id],
    )
    .map_err(|e| format!("Failed to delete tag mappings: {}", e))?;

    // add new tag mappings
    for tag_name in tags {
        // check if tag exists
        let mut stmt = conn
            .prepare("SELECT id FROM habit_tags WHERE name = ?")
            .map_err(|e| format!("Failed to prepare tag statement: {}", e))?;

        let tag_id: Result<i64, rusqlite::Error> =
            stmt.query_row(params![tag_name], |row| row.get(0));

        let tag_id = match tag_id {
            Ok(id) => id,
            Err(_) => {
                // create tag if it doesnt exist
                conn.execute(
                    "INSERT INTO habit_tags (name) VALUES (?)",
                    params![tag_name],
                )
                .map_err(|e| format!("Failed to create tag: {}", e))?;

                conn.last_insert_rowid()
            }
        };

        // tag mapping
        conn.execute(
            "INSERT OR IGNORE INTO habit_tag_mappings (habit_id, tag_id) VALUES (?, ?)",
            params![id, tag_id],
        )
        .map_err(|e| format!("Failed to add tag mapping: {}", e))?;
    }

    // update reminder if reminder_time is specified
    if let Some(time) = reminder_time {
        // check if a reminder exists
        let reminder_exists: bool = conn
            .query_row(
                "SELECT 1 FROM habit_reminders WHERE habit_id = ? LIMIT 1",
                params![id],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if reminder_exists {
            // update existing reminder
            let default_days = vec![1, 2, 3, 4, 5, 6, 7]; // all days
            let days_json = serde_json::to_string(&default_days)
                .map_err(|e| format!("Failed to serialize reminder days: {}", e))?;

            conn.execute(
                "UPDATE habit_reminders SET time = ?, days = ? WHERE habit_id = ?",
                params![time, days_json, id],
            )
            .map_err(|e| format!("Failed to update reminder: {}", e))?;
        } else {
            // create new reminder
            let default_days = vec![1, 2, 3, 4, 5, 6, 7]; // all days
            let days_json = serde_json::to_string(&default_days)
                .map_err(|e| format!("Failed to serialize reminder days: {}", e))?;

            conn.execute(
                "INSERT INTO habit_reminders (habit_id, time, days, is_enabled) VALUES (?, ?, ?, 1)",
                params![id, time, days_json],
            )
            .map_err(|e| format!("Failed to add reminder: {}", e))?;
        }
    } else {
        // if reminder_time is None, delete existing reminders
        conn.execute(
            "DELETE FROM habit_reminders WHERE habit_id = ?",
            params![id],
        )
        .map_err(|e| format!("Failed to delete reminders: {}", e))?;
    }

    info!("Updated habit with ID: {}", id);
    Ok(())
}

#[tauri::command]
async fn delete_habit(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute("DELETE FROM habits WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete habit: {}", e))?;

    info!("Deleted habit with ID: {}", id);
    Ok(())
}

#[tauri::command]
async fn toggle_habit_active(
    id: i64,
    is_active: bool,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE habits SET is_active = ?, updated_at = ? WHERE id = ?",
        params![is_active as i32, now, id],
    )
    .map_err(|e| format!("Failed to toggle habit active status: {}", e))?;

    info!(
        "Toggled active status to {} for habit with ID: {}",
        is_active, id
    );
    Ok(())
}

fn is_habit_due(
    frequency: &FrequencyPattern,
    reference_date: NaiveDate,
    last_completed: Option<DateTime<Utc>>,
) -> bool {
    let today = reference_date;

    match frequency {
        FrequencyPattern::Daily => {
            // for daily habits, it should be completed every day
            match last_completed {
                Some(last) => {
                    let last_date = last.date_naive();
                    // it's due if the last completion was before today
                    last_date < today
                }
                None => true, // never completed, so it's due
            }
        }
        FrequencyPattern::Weekly { days } => {
            // for weekly habits, check if today is one of the specified days
            let weekday = today.weekday().number_from_monday();
            if !days.contains(&weekday) {
                return false; // not due on this day of the week
            }

            // if it's one of the specified days, check if it was already completed today
            match last_completed {
                Some(last) => last.date_naive() < today,
                None => true,
            }
        }
        FrequencyPattern::Monthly { days } => {
            // for monthly habits, check if today is one of the specified days
            let day_of_month = today.day();
            if !days.contains(&day_of_month) {
                return false; // not due on this day of the month
            }

            // if it's one of the specified days, check if it was already completed today
            match last_completed {
                Some(last) => last.date_naive() < today,
                None => true,
            }
        }
        FrequencyPattern::Interval { days } => {
            // for interval habits, check if enough days have passed since the last completion
            match last_completed {
                Some(last) => {
                    let last_date = last.date_naive();
                    let days_since_last = today.signed_duration_since(last_date).num_days();
                    days_since_last >= *days as i64
                }
                None => true, // never completed, so it's due
            }
        }
        FrequencyPattern::Custom { pattern: _ } => {
            // for now, always assume it's due
            true
        }
    }
}

// check if a completion breaks the streak
fn breaks_streak(
    frequency: &FrequencyPattern,
    previous_completion: DateTime<Utc>,
    current_date: NaiveDate,
) -> bool {
    let prev_date = previous_completion.date_naive();

    match frequency {
        FrequencyPattern::Daily => {
            // for daily habits, streak breaks if more than 1 day passed
            let days_diff = current_date.signed_duration_since(prev_date).num_days();
            days_diff > 1
        }
        FrequencyPattern::Weekly { days } => {
            // for weekly, check if any expected days were missed
            let mut check_date = prev_date;
            while check_date < current_date {
                check_date = check_date.succ_opt().unwrap(); // move to next day
                let weekday = check_date.weekday().number_from_monday();

                // ff this day is in the frequency pattern and it's not the current date,
                // then we missed a day that breaks the streak
                if days.contains(&weekday) && check_date < current_date {
                    return true;
                }
            }
            false
        }
        FrequencyPattern::Monthly { days } => {
            // for monthly, check if any expected days were missed
            let mut check_date = prev_date;
            while check_date < current_date {
                check_date = check_date.succ_opt().unwrap(); // move to next day
                let day_of_month = check_date.day();

                // ff this day is in the frequency pattern and it's not the current date,
                // then we missed a day that breaks the streak
                if days.contains(&day_of_month) && check_date < current_date {
                    return true;
                }
            }
            false
        }
        FrequencyPattern::Interval { days } => {
            // for interval habits, streak breaks if more than specified interval passed
            let days_diff = current_date.signed_duration_since(prev_date).num_days();
            days_diff > *days as i64
        }
        FrequencyPattern::Custom { pattern: _ } => {
            // for now, assume no streak break
            false
        }
    }
}

#[tauri::command]
async fn update_habit_streaks(db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let today = Utc::now().date_naive();

    // get all active habits
    let mut habit_stmt = conn
        .prepare(
            "SELECT id, frequency_type, frequency_data, last_completed, current_streak
             FROM habits WHERE is_active = 1",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let habits_iter = habit_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i32>(4)?,
            ))
        })
        .map_err(|e| format!("Failed to query habits: {}", e))?;

    for habit_result in habits_iter {
        let (id, frequency_type, frequency_data, last_completed_str, current_streak) =
            habit_result.map_err(|e| format!("Failed to process habit: {}", e))?;

        // parse frequency
        let frequency = deserialize_frequency(&frequency_type, &frequency_data)
            .map_err(|e| format!("Failed to deserialize frequency: {}", e))?;

        // process last_completed
        let last_completed = match last_completed_str {
            Some(date) => Some(
                DateTime::parse_from_rfc3339(&date)
                    .map_err(|e| format!("Invalid last_completed date: {}", e))?
                    .with_timezone(&Utc),
            ),
            None => None,
        };

        // check if streak is broken
        let mut streak_broken = false;
        if let Some(last) = last_completed {
            let last_date = last.date_naive();

            if today > last_date {
                // check if habit was due on any day since last completion
                let mut check_date = last_date;
                while check_date < today {
                    check_date = check_date.succ_opt().unwrap();
                    if is_habit_due(&frequency, check_date, Some(last)) && check_date < today {
                        streak_broken = true;
                        break;
                    }
                }
            }
        }

        // Reset streak if broken
        if streak_broken && current_streak > 0 {
            conn.execute(
                "UPDATE habits SET current_streak = 0 WHERE id = ?",
                params![id],
            )
            .map_err(|e| format!("Failed to update streak: {}", e))?;

            info!("Reset streak for habit ID {} due to missed days", id);
        }
    }

    Ok(())
}

#[tauri::command]
async fn add_habit_completion(
    habit_id: i64,
    value: Option<f64>,
    notes: Option<String>,
    mood: Option<i32>,
    difficulty: Option<i32>,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let today = now.date_naive();

    // get current habit info to calculate streaks
    let (frequency_type, frequency_data, last_completed, current_streak, longest_streak): (
        String,
        String,
        Option<String>,
        i32,
        i32,
    ) = conn
        .query_row(
            "SELECT frequency_type, frequency_data, last_completed, current_streak, longest_streak
             FROM habits WHERE id = ?",
            params![habit_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| format!("Failed to get habit info: {}", e))?;

    // parse frequency
    let frequency = deserialize_frequency(&frequency_type, &frequency_data)
        .map_err(|e| format!("Failed to deserialize frequency: {}", e))?;

    // parse last completed
    let last_completed = match last_completed {
        Some(date) => Some(
            DateTime::parse_from_rfc3339(&date)
                .map_err(|e| format!("Invalid last_completed date: {}", e))?
                .with_timezone(&Utc),
        ),
        None => None,
    };

    // determine if this completion continues or resets streak
    let new_current_streak = match last_completed {
        Some(last) => {
            let last_date = last.date_naive();

            // skip duplicate completions on the same day
            if last_date == today {
                current_streak
            } else if breaks_streak(&frequency, last, today) {
                // streak broken, reset to 1
                1
            } else {
                // streak continues
                current_streak + 1
            }
        }
        None => 1, // first completion, streak of 1
    };

    // calculate new longest streak
    let new_longest_streak = std::cmp::max(longest_streak, new_current_streak);

    // insert the completion
    conn.execute(
        "INSERT INTO habit_completions (
            habit_id, completed_at, value, notes, mood, difficulty
        ) VALUES (?, ?, ?, ?, ?, ?)",
        params![habit_id, now_str, value, notes, mood, difficulty],
    )
    .map_err(|e| format!("Failed to add completion: {}", e))?;

    let completion_id = conn.last_insert_rowid();

    // update the habit's last_completed date and streak info
    conn.execute(
        "UPDATE habits SET
            last_completed = ?,
            current_streak = ?,
            longest_streak = ?
        WHERE id = ?",
        params![now_str, new_current_streak, new_longest_streak, habit_id],
    )
    .map_err(|e| format!("Failed to update habit: {}", e))?;

    info!(
        "Added completion for habit ID {} with completion ID: {}. Streak: {}",
        habit_id, completion_id, new_current_streak
    );
    Ok(completion_id)
}

#[tauri::command]
async fn get_all_tags(db_state: State<'_, DbState>) -> Result<Vec<HabitTag>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, name, color FROM habit_tags")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let tags_iter = stmt
        .query_map([], |row| {
            Ok(HabitTag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?;

    let mut tags = Vec::new();
    for tag_result in tags_iter {
        tags.push(tag_result.map_err(|e| format!("Failed to process tag: {}", e))?);
    }

    Ok(tags)
}

#[tauri::command]
async fn create_tag(
    name: String,
    color: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute(
        "INSERT INTO habit_tags (name, color) VALUES (?, ?)",
        params![name, color],
    )
    .map_err(|e| format!("Failed to create tag: {}", e))?;

    let tag_id = conn.last_insert_rowid();
    Ok(tag_id)
}

#[tauri::command]
async fn update_tag(
    id: i64,
    name: String,
    color: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute(
        "UPDATE habit_tags SET name = ?, color = ? WHERE id = ?",
        params![name, color, id],
    )
    .map_err(|e| format!("Failed to update tag: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn delete_tag(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute("DELETE FROM habit_tags WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete tag: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_habit_completions(
    habit_id: i64,
    db_state: State<'_, DbState>,
) -> Result<Vec<HabitCompletion>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, habit_id, completed_at, value, notes, mood, difficulty
             FROM habit_completions
             WHERE habit_id = ?
             ORDER BY completed_at DESC",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let completions_iter = stmt
        .query_map(params![habit_id], |row| {
            let completed_at_str: String = row.get(2)?;
            let completed_at = DateTime::parse_from_rfc3339(&completed_at_str)
                .map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(std::fmt::Error),
                    )
                })?
                .with_timezone(&Utc);

            Ok(HabitCompletion {
                id: row.get(0)?,
                habit_id: row.get(1)?,
                completed_at,
                value: row.get(3)?,
                notes: row.get(4)?,
                mood: row.get(5)?,
                difficulty: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query completions: {}", e))?;

    let mut completions = Vec::new();
    for completion_result in completions_iter {
        completions
            .push(completion_result.map_err(|e| format!("Failed to process completion: {}", e))?);
    }

    Ok(completions)
}

#[tauri::command]
async fn update_habit_completion(
    id: i64,
    value: Option<f64>,
    notes: Option<String>,
    mood: Option<i32>,
    difficulty: Option<i32>,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute(
        "UPDATE habit_completions SET value = ?, notes = ?, mood = ?, difficulty = ? WHERE id = ?",
        params![value, notes, mood, difficulty, id],
    )
    .map_err(|e| format!("Failed to update completion: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn delete_habit_completion(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute("DELETE FROM habit_completions WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete completion: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_habit_reminders(
    habit_id: i64,
    db_state: State<'_, DbState>,
) -> Result<Vec<HabitReminder>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, habit_id, time, days, is_enabled
             FROM habit_reminders
             WHERE habit_id = ?",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let reminders_iter = stmt
        .query_map(params![habit_id], |row| {
            let days_str: String = row.get(3)?;
            let days: Vec<u32> = serde_json::from_str(&days_str).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(std::fmt::Error),
                )
            })?;

            Ok(HabitReminder {
                id: row.get(0)?,
                habit_id: row.get(1)?,
                time: row.get(2)?,
                days,
                is_enabled: row.get::<_, i32>(4)? != 0,
            })
        })
        .map_err(|e| format!("Failed to query reminders: {}", e))?;

    let mut reminders = Vec::new();
    for reminder_result in reminders_iter {
        reminders.push(reminder_result.map_err(|e| format!("Failed to process reminder: {}", e))?);
    }

    Ok(reminders)
}

#[tauri::command]
async fn create_habit_reminder(
    habit_id: i64,
    time: String,
    days: Vec<u32>,
    is_enabled: bool,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let days_json =
        serde_json::to_string(&days).map_err(|e| format!("Failed to serialize days: {}", e))?;

    conn.execute(
        "INSERT INTO habit_reminders (habit_id, time, days, is_enabled) VALUES (?, ?, ?, ?)",
        params![habit_id, time, days_json, is_enabled as i32],
    )
    .map_err(|e| format!("Failed to create reminder: {}", e))?;

    let reminder_id = conn.last_insert_rowid();
    Ok(reminder_id)
}

#[tauri::command]
async fn update_habit_reminder(
    id: i64,
    time: String,
    days: Vec<u32>,
    is_enabled: bool,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let days_json =
        serde_json::to_string(&days).map_err(|e| format!("Failed to serialize days: {}", e))?;

    conn.execute(
        "UPDATE habit_reminders SET time = ?, days = ?, is_enabled = ? WHERE id = ?",
        params![time, days_json, is_enabled as i32, id],
    )
    .map_err(|e| format!("Failed to update reminder: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn delete_habit_reminder(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute("DELETE FROM habit_reminders WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete reminder: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn toggle_reminder(
    id: i64,
    is_enabled: bool,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute(
        "UPDATE habit_reminders SET is_enabled = ? WHERE id = ?",
        params![is_enabled as i32, id],
    )
    .map_err(|e| format!("Failed to toggle reminder: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_habit_stats(
    habit_id: i64,
    db_state: State<'_, DbState>,
) -> Result<HabitStats, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // Get total completions
    let total_completions: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM habit_completions WHERE habit_id = ?",
            params![habit_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get completion count: {}", e))?;

    // Get current and longest streaks from the habit table
    let (current_streak, longest_streak): (i32, i32) = conn
        .query_row(
            "SELECT current_streak, longest_streak FROM habits WHERE id = ?",
            params![habit_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Failed to get streak data: {}", e))?;

    // Calculate average value if applicable
    let average_value: Option<f64> = conn
        .query_row(
            "SELECT AVG(value) FROM habit_completions WHERE habit_id = ? AND value IS NOT NULL",
            params![habit_id],
            |row| row.get(0),
        )
        .ok();

    // Get frequency data for the habit to calculate completion rate
    let (frequency_type, frequency_data): (String, String) = conn
        .query_row(
            "SELECT frequency_type, frequency_data FROM habits WHERE id = ?",
            params![habit_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Failed to get frequency data: {}", e))?;

    // Get last 30 days completion status
    let mut last_30_days = HashMap::new();
    let today = Utc::now().date_naive();

    let mut stmt = conn
        .prepare(
            "SELECT strftime('%Y-%m-%d', completed_at) as completion_date
             FROM habit_completions
             WHERE habit_id = ?
             AND completed_at >= datetime('now', '-30 days')
             GROUP BY completion_date",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let dates_iter = stmt
        .query_map(params![habit_id], |row| {
            let date: String = row.get(0)?;
            Ok(date)
        })
        .map_err(|e| format!("Failed to query completion dates: {}", e))?;

    // Initialize all 30 days as false first
    for i in 0..30 {
        let date = today.checked_sub_days(chrono::Days::new(i as u64)).unwrap();
        last_30_days.insert(date.format("%Y-%m-%d").to_string(), false);
    }

    // Mark completed days as true
    for date_result in dates_iter {
        let date = date_result.map_err(|e| format!("Failed to process date: {}", e))?;
        last_30_days.insert(date, true);
    }

    // Calculate completion rate based on frequency and completed days
    let expected_completions = match frequency_type.as_str() {
        "daily" => 30, // Daily for 30 days
        "weekly" => {
            let days: Vec<u32> = serde_json::from_str(&frequency_data)
                .map_err(|e| format!("Failed to parse frequency data: {}", e))?;
            (30 / 7) * days.len() as i32 + 1 // Approx. number of occurrences in 30 days
        }
        "monthly" => 1, // Only happens once a month
        "interval" => {
            let days: u32 = serde_json::from_str(&frequency_data)
                .map_err(|e| format!("Failed to parse frequency data: {}", e))?;
            30 / days as i32 // Approx. number of occurrences in 30 days
        }
        _ => 30, // Default to daily
    };

    let completion_rate = if expected_completions > 0 {
        total_completions as f64 / expected_completions as f64
    } else {
        0.0
    };

    // Clamp to 0.0-1.0 range
    let completion_rate = completion_rate.min(1.0).max(0.0);

    Ok(HabitStats {
        habit_id,
        completion_rate,
        current_streak,
        longest_streak,
        total_completions,
        last_30_days,
        average_value,
    })
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

            // initial theme to system
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
            add_habit,
            get_habits,
            get_habit_by_id,
            update_habit,
            delete_habit,
            toggle_habit_active,
            add_habit_completion,
            // new tag functions
            get_all_tags,
            create_tag,
            update_tag,
            delete_tag,
            // new completion functions
            get_habit_completions,
            update_habit_completion,
            delete_habit_completion,
            // new reminder functions
            get_habit_reminders,
            create_habit_reminder,
            update_habit_reminder,
            delete_habit_reminder,
            toggle_reminder,
            // stats function
            get_habit_stats,
            // new streak update function
            update_habit_streaks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
