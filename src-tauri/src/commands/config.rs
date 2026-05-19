use tauri::AppHandle;
use crate::config::UserConfig;

#[tauri::command]
pub async fn get_config(_app: AppHandle) -> Result<serde_json::Value, String> {
    let config = UserConfig::load().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "scan_dirs": config.scan_dirs,
        "theme_mode": config.theme_mode,
        "show_main_on_start": config.show_main_on_start,
        "show_notes_on_start": config.show_notes_on_start,
    }))
}

#[tauri::command]
pub async fn save_config(_app: AppHandle, config: serde_json::Value) -> Result<(), String> {
    let mut user_config = UserConfig::load().map_err(|e| e.to_string())?;

    if let Some(scan_dirs) = config.get("scan_dirs").and_then(|v| v.as_array()) {
        user_config.scan_dirs = scan_dirs
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    if let Some(theme) = config.get("theme_mode").and_then(|v| v.as_str()) {
        user_config.theme_mode = theme.to_string();
    }
    if let Some(show) = config.get("show_main_on_start").and_then(|v| v.as_bool()) {
        user_config.show_main_on_start = show;
    }
    if let Some(show) = config.get("show_notes_on_start").and_then(|v| v.as_bool()) {
        user_config.show_notes_on_start = show;
    }

    user_config.save().map_err(|e| e.to_string())
}