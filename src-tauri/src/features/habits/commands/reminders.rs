use crate::db::init::DbState;
use rusqlite::params;
use tauri::State;

use crate::models::HabitReminder;

#[tauri::command]
pub async fn get_habit_reminders(
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
pub async fn create_habit_reminder(
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
pub async fn update_habit_reminder(
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
pub async fn delete_habit_reminder(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute("DELETE FROM habit_reminders WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete reminder: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_reminder(
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
