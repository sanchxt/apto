use std::sync::Mutex;
use tauri::{Emitter, Manager, Theme};
use window_vibrancy::apply_acrylic;

mod models;

mod db;
use db::init::*;

// ui imports
mod ui;
use ui::theme::*;

mod features;

// habits imports
use features::habits::commands::crud::{
    add_habit, delete_habit, get_habit_by_id, get_habits, toggle_habit_active, update_habit,
};
use features::habits::commands::habit_completion::{
    delete_habit_completion, get_habit_completions, update_habit_completion,
};
use features::habits::commands::reminders::{
    create_habit_reminder, delete_habit_reminder, get_habit_reminders, toggle_reminder,
    update_habit_reminder,
};
use features::habits::commands::stats::get_habit_stats;
use features::habits::commands::streaks::{add_habit_completion, update_habit_streaks};
use features::habits::commands::tag::{create_tag, delete_tag, get_all_tags, update_tag};

// for testing...
#[tauri::command]
fn greet(name: &str) -> String {
    format!("testt {}", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let db_conn =
                initialize_database(&app.handle()).expect("Failed to initialize database");
            app.manage(DbState(Mutex::new(db_conn)));

            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            apply_acrylic(&window, Some((100, 100, 100, 50))).expect("Failed to apply acrylic");

            // initial theme to system
            let system_theme = match dark_light::detect() {
                dark_light::Mode::Dark => Theme::Dark,
                dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
            };

            window
                .set_theme(Some(system_theme))
                .expect("Failed to set theme");

            // emit initial theme event with the actual theme name
            let system_theme_name = match dark_light::detect() {
                dark_light::Mode::Dark => "dark",
                dark_light::Mode::Light | dark_light::Mode::Default => "light",
            };

            // emit initial theme event
            let _ = window.emit("tauri://theme-changed", system_theme_name);

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            set_acrylic_effect,
            get_system_theme,
            set_theme,
            add_habit,
            get_habits,
            get_habit_by_id,
            update_habit,
            delete_habit,
            toggle_habit_active,
            add_habit_completion,
            // new tag functions
            get_all_tags,
            create_tag,
            update_tag,
            delete_tag,
            // new completion functions
            get_habit_completions,
            update_habit_completion,
            delete_habit_completion,
            // new reminder functions
            get_habit_reminders,
            create_habit_reminder,
            update_habit_reminder,
            delete_habit_reminder,
            toggle_reminder,
            // stats function
            get_habit_stats,
            // new streak update function
            update_habit_streaks
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
