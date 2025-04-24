use crate::db::init::DbState;
use crate::features::notes::models::Note;
use chrono::{DateTime, Utc};
use log::info;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn create_note(
    title: String,
    content: String,
    folder_id: Option<i64>,
    tags: Vec<String>,
    is_pinned: bool,
    is_archived: bool,
    color: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // insert the note
    conn.execute(
        "INSERT INTO notes (
            title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
        )",
        params![
            title,
            content,
            folder_id,
            is_pinned as i32,
            is_archived as i32,
            color,
            now,
            now
        ],
    )
    .map_err(|e| format!("Failed to create note: {}", e))?;

    let note_id = conn.last_insert_rowid();

    // process tags
    for tag_name in tags {
        // find if tag exists
        let mut stmt = conn
            .prepare("SELECT id FROM note_tags WHERE name = ?")
            .map_err(|e| format!("Failed to prepare tag statement: {}", e))?;

        let tag_id: Result<i64, rusqlite::Error> =
            stmt.query_row(params![tag_name], |row| row.get(0));

        let tag_id = match tag_id {
            Ok(id) => id, // tag exists
            Err(_) => {
                // tag doesn't exist, create it
                conn.execute("INSERT INTO note_tags (name) VALUES (?)", params![tag_name])
                    .map_err(|e| format!("Failed to create tag: {}", e))?;

                conn.last_insert_rowid()
            }
        };

        // add tag mapping
        let result = conn.execute(
            "INSERT OR IGNORE INTO note_tag_mappings (note_id, tag_id) VALUES (?, ?)",
            params![note_id, tag_id],
        );

        if let Err(e) = result {
            return Err(format!("Failed to add tag mapping: {}", e));
        }
    }

    // create initial revision
    conn.execute(
        "INSERT INTO note_revisions (note_id, content, created_at) VALUES (?, ?, ?)",
        params![note_id, content, now],
    )
    .map_err(|e| format!("Failed to create initial revision: {}", e))?;

    info!("Created note '{}' with ID: {}", title, note_id);
    Ok(note_id)
}

#[tauri::command]
pub async fn get_notes(db_state: State<'_, DbState>) -> Result<Vec<Note>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut notes = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT
                id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at
             FROM notes",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let note_rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let title: String = row.get(1)?;
            let content: String = row.get(2)?;
            let folder_id: Option<i64> = row.get(3)?;
            let is_pinned: i32 = row.get(4)?;
            let is_archived: i32 = row.get(5)?;
            let color: Option<String> = row.get(6)?;
            let created_at: String = row.get(7)?;
            let updated_at: String = row.get(8)?;

            Ok((
                id,
                title,
                content,
                folder_id,
                is_pinned,
                is_archived,
                color,
                created_at,
                updated_at,
            ))
        })
        .map_err(|e| format!("Failed to query notes: {}", e))?;

    for note_result in note_rows {
        let (id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at) =
            note_result.map_err(|e| format!("Failed to process note row: {}", e))?;

        // get tags for this note
        let mut tags_stmt = conn
            .prepare(
                "SELECT t.name FROM note_tags t
                 JOIN note_tag_mappings m ON t.id = m.tag_id
                 WHERE m.note_id = ?",
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

        // parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| format!("Invalid updated_at date: {}", e))?
            .with_timezone(&Utc);

        // create nbote struct
        let note = Note {
            id,
            title,
            content,
            folder_id,
            tags,
            is_pinned: is_pinned != 0,
            is_archived: is_archived != 0,
            color,
            created_at,
            updated_at,
        };

        notes.push(note);
    }

    Ok(notes)
}

