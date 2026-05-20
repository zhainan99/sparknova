//! 项目级公共路径与常量。

use std::path::PathBuf;

/// 应用名称
pub const APP_NAME: &str = "SparkNova";

/// 返回项目数据目录（当前目录下的 .data/）
pub fn app_data_dir() -> PathBuf {
    let path = PathBuf::from(".").join(".data");
    std::fs::create_dir_all(&path).ok();
    path
}