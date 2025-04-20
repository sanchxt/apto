use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Habit {
    pub id: i64,                               // unique identifier
    pub name: String,                          // habit name
    pub description: Option<String>,           // optional description
    pub category: Option<String>,              // category for grouping habits
    pub tags: Vec<String>,                     // tags for filtering
    pub frequency: FrequencyPattern,           // how often
    pub target_value: Option<f64>,             // target value for quantifiable habits
    pub target_unit: Option<String>,           // unit for the target
    pub color: Option<String>,                 // UI representation (hex code)
    pub icon: Option<String>,                  // icon identifier for the habit
    pub is_active: bool,                       // whether the habit is currently active
    pub priority: i32,                         // priority level (1-3, with 1 being highest)
    pub start_date: NaiveDate,                 // when to start tracking this habit
    pub end_date: Option<NaiveDate>,           // optional end date for temporary habits
    pub created_at: DateTime<Utc>,             // when the habit was defined
    pub updated_at: DateTime<Utc>,             // when the habit was last updated
    pub reminder_time: Option<String>,         // time of day for reminder (format: "HH:MM")
    pub current_streak: i32,                   // current streak count
    pub longest_streak: i32,                   // longest streak achieved
    pub last_completed: Option<DateTime<Utc>>, // last completion timestamp
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FrequencyPattern {
    Daily,                      // every day
    Weekly { days: Vec<u32> },  // specific days of week (1-7, Monday=1)
    Monthly { days: Vec<u32> }, // specific days of month (1-31)
    Interval { days: u32 },     // every X days (e.g., every 3 days)
    Custom { pattern: String }, // for more complex patterns
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HabitCompletion {
    pub id: i64,
    pub habit_id: i64,               // foreign key linking to the Habit
    pub completed_at: DateTime<Utc>, // when it was marked complete
    pub value: Option<f64>,          // optional value (e.g., 8 glasses, 30 minutes)
    pub notes: Option<String>,       // optional notes for this completion
    pub mood: Option<i32>,           // optional mood rating (1-5)
    pub difficulty: Option<i32>,     // how difficult was it today (1-5)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HabitStats {
    pub habit_id: i64,
    pub completion_rate: f64, // overall completion rate (0.0-1.0)
    pub current_streak: i32,
    pub longest_streak: i32,
    pub total_completions: i32,
    pub last_30_days: HashMap<String, bool>, // last 30 days completion status
    pub average_value: Option<f64>,          // average value if tracking quantities
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HabitTag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

// for reminder functionality
#[derive(Debug, Serialize, Deserialize)]
pub struct HabitReminder {
    pub id: i64,
    pub habit_id: i64,
    pub time: String,   // time of day (HH:MM)
    pub days: Vec<u32>, // days to remind (1-7 for weekly)
    pub is_enabled: bool,
}