#[tauri::command]
pub async fn get_note_by_id(id: i64, db_state: State<'_, DbState>) -> Result<Note, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let note_data = conn
        .query_row(
            "SELECT
                id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at
             FROM notes WHERE id = ?",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                ))
            },
        )
        .map_err(|e| format!("Failed to get note: {}", e))?;

    let (id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at) =
        note_data;

    // get tags for this note
    let mut tags_stmt = conn
        .prepare(
            "SELECT t.name FROM note_tags t
             JOIN note_tag_mappings m ON t.id = m.tag_id
             WHERE m.note_id = ?",
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

    // parse dates
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Invalid created_at date: {}", e))?
        .with_timezone(&Utc);

    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|e| format!("Invalid updated_at date: {}", e))?
        .with_timezone(&Utc);

    // create note struct
    let note = Note {
        id,
        title,
        content,
        folder_id,
        tags,
        is_pinned: is_pinned != 0,
        is_archived: is_archived != 0,
        color,
        created_at,
        updated_at,
    };

    Ok(note)
}

#[tauri::command]
pub async fn update_note(
    id: i64,
    title: String,
    content: String,
    folder_id: Option<i64>,
    tags: Vec<String>,
    is_pinned: bool,
    is_archived: bool,
    color: Option<String>,
    create_revision: bool,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // get the current content if revision is needed
    let current_content = if create_revision {
        conn.query_row(
            "SELECT content FROM notes WHERE id = ?",
            params![id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|e| format!("Failed to get current note content: {}", e))?
    } else {
        String::new() // empty string if no revision needed
    };

    // update the note
    conn.execute(
        "UPDATE notes SET
            title = ?, content = ?, folder_id = ?, is_pinned = ?, is_archived = ?,
            color = ?, updated_at = ?
         WHERE id = ?",
        params![
            title,
            content,
            folder_id,
            is_pinned as i32,
            is_archived as i32,
            color,
            now,
            id
        ],
    )
    .map_err(|e| format!("Failed to update note: {}", e))?;

    // Create a revision if requested
    if create_revision && !current_content.is_empty() {
        conn.execute(
            "INSERT INTO note_revisions (note_id, content, created_at) VALUES (?, ?, ?)",
            params![id, current_content, now],
        )
        .map_err(|e| format!("Failed to create revision: {}", e))?;
    }

    // delete existing tag mappings for this note
    conn.execute(
        "DELETE FROM note_tag_mappings WHERE note_id = ?",
        params![id],
    )
    .map_err(|e| format!("Failed to delete tag mappings: {}", e))?;

    // add new tag mappings
    for tag_name in tags {
        // check if tag exists
        let mut stmt = conn
            .prepare("SELECT id FROM note_tags WHERE name = ?")
            .map_err(|e| format!("Failed to prepare tag statement: {}", e))?;

        let tag_id: Result<i64, rusqlite::Error> =
            stmt.query_row(params![tag_name], |row| row.get(0));

        let tag_id = match tag_id {
            Ok(id) => id,
            Err(_) => {
                // create tag if it doesn't exist
                conn.execute("INSERT INTO note_tags (name) VALUES (?)", params![tag_name])
                    .map_err(|e| format!("Failed to create tag: {}", e))?;

                conn.last_insert_rowid()
            }
        };

        // tag mapping
        conn.execute(
            "INSERT OR IGNORE INTO note_tag_mappings (note_id, tag_id) VALUES (?, ?)",
            params![id, tag_id],
        )
        .map_err(|e| format!("Failed to add tag mapping: {}", e))?;
    }

    info!("Updated note with ID: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn delete_note(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // delete the note
    conn.execute("DELETE FROM notes WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete note: {}", e))?;

    info!("Deleted note with ID: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn toggle_note_pin(
    id: i64,
    is_pinned: bool,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE notes SET is_pinned = ?, updated_at = ? WHERE id = ?",
        params![is_pinned as i32, now, id],
    )
    .map_err(|e| format!("Failed to toggle note pin status: {}", e))?;

    info!(
        "Toggled pin status to {} for note with ID: {}",
        is_pinned, id
    );
    Ok(())
}

#[tauri::command]
pub async fn toggle_note_archive(
    id: i64,
    is_archived: bool,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE notes SET is_archived = ?, updated_at = ? WHERE id = ?",
        params![is_archived as i32, now, id],
    )
    .map_err(|e| format!("Failed to toggle note archive status: {}", e))?;

    info!(
        "Toggled archive status to {} for note with ID: {}",
        is_archived, id
    );
    Ok(())
}

