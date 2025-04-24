use crate::db::init::DbState;
use crate::features::notes::models::NoteRevision;
use chrono::{DateTime, Utc};
use log::info;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn get_note_revisions(
    note_id: i64,
    db_state: State<'_, DbState>,
) -> Result<Vec<NoteRevision>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut revisions = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT id, note_id, content, created_at
             FROM note_revisions
             WHERE note_id = ?
             ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let revision_rows = stmt
        .query_map(params![note_id], |row| {
            let id: i64 = row.get(0)?;
            let note_id: i64 = row.get(1)?;
            let content: String = row.get(2)?;
            let created_at: String = row.get(3)?;

            Ok((id, note_id, content, created_at))
        })
        .map_err(|e| format!("Failed to query revisions: {}", e))?;

    for revision_result in revision_rows {
        let (id, note_id, content, created_at) =
            revision_result.map_err(|e| format!("Failed to process revision row: {}", e))?;

        // parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        // create NoteRevision struct
        let revision = NoteRevision {
            id,
            note_id,
            content,
            created_at,
        };

        revisions.push(revision);
    }

    Ok(revisions)
}

#[tauri::command]
pub async fn create_revision(
    note_id: i64,
    content: String,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // insert revision
    conn.execute(
        "INSERT INTO note_revisions (
            note_id, content, created_at
        ) VALUES (
            ?1, ?2, ?3
        )",
        params![note_id, content, now],
    )
    .map_err(|e| format!("Failed to create revision: {}", e))?;

    let revision_id = conn.last_insert_rowid();

    info!(
        "Created revision for note ID: {} with revision ID: {}",
        note_id, revision_id
    );
    Ok(revision_id)
}

#[tauri::command]
pub async fn restore_revision(
    revision_id: i64,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // get the revision data
    let (note_id, content): (i64, String) = conn
        .query_row(
            "SELECT note_id, content FROM note_revisions WHERE id = ?",
            params![revision_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Failed to get revision: {}", e))?;

    // get current content of the note to save as a new revision
    let current_content: String = conn
        .query_row(
            "SELECT content FROM notes WHERE id = ?",
            params![note_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get current note content: {}", e))?;

    // save current content as a new revision
    conn.execute(
        "INSERT INTO note_revisions (
            note_id, content, created_at
        ) VALUES (
            ?1, ?2, ?3
        )",
        params![note_id, current_content, now],
    )
    .map_err(|e| format!("Failed to save current content as revision: {}", e))?;

    // update the note with the revision content
    conn.execute(
        "UPDATE notes SET content = ?, updated_at = ? WHERE id = ?",
        params![content, now, note_id],
    )
    .map_err(|e| format!("Failed to update note with revision content: {}", e))?;

    info!(
        "Restored revision ID: {} for note ID: {}",
        revision_id, note_id
    );
    Ok(())
}

#[tauri::command]
pub async fn delete_revision(revision_id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // get the note ID for logging
    let note_id: i64 = conn
        .query_row(
            "SELECT note_id FROM note_revisions WHERE id = ?",
            params![revision_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get note ID for revision: {}", e))?;

    // delete the revision
    conn.execute(
        "DELETE FROM note_revisions WHERE id = ?",
        params![revision_id],
    )
    .map_err(|e| format!("Failed to delete revision: {}", e))?;

    info!(
        "Deleted revision ID: {} for note ID: {}",
        revision_id, note_id
    );
    Ok(())
}

#[tauri::command]
pub async fn get_revision_by_id(
    revision_id: i64,
    db_state: State<'_, DbState>,
) -> Result<NoteRevision, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let revision_data = conn
        .query_row(
            "SELECT id, note_id, content, created_at FROM note_revisions WHERE id = ?",
            params![revision_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .map_err(|e| format!("Failed to get revision: {}", e))?;

    let (id, note_id, content, created_at) = revision_data;

    // parse dates
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Invalid created_at date: {}", e))?
        .with_timezone(&Utc);

    // create NoteRevision struct
    let revision = NoteRevision {
        id,
        note_id,
        content,
        created_at,
    };

    Ok(revision)
}

#[tauri::command]
pub async fn clean_old_revisions(
    note_id: i64,
    keep_count: u32,
    db_state: State<'_, DbState>,
) -> Result<u32, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // count the total number of revisions
    let total_revisions: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_revisions WHERE note_id = ?",
            params![note_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count revisions: {}", e))?;

    if total_revisions <= keep_count {
        // no need to delete anything
        return Ok(0);
    }

    // calculate how many to delete
    let to_delete = total_revisions - keep_count;

    // delete oldest revisions beyond the keep_count
    let result = conn.execute(
        "DELETE FROM note_revisions
         WHERE id IN (
             SELECT id FROM note_revisions
             WHERE note_id = ?
             ORDER BY created_at ASC
             LIMIT ?
         )",
        params![note_id, to_delete],
    );

    match result {
        Ok(deleted_count) => {
            info!(
                "Cleaned {} old revisions for note ID: {}",
                deleted_count, note_id
            );
            Ok(deleted_count as u32)
        }
        Err(e) => Err(format!("Failed to clean old revisions: {}", e)),
    }
}
