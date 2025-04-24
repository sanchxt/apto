use crate::db::init::DbState;
use chrono::Utc;
use rusqlite::params;
use serde_json;
use std::collections::HashMap;
use tauri::State;

use crate::features::habits::models::HabitStats;

#[tauri::command]
pub async fn get_habit_stats(
    habit_id: i64,
    db_state: State<'_, DbState>,
) -> Result<HabitStats, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    // Get total completions
    let total_completions: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM habit_completions WHERE habit_id = ?",
            params![habit_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to get completion count: {}", e))?;

    // Get current and longest streaks from the habit table
    let (current_streak, longest_streak): (i32, i32) = conn
        .query_row(
            "SELECT current_streak, longest_streak FROM habits WHERE id = ?",
            params![habit_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Failed to get streak data: {}", e))?;

    // Calculate average value if applicable
    let average_value: Option<f64> = conn
        .query_row(
            "SELECT AVG(value) FROM habit_completions WHERE habit_id = ? AND value IS NOT NULL",
            params![habit_id],
            |row| row.get(0),
        )
        .ok();

    // Get frequency data for the habit to calculate completion rate
    let (frequency_type, frequency_data): (String, String) = conn
        .query_row(
            "SELECT frequency_type, frequency_data FROM habits WHERE id = ?",
            params![habit_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Failed to get frequency data: {}", e))?;

    // Get last 30 days completion status
    let mut last_30_days = HashMap::new();
    let today = Utc::now().date_naive();

    let mut stmt = conn
        .prepare(
            "SELECT strftime('%Y-%m-%d', completed_at) as completion_date
             FROM habit_completions
             WHERE habit_id = ?
             AND completed_at >= datetime('now', '-30 days')
             GROUP BY completion_date",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let dates_iter = stmt
        .query_map(params![habit_id], |row| {
            let date: String = row.get(0)?;
            Ok(date)
        })
        .map_err(|e| format!("Failed to query completion dates: {}", e))?;

    // Initialize all 30 days as false first
    for i in 0..30 {
        let date = today.checked_sub_days(chrono::Days::new(i as u64)).unwrap();
        last_30_days.insert(date.format("%Y-%m-%d").to_string(), false);
    }

    // Mark completed days as true
    for date_result in dates_iter {
        let date = date_result.map_err(|e| format!("Failed to process date: {}", e))?;
        last_30_days.insert(date, true);
    }

    // Calculate completion rate based on frequency and completed days
    let expected_completions = match frequency_type.as_str() {
        "daily" => 30, // Daily for 30 days
        "weekly" => {
            let days: Vec<u32> = serde_json::from_str(&frequency_data)
                .map_err(|e| format!("Failed to parse frequency data: {}", e))?;
            (30 / 7) * days.len() as i32 + 1 // Approx. number of occurrences in 30 days
        }
        "monthly" => 1, // Only happens once a month
        "interval" => {
            let days: u32 = serde_json::from_str(&frequency_data)
                .map_err(|e| format!("Failed to parse frequency data: {}", e))?;
            30 / days as i32 // Approx. number of occurrences in 30 days
        }
        _ => 30, // Default to daily
    };

    let completion_rate = if expected_completions > 0 {
        total_completions as f64 / expected_completions as f64
    } else {
        0.0
    };

    // Clamp to 0.0-1.0 range
    let completion_rate = completion_rate.min(1.0).max(0.0);

    Ok(HabitStats {
        habit_id,
        completion_rate,
        current_streak,
        longest_streak,
        total_completions,
        last_30_days,
        average_value,
    })
}
