use sqlx::{Pool, Sqlite};
use tauri::Manager;
use tauri::{path, State};
use window_vibrancy::{apply_acrylic, clear_acrylic};

mod habits;
mod models;
mod templates;

use habits::{
    calculate_habit_streak, create_default_habit_templates, create_habit,
    create_habit_from_template, delete_habit, get_all_habits, get_all_habits_with_stats, get_habit,
    get_habit_logs, get_habit_with_stats, log_habit, update_habit,
};
use templates::{
    delete_template_file, init_template_directories, list_template_files, read_template_file,
    save_template_file,
};

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("testt {}", name)
}

#[tauri::command]
fn set_acrylic_effect(window: tauri::WebviewWindow, enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if enable {
            apply_acrylic(&window, Some((100, 100, 100, 50)))
                .map_err(|e| format!("Failed to apply acrylic: {}", e))?;
        } else {
            clear_acrylic(&window).map_err(|e| format!("Failed to clear acrylic: {}", e))?;
        }
    }
    Ok(())
}

async fn initialize_database(pool: &Pool<Sqlite>) -> Result<(), String> {
    println!("Starting database initialization...");

    // Check if tables exist first
    let tables_to_check = vec![
        "habits", "habit_logs", "tasks", "notes", "goals",
        "kanban_boards", "kanban_columns", "kanban_cards",
        "pomodoro_sessions", "pomodoro_configs"
    ];

    for table in &tables_to_check {
        let exists = sqlx::query(&format!(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='table' AND name='{}';
            "#,
            table
        ))
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check if {} table exists: {}", table, e))?;

        if exists.is_none() {
            println!("Table '{}' does not exist", table);
        } else {
            println!("Table '{}' already exists", table);
        }
    }

    // Create habits and habit logs tables
    println!("Creating habits table...");
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

    println!("Creating habit_logs table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS habit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            habit_id INTEGER NOT NULL,
            date DATE NOT NULL,
            completed BOOLEAN NOT NULL,
            FOREIGN KEY (habit_id) REFERENCES habits (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create habit_logs table: {}", e))?;

    // Tasks (for to-do lists)
    println!("Creating tasks table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            due_date DATE,
            priority TEXT,
            status TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create tasks table: {}", e))?;

    // Notes (for journals/notes)
    println!("Creating notes table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create notes table: {}", e))?;

    // Goals (for goal tracker)
    println!("Creating goals table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            target_date DATE,
            progress INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create goals table: {}", e))?;

    // Kanban boards, columns, and cards
    println!("Creating kanban tables...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS kanban_boards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create kanban_boards table: {}", e))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS kanban_columns (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            board_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            FOREIGN KEY (board_id) REFERENCES kanban_boards (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create kanban_columns table: {}", e))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS kanban_cards (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            board_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            column_id INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (board_id) REFERENCES kanban_boards (id),
            FOREIGN KEY (column_id) REFERENCES kanban_columns (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create kanban_cards table: {}", e))?;

    // Pomodoro sessions
    println!("Creating pomodoro tables...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pomodoro_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            duration_minutes INTEGER NOT NULL,
            break_duration_minutes INTEGER NOT NULL,
            completed BOOLEAN DEFAULT FALSE,
            task_id INTEGER,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES tasks (id)
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create pomodoro_sessions table: {}", e))?;

    // Pomodoro configurations (for custom timer settings)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pomodoro_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            work_duration_minutes INTEGER NOT NULL,
            short_break_minutes INTEGER NOT NULL,
            long_break_minutes INTEGER NOT NULL,
            cycles_before_long_break INTEGER NOT NULL,
            is_default BOOLEAN DEFAULT FALSE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create pomodoro_configs table: {}", e))?;

    // Check if tables were created successfully
    for table in &tables_to_check {
        let exists = sqlx::query(&format!(
            r#"
            SELECT name FROM sqlite_master
            WHERE type='table' AND name='{}';
            "#,
            table
        ))
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("Failed to check if {} table exists after creation: {}", table, e))?;

        if exists.is_none() {
            return Err(format!("Failed to create table '{}' during initialization", table));
        } else {
            println!("Confirmed table '{}' was created successfully", table);
        }
    }

    println!("Database initialization completed successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // tokio runtime for blocking on async operations
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;

            rt.block_on(async {
                // handle acrylic effect
                #[cfg(target_os = "windows")]
                {
                    let window = app.get_webview_window("main").unwrap();
                    apply_acrylic(&window, Some((100, 100, 100, 50)))
                        .expect("Failed to apply acrylic");
                }

                // init db connection
                // get app data directory for database storage
                let app_handle = app.handle();
                let app_data_dir = app_handle
                    .path()
                    .app_data_dir()
                    .map_err(|_| "Failed to resolve app data directory".to_string())?;

                // Create the app_data_dir if it doesn't exist
                if !app_data_dir.exists() {
                    println!("App data directory does not exist, creating it at: {}", app_data_dir.display());
                    std::fs::create_dir_all(&app_data_dir)
                        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
                    println!("App data directory created successfully");
                } else {
                    println!("App data directory already exists at: {}", app_data_dir.display());
                }

                let db_path = app_data_dir.join("apto.db");
                println!("Database path: {}", db_path.display());

                let database_url = format!("sqlite:{}", db_path.display());
                println!("Database URL: {}", database_url);

                println!("Connecting to SQLite database...");
                let pool = sqlx::Pool::<Sqlite>::connect(&database_url)
                    .await
                    .map_err(|e| format!("Failed to connect to database: {}", e))?;
                println!("Successfully connected to database");

                // Initialize database tables with extensive logging
                println!("Starting database initialization...");
                match initialize_database(&pool).await {
                    Ok(_) => println!("Database initialization successful"),
                    Err(e) => {
                        println!("Database initialization failed: {}", e);
                        return Err(e);
                    }
                }

                // Manage state with the database pool
                println!("Setting up app state...");
                app.manage(AppState { db: pool.clone() });

                // init template directories
                println!("Initializing template directories...");
                let app_data_dir_resolver = app.app_data_dir_resolver();
                init_template_directories(app_data_dir_resolver.clone())
                    .map_err(|e| format!("Failed to initialize template directories: {}", e))?;

                // create default habit templates
                println!("Creating default habit templates...");
                create_default_habit_templates(app_data_dir_resolver)
                    .map_err(|e| format!("Failed to create default habit templates: {}", e))?;

                println!("Application setup completed successfully");
                Ok::<(), String>(())
            })?;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(path::AppDataDirPathResolver::new())
        .invoke_handler(tauri::generate_handler![
            // UI commands
            greet,
            set_acrylic_effect,
            // template management commands
            init_template_directories,
            save_template_file,
            read_template_file,
            list_template_files,
            delete_template_file,
            // habit management commands
            create_habit,
            get_habit,
            get_all_habits,
            update_habit,
            delete_habit,
            log_habit,
            get_habit_logs,
            calculate_habit_streak,
            get_habit_with_stats,
            get_all_habits_with_stats,
            create_default_habit_templates,
            create_habit_from_template
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
