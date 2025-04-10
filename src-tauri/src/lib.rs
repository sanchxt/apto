use tauri::Manager;
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
            clear_acrylic(&window)
                .map_err(|e| format!("Failed to clear acrylic: {}", e))?;
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            #[cfg(target_os = "windows")]
            apply_acrylic(&window, Some((100, 100, 100, 50))).expect("Failed to apply acrylic");
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, set_acrylic_effect])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
