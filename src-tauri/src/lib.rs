//! SparkNova 窗口控制模块
//!
//! 功能：
//! - 全局快捷键切换主窗口显隐
//! - 失焦后按保护期自动隐藏
//! - 提供命令给前端（隐藏/切换）

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::App;
use tracing_subscriber::{fmt, EnvFilter};

mod shortcuts;
mod window;
use shortcuts::register_global_shortcuts;

#[cfg_attr(mobile, tauri::mobile_entry_point())]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::Builder::default().build())
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 tauri 应用时出错");
}
