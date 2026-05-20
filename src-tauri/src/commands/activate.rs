use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn activate(app: AppHandle, path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    
    // 先隐藏窗口（无论启动成功与否都隐藏）
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }

    // 检查路径是否存在
    if !path.exists() {
        return Err(format!("路径不存在: {}", path.display()));
    }
    
    // 检查是否是 exe 文件
    if !path.extension().map_or(false, |e| e.eq_ignore_ascii_case("exe")) {
        return Err(format!("不是可执行文件: {}", path.display()));
    }

    // 启动进程
    let mut cmd = Command::new(&path);
    if let Some(cwd) = path.parent() {
        cmd.current_dir(cwd);
    }
    cmd.spawn().map_err(|e| format!("启动失败: {}", e))?;

    // 更新频次
    let state = app.state::<crate::commands::search::SearchState>();
    if let Some(ref path_str) = path.to_str() {
        state.record_launch(path_str);
    }

    Ok(())
}