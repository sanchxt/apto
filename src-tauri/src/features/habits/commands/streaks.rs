use crate::db::init::DbState;
use crate::features::habits::utils::deserialize_frequency;
use crate::features::habits::utils::streaks::{breaks_streak, is_habit_due};
use chrono::{DateTime, Utc};
use log::info;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub async fn update_habit_streaks(db_state: State<'_, DbState>) -> Result<(), String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let today = Utc::now().date_naive();

    // get all active habits
    let mut habit_stmt = conn
        .prepare(
            "SELECT id, frequency_type, frequency_data, last_completed, current_streak
             FROM habits WHERE is_active = 1",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let habits_iter = habit_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i32>(4)?,
            ))
        })
        .map_err(|e| format!("Failed to query habits: {}", e))?;

    for habit_result in habits_iter {
        let (id, frequency_type, frequency_data, last_completed_str, current_streak) =
            habit_result.map_err(|e| format!("Failed to process habit: {}", e))?;

        // parse frequency
        let frequency = deserialize_frequency(&frequency_type, &frequency_data)
            .map_err(|e| format!("Failed to deserialize frequency: {}", e))?;

        // process last_completed
        let last_completed = match last_completed_str {
            Some(date) => Some(
                DateTime::parse_from_rfc3339(&date)
                    .map_err(|e| format!("Invalid last_completed date: {}", e))?
                    .with_timezone(&Utc),
            ),
            None => None,
        };

        // check if streak is broken
        let mut streak_broken = false;
        if let Some(last) = last_completed {
            let last_date = last.date_naive();

            if today > last_date {
                // check if habit was due on any day since last completion
                let mut check_date = last_date;
                while check_date < today {
                    check_date = check_date.succ_opt().unwrap();
                    if is_habit_due(&frequency, check_date, Some(last)) && check_date < today {
                        streak_broken = true;
                        break;
                    }
                }
            }
        }

        // Reset streak if broken
        if streak_broken && current_streak > 0 {
            conn.execute(
                "UPDATE habits SET current_streak = 0 WHERE id = ?",
                params![id],
            )
            .map_err(|e| format!("Failed to update streak: {}", e))?;

            info!("Reset streak for habit ID {} due to missed days", id);
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn add_habit_completion(
    habit_id: i64,
    value: Option<f64>,
    notes: Option<String>,
    mood: Option<i32>,
    difficulty: Option<i32>,
    db_state: State<'_, DbState>,
) -> Result<i64, String> {
    let conn = db_state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock DB mutex: {}", e))?;

    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let today = now.date_naive();

    // get current habit info to calculate streaks
    let (frequency_type, frequency_data, last_completed, current_streak, longest_streak): (
        String,
        String,
        Option<String>,
        i32,
        i32,
    ) = conn
        .query_row(
            "SELECT frequency_type, frequency_data, last_completed, current_streak, longest_streak
             FROM habits WHERE id = ?",
            params![habit_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| format!("Failed to get habit info: {}", e))?;

    // parse frequency
    let frequency = deserialize_frequency(&frequency_type, &frequency_data)
        .map_err(|e| format!("Failed to deserialize frequency: {}", e))?;

    // parse last completed
    let last_completed = match last_completed {
        Some(date) => Some(
            DateTime::parse_from_rfc3339(&date)
                .map_err(|e| format!("Invalid last_completed date: {}", e))?
                .with_timezone(&Utc),
        ),
        None => None,
    };

    // determine if this completion continues or resets streak
    let new_current_streak = match last_completed {
        Some(last) => {
            let last_date = last.date_naive();

            // skip duplicate completions on the same day
            if last_date == today {
                current_streak
            } else if breaks_streak(&frequency, last, today) {
                // streak broken, reset to 1
                1
            } else {
                // streak continues
                current_streak + 1
            }
        }
        None => 1, // first completion, streak of 1
    };

    // calculate new longest streak
    let new_longest_streak = std::cmp::max(longest_streak, new_current_streak);

    // insert the completion
    conn.execute(
        "INSERT INTO habit_completions (
            habit_id, completed_at, value, notes, mood, difficulty
        ) VALUES (?, ?, ?, ?, ?, ?)",
        params![habit_id, now_str, value, notes, mood, difficulty],
    )
    .map_err(|e| format!("Failed to add completion: {}", e))?;

    let completion_id = conn.last_insert_rowid();

    // update the habit's last_completed date and streak info
    conn.execute(
        "UPDATE habits SET
            last_completed = ?,
            current_streak = ?,
            longest_streak = ?
        WHERE id = ?",
        params![now_str, new_current_streak, new_longest_streak, habit_id],
    )
    .map_err(|e| format!("Failed to update habit: {}", e))?;

    info!(
        "Added completion for habit ID {} with completion ID: {}. Streak: {}",
        habit_id, completion_id, new_current_streak
    );
    Ok(completion_id)
}
