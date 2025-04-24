use tauri::{Emitter, Theme};
use window_vibrancy::{apply_acrylic, clear_acrylic};

#[tauri::command]
pub fn set_acrylic_effect(window: tauri::WebviewWindow, enable: bool) -> Result<(), String> {
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
pub fn get_system_theme() -> String {
    match dark_light::detect() {
        dark_light::Mode::Dark => "dark".to_string(),
        dark_light::Mode::Light => "light".to_string(),
        dark_light::Mode::Default => "system".to_string(),
    }
}

// store the current theme preference
static mut CURRENT_THEME: Option<String> = None;

#[tauri::command]
pub fn set_theme(window: tauri::WebviewWindow, theme: &str) -> Result<(), String> {
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
            // detect the current OS theme
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
