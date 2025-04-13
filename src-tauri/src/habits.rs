use crate::models::{Habit, HabitLog, HabitStreak, HabitWithStats};
use crate::templates::{TemplateCategory, TemplateManager};
use chrono::{Duration, Local, NaiveDate};
use serde_json::{json, Value};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::path::AppDataDirPathResolver;
use tauri::State;

// habits CRUD

#[tauri::command]
pub async fn create_habit(habit: Habit, db_pool: State<'_, Pool<Sqlite>>) -> Result<i64, String> {
    // Access the pool correctly - using .inner() to get the Pool<Sqlite> from State
    let pool = db_pool.inner();

    // First check if the habits table exists
    let table_exists = sqlx::query(
        r#"
        SELECT name FROM sqlite_master
        WHERE type='table' AND name='habits';
        "#
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to check if habits table exists: {}", e))?;

    // If table doesn't exist, create it
    if table_exists.is_none() {
        println!("Habits table does not exist, creating it now");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS habits (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                frequency TEXT NOT NULL,
                start_date DATE NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to create habits table: {}", e))?;

        println!("Habits table created successfully");
    } else {
        println!("Habits table already exists");
    }

    // Now insert the habit
    println!("Inserting habit: {}", habit.name);

    let result = sqlx::query!(
        r#"
        INSERT INTO habits (name, description, frequency, start_date)
        VALUES (?, ?, ?, ?)
        "#,
        habit.name,
        habit.description,
        habit.frequency,
        habit.start_date,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create habit: {}", e))?;

    println!("Habit inserted successfully");

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn get_habit(id: i64, db_pool: State<'_, Pool<Sqlite>>) -> Result<Habit, String> {
    let habit = sqlx::query_as!(
        Habit,
        r#"
        SELECT id, name, description, frequency, start_date, created_at, updated_at
        FROM habits
        WHERE id = ?
        "#,
        id
    )
    .fetch_one(&**db_pool)
    .await
    .map_err(|e| format!("Failed to get habit: {}", e))?;

    Ok(habit)
}

#[tauri::command]
pub async fn get_all_habits(db_pool: State<'_, Pool<Sqlite>>) -> Result<Vec<Habit>, String> {
    let habits = sqlx::query_as!(
        Habit,
        r#"
        SELECT id, name, description, frequency, start_date, created_at, updated_at
        FROM habits
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&**db_pool)
    .await
    .map_err(|e| format!("Failed to get habits: {}", e))?;

    Ok(habits)
}

#[tauri::command]
pub async fn update_habit(
    id: i64,
    habit: Habit,
    db_pool: State<'_, Pool<Sqlite>>,
) -> Result<(), String> {
    sqlx::query!(
        r#"
        UPDATE habits
        SET name = ?, description = ?, frequency = ?, start_date = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
        habit.name,
        habit.description,
        habit.frequency,
        habit.start_date,
        id
    )
    .execute(&**db_pool)
    .await
    .map_err(|e| format!("Failed to update habit: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_habit(id: i64, db_pool: State<'_, Pool<Sqlite>>) -> Result<(), String> {
    // start transaction
    let mut tx = db_pool
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // delete all habit logs for this habit
    sqlx::query!(
        r#"
        DELETE FROM habit_logs
        WHERE habit_id = ?
        "#,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to delete habit logs: {}", e))?;

    // delete habit
    sqlx::query!(
        r#"
        DELETE FROM habits
        WHERE id = ?
        "#,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to delete habit: {}", e))?;

    // commit transaction
    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(())
}

// habit logs

#[tauri::command]
pub async fn log_habit(
    habit_id: i64,
    date: NaiveDate,
    completed: bool,
    db_pool: State<'_, Pool<Sqlite>>,
) -> Result<i64, String> {
    // check if log already exists for this date
    let existing_log = sqlx::query!(
        r#"
        SELECT id FROM habit_logs
        WHERE habit_id = ? AND date = ?
        "#,
        habit_id,
        date
    )
    .fetch_optional(&**db_pool)
    .await
    .map_err(|e| format!("Failed to check existing log: {}", e))?;

    let log_id = if let Some(log) = existing_log {
        // update existing log
        sqlx::query!(
            r#"
            UPDATE habit_logs
            SET completed = ?
            WHERE id = ?
            "#,
            completed,
            log.id
        )
        .execute(&**db_pool)
        .await
        .map_err(|e| format!("Failed to update habit log: {}", e))?;

        log.id
    } else {
        // create new log
        let result = sqlx::query!(
            r#"
            INSERT INTO habit_logs (habit_id, date, completed)
            VALUES (?, ?, ?)
            "#,
            habit_id,
            date,
            completed
        )
        .execute(&**db_pool)
        .await
        .map_err(|e| format!("Failed to create habit log: {}", e))?;

        result.last_insert_rowid()
    };

    Ok(log_id)
}

#[tauri::command]
pub async fn get_habit_logs(
    habit_id: i64,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    db_pool: State<'_, Pool<Sqlite>>,
) -> Result<Vec<HabitLog>, String> {
    let today = Local::now().date_naive();
    let start = start_date.unwrap_or_else(|| today - Duration::days(30));
    let end = end_date.unwrap_or_else(|| today);

    let logs = sqlx::query_as!(
        HabitLog,
        r#"
        SELECT id, habit_id, date, completed
        FROM habit_logs
        WHERE habit_id = ? AND date BETWEEN ? AND ?
        ORDER BY date DESC
        "#,
        habit_id,
        start,
        end
    )
    .fetch_all(&**db_pool)
    .await
    .map_err(|e| format!("Failed to get habit logs: {}", e))?;

    Ok(logs)
}

// business logic for habits

#[tauri::command]
pub async fn calculate_habit_streak(
    habit_id: i64,
    db_pool: State<'_, Pool<Sqlite>>,
) -> Result<HabitStreak, String> {
    // get the habit details
    let habit = get_habit(habit_id, db_pool.clone()).await?;

    // get all logs for this habit
    let logs = sqlx::query!(
        r#"
        SELECT date, completed
        FROM habit_logs
        WHERE habit_id = ?
        ORDER BY date DESC
        "#,
        habit_id
    )
    .fetch_all(&**db_pool)
    .await
    .map_err(|e| format!("Failed to get habit logs: {}", e))?;

    // map of date to completion status
    let mut completion_map: HashMap<NaiveDate, bool> = HashMap::new();
    for log in logs.iter() {
        completion_map.insert(log.date, log.completed);
    }

    // calculate current streak
    let today = Local::now().date_naive();
    let mut current_streak = 0;
    let mut current_date = today;

    // handle different frequencies
    let date_step = match habit.frequency.as_str() {
        "daily" => 1,
        "weekly" => 7,
        _ => 1, // default to daily for custom or other frequencies
    };

    loop {
        if let Some(&completed) = completion_map.get(&current_date) {
            if completed {
                current_streak += 1;
                current_date = current_date - Duration::days(date_step);
            } else {
                break;
            }
        } else {
            // if no log for today yet, check if it's after start_date
            if current_date >= habit.start_date {
                // missing log breaks streak
                break;
            } else {
                // if we're before start_date, we're done
                break;
            }
        }
    }

    // calculate longest streak
    let mut longest_streak = 0;
    let mut current_longest = 0;

    // sort dates chronologically
    let mut dates: Vec<NaiveDate> = completion_map.keys().cloned().collect();
    dates.sort();

    for i in 0..dates.len() {
        if *completion_map.get(&dates[i]).unwrap_or(&false) {
            // check if this is consecutive based on frequency
            if i > 0 {
                let diff = (dates[i] - dates[i - 1]).num_days();
                if diff == date_step as i64 {
                    current_longest += 1;
                } else {
                    current_longest = 1;
                }
            } else {
                current_longest = 1;
            }

            if current_longest > longest_streak {
                longest_streak = current_longest;
            }
        } else {
            current_longest = 0;
        }
    }

    // calculate completion rate
    let total_days = match habit.frequency.as_str() {
        "daily" => (today - habit.start_date).num_days() + 1,
        "weekly" => ((today - habit.start_date).num_days() / 7) + 1,
        _ => (today - habit.start_date).num_days() + 1,
    };

    let total_completions = logs.iter().filter(|log| log.completed).count() as i64;
    let completion_rate = if total_days > 0 {
        (total_completions as f64 / total_days as f64) * 100.0
    } else {
        0.0
    };

    Ok(HabitStreak {
        habit_id,
        habit_name: habit.name,
        current_streak,
        longest_streak,
        completion_rate,
    })
}

#[tauri::command]
pub async fn get_habit_with_stats(
    habit_id: i64,
    db_pool: State<'_, Pool<Sqlite>>,
) -> Result<HabitWithStats, String> {
    // get the habit
    let habit = get_habit(habit_id, db_pool.clone()).await?;

    // get the streak info
    let streak = calculate_habit_streak(habit_id, db_pool.clone()).await?;

    // count total completions
    let total_completions = sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM habit_logs
        WHERE habit_id = ? AND completed = true
        "#,
        habit_id
    )
    .fetch_one(&**db_pool)
    .await
    .map_err(|e| format!("Failed to count completions: {}", e))?
    .count as i32;

    Ok(HabitWithStats {
        habit,
        current_streak: streak.current_streak,
        longest_streak: streak.longest_streak,
        completion_rate: streak.completion_rate,
        total_completions,
    })
}

#[tauri::command]
pub async fn get_all_habits_with_stats(
    db_pool: State<'_, Pool<Sqlite>>,
) -> Result<Vec<HabitWithStats>, String> {
    // get all habits
    let habits = get_all_habits(db_pool.clone()).await?;

    // calculate stats for each habit
    let mut habits_with_stats = Vec::new();
    for habit in habits {
        if let Some(id) = habit.id {
            let habit_stats = get_habit_with_stats(id, db_pool.clone()).await?;
            habits_with_stats.push(habit_stats);
        }
    }

    Ok(habits_with_stats)
}

// template related functions
#[tauri::command]
pub fn create_default_habit_templates(
    app_data_dir_resolver: tauri::State<'_, AppDataDirPathResolver>,
) -> Result<(), String> {
    let app_data_dir = app_data_dir_resolver
        .resolve()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let template_manager = TemplateManager::new(&app_data_dir);

    // create default templates
    let templates = vec![
        json!({
            "name": "Daily Exercise",
            "description": "Exercise for at least 30 minutes",
            "frequency": "daily",
            "icon": "dumbbell",
            "color": "#e74c3c",
            "goal_count": 1,
            "reminder_time": "08:00"
        }),
        json!({
            "name": "Read Book",
            "description": "Read for at least 15 minutes",
            "frequency": "daily",
            "icon": "book",
            "color": "#3498db",
            "goal_count": 1,
            "reminder_time": "21:00"
        }),
        json!({
            "name": "Weekly Review",
            "description": "Review goals and plan the next week",
            "frequency": "weekly",
            "icon": "calendar",
            "color": "#9b59b6",
            "goal_count": 1,
            "reminder_time": "18:00"
        }),
        json!({
            "name": "Meditation",
            "description": "Meditate for at least 10 minutes",
            "frequency": "daily",
            "icon": "peace",
            "color": "#2ecc71",
            "goal_count": 1,
            "reminder_time": "07:00"
        }),
        json!({
            "name": "Drink Water",
            "description": "Drink 8 glasses of water",
            "frequency": "daily",
            "icon": "droplet",
            "color": "#3498db",
            "goal_count": 8,
            "reminder_time": null
        }),
    ];

    // save each template
    for (i, template) in templates.iter().enumerate() {
        let name = format!("default_{}", i + 1);
        template_manager
            .save_template(&TemplateCategory::Daily, &name, &template.to_string())
            .map_err(|e| format!("Failed to save template: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn create_habit_from_template(
    template_name: String,
    start_date: NaiveDate,
    app_data_dir_resolver: tauri::State<'_, AppDataDirPathResolver>,
    db_pool: State<'_, Pool<Sqlite>>,
) -> Result<i64, String> {
    let app_data_dir = app_data_dir_resolver
        .resolve()
        .map_err(|e| format!("Failed to resolve app data directory: {}", e))?;

    let template_manager = TemplateManager::new(&app_data_dir);

    // try to find the template in different categories
    let categories = vec![
        TemplateCategory::Daily,
        TemplateCategory::Weekly,
        TemplateCategory::Custom,
    ];

    let mut template_content = None;

    for category in categories {
        match template_manager.read_template(&category, &template_name) {
            Ok(content) => {
                template_content = Some(content);
                break;
            }
            Err(_) => continue,
        }
    }

    let template_str =
        template_content.ok_or_else(|| format!("Template not found: {}", template_name))?;

    // parse the template
    let template_value: Value = serde_json::from_str(&template_str)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    // create habit from template
    let habit = Habit {
        id: None,
        name: template_value["name"]
            .as_str()
            .unwrap_or("New Habit")
            .to_string(),
        description: template_value["description"]
            .as_str()
            .map(|s| s.to_string()),
        frequency: template_value["frequency"]
            .as_str()
            .unwrap_or("daily")
            .to_string(),
        start_date,
        created_at: None,
        updated_at: None,
    };

    // save the habit
    create_habit(habit, db_pool).await
}
