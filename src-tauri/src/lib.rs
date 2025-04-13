use tauri::{Emitter, Manager, Theme};
use window_vibrancy::{apply_acrylic, clear_acrylic};

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

#[tauri::command]
fn get_system_theme() -> String {
    match dark_light::detect() {
        dark_light::Mode::Dark => "dark".to_string(),
        dark_light::Mode::Light => "light".to_string(),
        dark_light::Mode::Default => "system".to_string(),
    }
}

// Store the current theme preference
static mut CURRENT_THEME: Option<String> = None;

#[tauri::command]
fn set_theme(window: tauri::WebviewWindow, theme: &str) -> Result<(), String> {
    // store the theme preference
    unsafe {
        CURRENT_THEME = Some(theme.to_string());
    }

    match theme {
        "light" => window
            .set_theme(Some(Theme::Light))
            .map_err(|e| e.to_string())?,
        "dark" => window
            .set_theme(Some(Theme::Dark))
            .map_err(|e| e.to_string())?,
        "system" => {
            // for system theme, we need to detect the current OS theme
            let system_theme = match dark_light::detect() {
                dark_light::Mode::Dark => Theme::Dark,
                dark_light::Mode::Light | dark_light::Mode::Default => Theme::Light,
            };
            window
                .set_theme(Some(system_theme))
                .map_err(|e| e.to_string())?;
        }
        _ => return Err("Invalid theme".to_string()),
    }

    // emit theme changed event
    let _ = window.emit("tauri://theme-changed", theme);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            apply_acrylic(&window, Some((100, 100, 100, 50))).expect("Failed to apply acrylic");

            // Set the initial theme to system (which means detect OS theme)
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
            set_theme
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
