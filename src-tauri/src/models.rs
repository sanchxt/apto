use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Habit {
    pub id: i64,                     // Unique identifier from the database
    pub name: String,                // Name of the habit (e.g., "Drink Water")
    pub description: Option<String>, // Optional details about the habit
    pub frequency: String, // How often? (e.g., "daily", "weekly:Mon,Wed,Fri", "monthly:15")
    pub created_at: DateTime<Utc>, // When the habit was defined
                           // pub goal: Option<String>, // e.g., "8 glasses"
                           // pub color: Option<String>, // For UI representation
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HabitCompletion {
    pub id: i64,
    pub habit_id: i64,               // Foreign key linking to the Habit
    pub completed_at: DateTime<Utc>, // When it was marked complete
    pub notes: Option<String>,       // Optional notes for this completion
}
