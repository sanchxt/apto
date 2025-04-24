use crate::db::init::DbState;
use crate::features::notes::models::NoteTag;
use log::info;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn create_note_tag(
    name: String,
    color: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // insert, but silently handle unique constraint violations
    let result = conn.execute(
        "INSERT OR IGNORE INTO note_tags (name, color) VALUES (?, ?)",
        params![name, color],
    );

    match result {
        Ok(changes) => {
            if changes > 0 {
                // new tag created
                let tag_id = conn.last_insert_rowid();
                info!("Created note tag '{}' with ID: {}", name, tag_id);
                Ok(tag_id)
            } else {
                // tag exists, get its ID
                let tag_id: i64 = conn
                    .query_row(
                        "SELECT id FROM note_tags WHERE name = ?",
                        params![name],
                        |row| row.get(0),
                    )
                    .map_err(|e| format!("Failed to get existing tag ID: {}", e))?;

                info!("Using existing note tag '{}' with ID: {}", name, tag_id);
                Ok(tag_id)
            }
        }
        Err(e) => Err(format!("Failed to create note tag: {}", e)),
    }
}

#[tauri::command]
pub async fn get_all_note_tags(db_state: State<'_, DbState>) -> Result<Vec<NoteTag>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut tags = Vec::new();

    let mut stmt = conn
        .prepare("SELECT id, name, color FROM note_tags")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let tags_rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let color: Option<String> = row.get(2)?;

            Ok((id, name, color))
        })
        .map_err(|e| format!("Failed to query tags: {}", e))?;

    for tag_result in tags_rows {
        let (id, name, color) =
            tag_result.map_err(|e| format!("Failed to process tag row: {}", e))?;

        // create NoteTag struct
        let tag = NoteTag { id, name, color };

        tags.push(tag);
    }

    Ok(tags)
}

#[tauri::command]
pub async fn update_note_tag(
    id: i64,
    name: String,
    color: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // check for unique constraint before updating
    let existing_id: Result<i64, rusqlite::Error> = conn.query_row(
        "SELECT id FROM note_tags WHERE name = ? AND id != ?",
        params![name, id],
        |row| row.get(0),
    );

    if let Ok(_) = existing_id {
        return Err(format!("Tag name '{}' already exists", name));
    }

    // update the tag
    conn.execute(
        "UPDATE note_tags SET name = ?, color = ? WHERE id = ?",
        params![name, color, id],
    )
    .map_err(|e| format!("Failed to update note tag: {}", e))?;

    info!("Updated note tag with ID: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn delete_note_tag(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // check if the tag is used in any notes
    let usage_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_tag_mappings WHERE tag_id = ?",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check tag usage: {}", e))?;

    if usage_count > 0 {
        return Err(format!(
            "Cannot delete tag: it is used by {} notes",
            usage_count
        ));
    }

    // delete the tag
    conn.execute("DELETE FROM note_tags WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete note tag: {}", e))?;

    info!("Deleted note tag with ID: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn get_notes_by_tag(
    tag_name: String,
    db_state: State<'_, DbState>,
) -> Result<Vec<i64>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut note_ids = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT n.note_id
             FROM note_tag_mappings n
             JOIN note_tags t ON n.tag_id = t.id
             WHERE t.name = ?",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let note_rows = stmt
        .query_map(params![tag_name], |row| {
            let note_id: i64 = row.get(0)?;
            Ok(note_id)
        })
        .map_err(|e| format!("Failed to query notes by tag: {}", e))?;

    for note_id_result in note_rows {
        note_ids.push(note_id_result.map_err(|e| format!("Failed to process note ID: {}", e))?);
    }

    Ok(note_ids)
}
