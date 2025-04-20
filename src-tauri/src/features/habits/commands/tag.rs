use crate::db::init::DbState;
use rusqlite::params;
use tauri::State;

use crate::features::habits::models::HabitTag;

#[tauri::command]
pub async fn get_all_tags(db_state: State<'_, DbState>) -> Result<Vec<HabitTag>, String> {
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
pub async fn create_tag(
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
pub async fn update_tag(
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
pub async fn delete_tag(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    conn.execute("DELETE FROM habit_tags WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete tag: {}", e))?;

    Ok(())
}
