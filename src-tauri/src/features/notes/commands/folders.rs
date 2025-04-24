use crate::db::init::DbState;
use crate::features::notes::models::NoteFolder;
use chrono::{DateTime, Utc};
use log::info;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn create_folder(
    name: String,
    parent_id: Option<i64>,
    color: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // insert folder
    conn.execute(
        "INSERT INTO note_folders (
            name, parent_id, color, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5
        )",
        params![name, parent_id, color, now, now],
    )
    .map_err(|e| format!("Failed to create folder: {}", e))?;

    let folder_id = conn.last_insert_rowid();

    info!("Created folder '{}' with ID: {}", name, folder_id);
    Ok(folder_id)
}

#[tauri::command]
pub async fn get_folders(db_state: State<'_, DbState>) -> Result<Vec<NoteFolder>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut folders = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT
                id, name, parent_id, color, created_at, updated_at
             FROM note_folders",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let folder_rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let parent_id: Option<i64> = row.get(2)?;
            let color: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            let updated_at: String = row.get(5)?;

            Ok((id, name, parent_id, color, created_at, updated_at))
        })
        .map_err(|e| format!("Failed to query folders: {}", e))?;

    for folder_result in folder_rows {
        let (id, name, parent_id, color, created_at, updated_at) =
            folder_result.map_err(|e| format!("Failed to process folder row: {}", e))?;

        // parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| format!("Invalid updated_at date: {}", e))?
            .with_timezone(&Utc);

        // create NoteFolder struct
        let folder = NoteFolder {
            id,
            name,
            parent_id,
            color,
            created_at,
            updated_at,
        };

        folders.push(folder);
    }

    Ok(folders)
}

#[tauri::command]
pub async fn get_folder_by_id(id: i64, db_state: State<'_, DbState>) -> Result<NoteFolder, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let folder_data = conn
        .query_row(
            "SELECT
                id, name, parent_id, color, created_at, updated_at
             FROM note_folders WHERE id = ?",
            params![id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .map_err(|e| format!("Failed to get folder: {}", e))?;

    let (id, name, parent_id, color, created_at, updated_at) = folder_data;

    // parse dates
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Invalid created_at date: {}", e))?
        .with_timezone(&Utc);

    let updated_at = DateTime::parse_from_rfc3339(&updated_at)
        .map_err(|e| format!("Invalid updated_at date: {}", e))?
        .with_timezone(&Utc);

    // create NoteFolder struct
    let folder = NoteFolder {
        id,
        name,
        parent_id,
        color,
        created_at,
        updated_at,
    };

    Ok(folder)
}

#[tauri::command]
pub async fn update_folder(
    id: i64,
    name: String,
    parent_id: Option<i64>,
    color: Option<String>,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // update folder
    conn.execute(
        "UPDATE note_folders SET
            name = ?, parent_id = ?, color = ?, updated_at = ?
         WHERE id = ?",
        params![name, parent_id, color, now, id],
    )
    .map_err(|e| format!("Failed to update folder: {}", e))?;

    info!("Updated folder with ID: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn delete_folder(id: i64, db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // check if there are notes in this folder
    let note_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM notes WHERE folder_id = ?",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count notes in folder: {}", e))?;

    if note_count > 0 {
        return Err(format!(
            "Cannot delete folder: it contains {} notes",
            note_count
        ));
    }

    // check if there are subfolders
    let subfolder_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_folders WHERE parent_id = ?",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count subfolders: {}", e))?;

    if subfolder_count > 0 {
        return Err(format!(
            "Cannot delete folder: it contains {} subfolders",
            subfolder_count
        ));
    }

    // delete the folder
    conn.execute("DELETE FROM note_folders WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to delete folder: {}", e))?;

    info!("Deleted folder with ID: {}", id);
    Ok(())
}

#[tauri::command]
pub async fn get_subfolders(
    parent_id: Option<i64>,
    db_state: State<'_, DbState>,
) -> Result<Vec<NoteFolder>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut folders = Vec::new();

    // build the query based on whether parent_id is Some or None (root folders)
    let query = if parent_id.is_some() {
        "SELECT id, name, parent_id, color, created_at, updated_at FROM note_folders WHERE parent_id = ?"
    } else {
        "SELECT id, name, parent_id, color, created_at, updated_at FROM note_folders WHERE parent_id IS NULL"
    };

    let mut stmt = conn
        .prepare(query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    // execute query
    let mut rows = if let Some(id) = parent_id {
        stmt.query(params![id])
            .map_err(|e| format!("Failed to execute query: {}", e))?
    } else {
        stmt.query([])
            .map_err(|e| format!("Failed to execute query: {}", e))?
    };

    // process each row manually
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("Failed to get next row: {}", e))?
    {
        let id: i64 = row.get(0).map_err(|e| format!("Failed to get id: {}", e))?;
        let name: String = row
            .get(1)
            .map_err(|e| format!("Failed to get name: {}", e))?;
        let parent_id: Option<i64> = row
            .get(2)
            .map_err(|e| format!("Failed to get parent_id: {}", e))?;
        let color: Option<String> = row
            .get(3)
            .map_err(|e| format!("Failed to get color: {}", e))?;
        let created_at: String = row
            .get(4)
            .map_err(|e| format!("Failed to get created_at: {}", e))?;
        let updated_at: String = row
            .get(5)
            .map_err(|e| format!("Failed to get updated_at: {}", e))?;

        // parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at)
            .map_err(|e| format!("Invalid updated_at date: {}", e))?
            .with_timezone(&Utc);

        // create NoteFolder struct
        let folder = NoteFolder {
            id,
            name,
            parent_id,
            color,
            created_at,
            updated_at,
        };

        folders.push(folder);
    }

    Ok(folders)
}