#[tauri::command]
pub async fn get_notes_by_folder(
    folder_id: Option<i64>,
    db_state: State<'_, DbState>,
) -> Result<Vec<Note>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut notes = Vec::new();

    // build the query based on whether folder_id is Some or None
    let query = if folder_id.is_some() {
        "SELECT id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at FROM notes WHERE folder_id = ?"
    } else {
        "SELECT id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at FROM notes WHERE folder_id IS NULL"
    };

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    // execute query
    let mut rows = if let Some(id) = folder_id {
        stmt.query(params![id])
            .map_err(|e| format!("Failed to execute query: {}", e))?
    } else {
        stmt.query([])
            .map_err(|e| format!("Failed to execute query: {}", e))?
    };

    // process each row
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to get next row: {}", e))?
    {
        let id: i64 = row.get(0).map_err(|e| format!("Failed to get id: {}", e))?;
        let title: String = row
            .get(1)
            .map_err(|e| format!("Failed to get title: {}", e))?;
        let content: String = row
            .get(2)
            .map_err(|e| format!("Failed to get content: {}", e))?;
        let folder_id: Option<i64> = row
            .get(3)
            .map_err(|e| format!("Failed to get folder_id: {}", e))?;
        let is_pinned: i32 = row
            .get(4)
            .map_err(|e| format!("Failed to get is_pinned: {}", e))?;
        let is_archived: i32 = row
            .get(5)
            .map_err(|e| format!("Failed to get is_archived: {}", e))?;
        let color: Option<String> = row
            .get(6)
            .map_err(|e| format!("Failed to get color: {}", e))?;
        let created_at: String = row
            .get(7)
            .map_err(|e| format!("Failed to get created_at: {}", e))?;
        let updated_at: String = row
            .get(8)
            .map_err(|e| format!("Failed to get updated_at: {}", e))?;

        // get tags for this note
        let mut tags_stmt = conn
            .prepare(
                "SELECT t.name FROM note_tags t
                 JOIN note_tag_mappings m ON t.id = m.tag_id
                 WHERE m.note_id = ?",
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

        // parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| format!("Invalid updated_at date: {}", e))?
            .with_timezone(&Utc);

        // create Note struct
        let note = Note {
            id,
            title,
            content,
            folder_id,
            tags,
            is_pinned: is_pinned != 0,
            is_archived: is_archived != 0,
            color,
            created_at,
            updated_at,
        };

        notes.push(note);
    }

    Ok(notes)
}

#[tauri::command]
pub async fn search_notes(
    query: String,
    db_state: State<'_, DbState>,
) -> Result<Vec<Note>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let search_query = format!("%{}%", query);
    let mut notes = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT
                id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at
             FROM notes
             WHERE title LIKE ? OR content LIKE ?",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let note_rows = stmt
        .query_map(params![search_query, search_query], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<i64>>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
            ))
        })
        .map_err(|e| format!("Failed to query notes: {}", e))?;

    for note_result in note_rows {
        let (id, title, content, folder_id, is_pinned, is_archived, color, created_at, updated_at) =
            note_result.map_err(|e| format!("Failed to process note row: {}", e))?;

        // get tags for this note
        let mut tags_stmt = conn
            .prepare(
                "SELECT t.name FROM note_tags t
                 JOIN note_tag_mappings m ON t.id = m.tag_id
                 WHERE m.note_id = ?",
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

        // parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| format!("Invalid updated_at date: {}", e))?
            .with_timezone(&Utc);

        // create note struct
        let note = Note {
            id,
            title,
            content,
            folder_id,
            tags,
            is_pinned: is_pinned != 0,
            is_archived: is_archived != 0,
            color,
            created_at,
            updated_at,
        };

        notes.push(note);
    }

    Ok(notes)
}
