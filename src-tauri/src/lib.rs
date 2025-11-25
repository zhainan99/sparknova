//! SparkNova 窗口控制模块
//!
//! 功能：
//! - 全局快捷键（仅处理按下事件）切换主窗口显隐
//! - 失焦后按保护期自动隐藏
//! - 提供命令给前端（隐藏/切换）
//!
//! 流程图：
//!
//! ```text
//! [快捷键 Pressed]
//!   -> toggle_main_window()
//!      -> if visible => hide
//!      -> else => show + focus + center + mark last_show_time
//!
//! [窗口事件: Focused(false)]
//!   -> if now - last_show_time > BLUR_HIDE_DELAY_MS => hide
//!   -> else => ignore (保护期)
//! ```
//!
//! 使用示例：
//! - 前端 Esc 调用 `hide_main_window`
//! - 按 `Ctrl+Shift+S` 切换窗口
// src-tauri/src/lib.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{App, AppHandle, Manager, State};

mod window;
use window::{init_window_events, register_global_shortcuts, toggle_main_window, WindowController};

#[tauri::command]
fn open_or_focus_main_window(app: AppHandle, ctrl: State<WindowController>) {
    toggle_main_window(&app, ctrl.inner())
}

#[cfg_attr(mobile, tauri::mobile_entry_point())]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::Builder::default().build())
        .invoke_handler(tauri::generate_handler![open_or_focus_main_window])
        .setup(|app: &mut App| {
            app.manage(WindowController::new());
            println!("[Setup] Registering global shortcuts...");
            register_global_shortcuts(app)?;

            println!("[Setup] Initializing window events...");
            let ctrl = app.state::<WindowController>();
            init_window_events(app.handle(), ctrl);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
