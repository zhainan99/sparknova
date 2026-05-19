//! 项目级公共路径与常量。
//!
//! 所有需要 `APP_NAME` 或 `%LOCALAPPDATA%\Nimbus` 的模块都从此处引用，
//! 避免字符串散落各处。

use std::path::PathBuf;

/// 应用名称 —— Rust/Slint 引用的唯一来源。
pub const APP_NAME: &str = "Nimbus";

/// 返回 `%LOCALAPPDATA%\Nimbus`（fallback → `%APPDATA%\Nimbus` → `.\Nimbus`）。
///
/// 不保证目录存在，调用方按需 `create_dir_all`。
pub fn app_data_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    base.join(APP_NAME)
}