use crate::db::init::DbState;
use chrono::{DateTime, Utc};
use rusqlite::params;
use tauri::State;

use crate::features::habits::models::HabitCompletion;

#[tauri::command]
pub async fn get_habit_completions(
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
pub async fn update_habit_completion(
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
pub async fn delete_habit_completion(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute("DELETE FROM habit_completions WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete completion: {}", e))?;

    Ok(())
}
