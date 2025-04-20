use crate::db::init::DbState;
use crate::features::habits::models::{FrequencyPattern, Habit};
use crate::features::habits::utils::{deserialize_frequency, serialize_frequency};
use chrono::{DateTime, NaiveDate, Utc};
use log::info;
use rusqlite::params;
use serde_json;
use tauri::State;

#[tauri::command]
pub async fn add_habit(
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
pub async fn get_habits(db_state: State<'_, DbState>) -> Result<Vec<Habit>, String> {
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
pub async fn get_habit_by_id(id: i64, db_state: State<'_, DbState>) -> Result<Habit, String> {
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
pub async fn update_habit(
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
pub async fn delete_habit(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
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
pub async fn toggle_habit_active(
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
