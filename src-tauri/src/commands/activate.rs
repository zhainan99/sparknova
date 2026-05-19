use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn activate(app: AppHandle, path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    // 启动进程
    let mut cmd = Command::new(&path);
    if let Some(cwd) = path.parent() {
        cmd.current_dir(cwd);
    }
    cmd.spawn().map_err(|e| e.to_string())?;

    // Layer 3 "用过即学" 逻辑：更新频次缓存
    let state = app.state::<crate::commands::search::SearchState>();
    if let Some(ref path_str) = path.to_str() {
        let mut freq_cache = state.frequency_cache.write().unwrap();
        let key = path_str.to_string();
        let count = freq_cache.get(&key).copied().unwrap_or(0);
        freq_cache.insert(key, count + 1);
    }

    // 隐藏窗口
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }

    Ok(())
}