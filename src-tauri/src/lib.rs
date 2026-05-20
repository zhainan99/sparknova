//! SparkNova 入口模块
//!
//! 功能：
//! - 全局快捷键切换主窗口显隐
//! - 失焦后按保护期自动隐藏
//! - 窗口尺寸自适应屏幕

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::App;
use tracing_subscriber::{fmt, EnvFilter};

mod window;
use window::{init_window_events, register_global_shortcuts, AppState};

#[tauri::command]
fn hide_main_window_cmd(app: tauri::AppHandle) -> bool {
    window::hide_main_window(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point())]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::Builder::default().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![hide_main_window_cmd])
        .setup(|app: &mut App| {
            // 初始化日志
            let _ = fmt()
                .with_env_filter(
                    EnvFilter::from_default_env().add_directive("info".parse().unwrap()),
                )
                .with_target(false)
                .compact()
                .try_init();

            // 注册全局快捷键
            register_global_shortcuts(app)?;

            // 初始化窗口事件
            init_window_events(&app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 tauri 应用时出错");
}