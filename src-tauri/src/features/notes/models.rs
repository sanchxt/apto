use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,                   // unique identifier
    pub title: String,             // note title
    pub content: String,           // note content
    pub folder_id: Option<i64>,    // optional folder ID (can be null if in root)
    pub tags: Vec<String>,         // tags for filtering and organization
    pub is_pinned: bool,           // whether the note is pinned
    pub is_archived: bool,         // whether the note is archived
    pub color: Option<String>,     // UI representation (hex code)
    pub created_at: DateTime<Utc>, // when the note was created
    pub updated_at: DateTime<Utc>, // when the note was last updated
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteFolder {
    pub id: i64,                   // unique identifier
    pub name: String,              // folder name
    pub parent_id: Option<i64>,    // parent folder ID (null if root folder)
    pub color: Option<String>,     // UI representation (hex code)
    pub created_at: DateTime<Utc>, // when the folder was created
    pub updated_at: DateTime<Utc>, // when the folder was last updated
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoteTag {
    pub id: i64,               // unique identifier
    pub name: String,          // tag name
    pub color: Option<String>, // UI representation (hex code)
}

// For tracking revision history of notes
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteRevision {
    pub id: i64,                   // unique identifier
    pub note_id: i64,              // foreign key linking to the Note
    pub content: String,           // previous content
    pub created_at: DateTime<Utc>, // when this revision was created
}

// For attachments within notes
#[derive(Debug, Serialize, Deserialize)]
pub struct NoteAttachment {
    pub id: i64,                   // unique identifier
    pub note_id: i64,              // foreign key linking to the Note
    pub file_name: String,         // original file name
    pub file_path: String,         // path to the stored file
    pub file_type: String,         // MIME type or file extension
    pub file_size: i64,            // size in bytes
    pub created_at: DateTime<Utc>, // when the attachment was added
}
