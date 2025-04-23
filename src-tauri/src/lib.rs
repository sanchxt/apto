use std::sync::Mutex;
use tauri::{Emitter, Manager, Theme};
use window_vibrancy::apply_acrylic;

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

// notes imports
use features::notes::commands::attachments::{
    add_attachment, delete_attachment, get_attachment_by_id, get_note_attachments, open_attachment,
};
use features::notes::commands::crud::{
    create_note, delete_note, get_note_by_id, get_notes, get_notes_by_folder, search_notes,
    toggle_note_archive, toggle_note_pin, update_note,
};
use features::notes::commands::folders::{
    create_folder, delete_folder, get_folder_by_id, get_folders, get_subfolders, update_folder,
};
use features::notes::commands::revisions::{
    clean_old_revisions, create_revision, delete_revision, get_note_revisions, get_revision_by_id,
    restore_revision,
};
use features::notes::commands::tags::{
    create_note_tag, delete_note_tag, get_all_note_tags, get_notes_by_tag, update_note_tag,
};

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
            // ui commands
            greet,
            set_acrylic_effect,
            get_system_theme,
            set_theme,
            // habit copmmands
            add_habit,
            get_habits,
            get_habit_by_id,
            update_habit,
            delete_habit,
            toggle_habit_active,
            add_habit_completion,
            // habit tag functions
            get_all_tags,
            create_tag,
            update_tag,
            delete_tag,
            // habit completion functions
            get_habit_completions,
            update_habit_completion,
            delete_habit_completion,
            // habit reminder functions
            get_habit_reminders,
            create_habit_reminder,
            update_habit_reminder,
            delete_habit_reminder,
            toggle_reminder,
            // habit stats function
            get_habit_stats,
            // habit streak update function
            update_habit_streaks,
            // note commands
            create_note,
            get_notes,
            get_note_by_id,
            update_note,
            delete_note,
            toggle_note_pin,
            toggle_note_archive,
            get_notes_by_folder,
            search_notes,
            // note folder commands
            create_folder,
            get_folders,
            get_folder_by_id,
            update_folder,
            delete_folder,
            get_subfolders,
            // note tag commands
            create_note_tag,
            get_all_note_tags,
            update_note_tag,
            delete_note_tag,
            get_notes_by_tag,
            // note revision commands
            get_note_revisions,
            create_revision,
            restore_revision,
            delete_revision,
            get_revision_by_id,
            clean_old_revisions,
            // note attachment commands
            add_attachment,
            get_note_attachments,
            delete_attachment,
            get_attachment_by_id,
            open_attachment
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
