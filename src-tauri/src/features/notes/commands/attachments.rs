use crate::db::init::DbState;
use crate::features::notes::models::NoteAttachment;
use chrono::{DateTime, Utc};
use log::{error, info};
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

// get the attachments directory path
fn get_attachments_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let attachments_dir = app_data_dir.join("note_attachments");

    // create directory if it doesnt exist
    if !attachments_dir.exists() {
        fs::create_dir_all(&attachments_dir)
            .map_err(|e| format!("Failed to create attachments directory: {}", e))?;
    }

    Ok(attachments_dir)
}

#[tauri::command]
pub async fn add_attachment(
    note_id: i64,
    file_path: String,
    app_handle: tauri::AppHandle,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now().to_rfc3339();

    // get the source file path
    let source_path = std::path::Path::new(&file_path);

    // extract file information
    let file_name = source_path
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?
        .to_string_lossy()
        .to_string();

    let file_type = source_path
        .extension()
        .map(|ext| ext.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // get file size
    let metadata =
        fs::metadata(&file_path).map_err(|e| format!("Failed to read file metadata: {}", e))?;
    let file_size = metadata.len() as i64;

    // generate a unique filename to avoid collisions
    // using timestamp and random suffix
    let timestamp = Utc::now().timestamp();
    let random_suffix = rand::random::<u32>();
    let unique_filename = format!("{}_{}_{}_{}", note_id, timestamp, random_suffix, file_name);

    // get the destination directory
    let attachments_dir = get_attachments_dir(&app_handle)?;
    let destination_path = attachments_dir.join(&unique_filename);

    // copy the file to the attachments directory
    fs::copy(source_path, &destination_path)
        .map_err(|e| format!("Failed to copy file to attachments directory: {}", e))?;

    // store the relative path in the database
    let stored_path = format!("note_attachments/{}", unique_filename);

    // insert attachment record
    conn.execute(
        "INSERT INTO note_attachments (
            note_id, file_name, file_path, file_type, file_size, created_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6
        )",
        params![note_id, file_name, stored_path, file_type, file_size, now],
    )
    .map_err(|e| format!("Failed to add attachment record: {}", e))?;

    let attachment_id = conn.last_insert_rowid();

    info!(
        "Added attachment '{}' with ID: {} to note ID: {}",
        file_name, attachment_id, note_id
    );
    Ok(attachment_id)
}

#[tauri::command]
pub async fn get_note_attachments(
    note_id: i64,
    db_state: State<'_, DbState>,
) -> Result<Vec<NoteAttachment>, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let mut attachments = Vec::new();

    let mut stmt = conn
        .prepare(
            "SELECT
                id, note_id, file_name, file_path, file_type, file_size, created_at
             FROM note_attachments
             WHERE note_id = ?",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let attachment_rows = stmt
        .query_map(params![note_id], |row| {
            let id: i64 = row.get(0)?;
            let note_id: i64 = row.get(1)?;
            let file_name: String = row.get(2)?;
            let file_path: String = row.get(3)?;
            let file_type: String = row.get(4)?;
            let file_size: i64 = row.get(5)?;
            let created_at: String = row.get(6)?;

            Ok((
                id, note_id, file_name, file_path, file_type, file_size, created_at,
            ))
        })
        .map_err(|e| format!("Failed to query attachments: {}", e))?;

    for attachment_result in attachment_rows {
        let (id, note_id, file_name, file_path, file_type, file_size, created_at) =
            attachment_result.map_err(|e| format!("Failed to process attachment row: {}", e))?;

        // parse dates
        let created_at = DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| format!("Invalid created_at date: {}", e))?
            .with_timezone(&Utc);

        // create NoteAttachment struct
        let attachment = NoteAttachment {
            id,
            note_id,
            file_name,
            file_path,
            file_type,
            file_size,
            created_at,
        };

        attachments.push(attachment);
    }

    Ok(attachments)
}

#[tauri::command]
pub async fn delete_attachment(
    attachment_id: i64,
    app_handle: tauri::AppHandle,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // get the file path before deleting the record
    let file_path: String = conn
        .query_row(
            "SELECT file_path FROM note_attachments WHERE id = ?",
            params![attachment_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get attachment file path: {}", e))?;

    // delete the record from the database
    conn.execute(
        "DELETE FROM note_attachments WHERE id = ?",
        params![attachment_id],
    )
    .map_err(|e| format!("Failed to delete attachment: {}", e))?;

    // delete the physical file
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let file_path = app_data_dir.join(file_path);

    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete attachment file: {}", e))?;
    } else {
        // log but don't fail if file doesn't exist
        error!("Attachment file not found at path: {:?}", file_path);
    }

    info!("Deleted attachment with ID: {}", attachment_id);
    Ok(())
}

#[tauri::command]
pub async fn get_attachment_by_id(
    attachment_id: i64,
    db_state: State<'_, DbState>,
) -> Result<NoteAttachment, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let attachment_data = conn
        .query_row(
            "SELECT
                id, note_id, file_name, file_path, file_type, file_size, created_at
             FROM note_attachments
             WHERE id = ?",
            params![attachment_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, String>(6)?,
                ))
            },
        )
        .map_err(|e| format!("Failed to get attachment: {}", e))?;

    let (id, note_id, file_name, file_path, file_type, file_size, created_at) = attachment_data;

    // parse dates
    let created_at = DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Invalid created_at date: {}", e))?
        .with_timezone(&Utc);

    // create NoteAttachment struct
    let attachment = NoteAttachment {
        id,
        note_id,
        file_name,
        file_path,
        file_type,
        file_size,
        created_at,
    };

    Ok(attachment)
}

#[tauri::command]
pub async fn open_attachment(
    attachment_id: i64,
    app_handle: tauri::AppHandle,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // get file path
    let file_path: String = conn
        .query_row(
            "SELECT file_path FROM note_attachments WHERE id = ?",
            params![attachment_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get attachment file path: {}", e))?;

    // get full path
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let full_path = app_data_dir.join(file_path);

    // ensure file exists
    if !full_path.exists() {
        return Err(format!(
            "Attachment file not found at path: {:?}",
            full_path
        ));
    }

    // open the file with the system's default application
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", &full_path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to open attachment: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&full_path)
            .spawn()
            .map_err(|e| format!("Failed to open attachment: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&full_path)
            .spawn()
            .map_err(|e| format!("Failed to open attachment: {}", e))?;
    }

    info!("Opened attachment with ID: {}", attachment_id);
    Ok(())
}
