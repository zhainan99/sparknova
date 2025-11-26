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

use tauri::{App, AppHandle, Manager};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod window;
use window::{register_global_shortcuts, toggle_main_window};

#[tauri::command]
fn open_or_focus_main_window(app: AppHandle) {
    toggle_main_window(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point())]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::Builder::default().build())
        .invoke_handler(tauri::generate_handler![open_or_focus_main_window])
        .setup(|app: &mut App| {
            let _ = fmt()
                .with_env_filter(
                    EnvFilter::from_default_env().add_directive("info".parse().unwrap()),
                )
                .with_target(false)
                .compact()
                .try_init();
            info!("Registering global shortcuts...");
            register_global_shortcuts(app)?;

            #[cfg(debug_assertions)]
            {
                if let Some(_win) = app.get_webview_window("main") {
                    // let _ = win.open_devtools();
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
