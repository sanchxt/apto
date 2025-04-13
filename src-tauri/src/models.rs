use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// habit models
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Habit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub frequency: String, // "daily", "weekly", "custom"
    pub start_date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HabitLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub habit_id: i64,
    pub date: NaiveDate,
    pub completed: bool,
}

// streak calculation results
#[derive(Debug, Serialize, Deserialize)]
pub struct HabitStreak {
    pub habit_id: i64,
    pub habit_name: String,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub completion_rate: f64, // Percentage of completion
}

// habit summary with stats
#[derive(Debug, Serialize, Deserialize)]
pub struct HabitWithStats {
    pub habit: Habit,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub completion_rate: f64,
    pub total_completions: i32,
}

// habit template
#[derive(Debug, Serialize, Deserialize)]
pub struct HabitTemplate {
    pub name: String,
    pub description: Option<String>,
    pub frequency: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub goal_count: Option<i32>,
    pub reminder_time: Option<String>,
}
