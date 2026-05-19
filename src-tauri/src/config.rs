//! 用户配置：管理扫描目录、主题、启动行为等用户可自定义的设置。
//!
//! 使用 SQLite 存储，提供更强的查询能力。

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::{debug, info};

use crate::storage::SqliteDb;

/// 主题模式常量。Rust 侧统一引用，避免 `"dark"`/`"light"` 散落各处。
pub const THEME_DARK: &str = "dark";
pub const THEME_LIGHT: &str = "light";

/// 设置键常量。
pub const KEY_THEME_MODE: &str = "theme_mode";
pub const KEY_SHOW_MAIN_ON_START: &str = "show_main_on_start";
pub const KEY_SHOW_NOTES_ON_START: &str = "show_notes_on_start";

/// 用户可配置项。
#[derive(Clone)]
pub struct UserConfig {
    pub scan_dirs: Vec<String>,
    pub theme_mode: String,
    pub show_main_on_start: bool,
    pub show_notes_on_start: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            scan_dirs: Vec::new(),
            theme_mode: THEME_DARK.to_string(),
            show_main_on_start: false,
            show_notes_on_start: false,
        }
    }
}

impl UserConfig {
    /// 从 SqliteDb 加载配置（用于共享存储场景）。
    pub fn load_from_sqlite(sqlite: &SqliteDb) -> Result<Self> {
        let theme_mode = sqlite
            .get_setting(KEY_THEME_MODE)
            .unwrap_or_else(|_| Some(THEME_DARK.to_string()))
            .unwrap_or_else(|| THEME_DARK.to_string());

        let show_main_on_start = sqlite
            .get_setting(KEY_SHOW_MAIN_ON_START)?
            .map(|v| v == "true")
            .unwrap_or(false);

        let show_notes_on_start = sqlite
            .get_setting(KEY_SHOW_NOTES_ON_START)?
            .map(|v| v == "true")
            .unwrap_or(false);

        let scan_dirs = sqlite.get_scan_dirs().unwrap_or_default();

        Ok(Self {
            scan_dirs,
            theme_mode,
            show_main_on_start,
            show_notes_on_start,
        })
    }

    /// 从单独的 SQLite 连接加载配置（用于 UI 回调等场景）。
    pub fn load() -> Result<Self> {
        let sqlite = SqliteDb::open()?;
        Self::load_from_sqlite(&sqlite)
    }

    /// 保存配置到 SqliteDb。
    pub fn save_to_sqlite(&self, sqlite: &SqliteDb) -> Result<()> {
        debug!("保存用户配置");

        sqlite.set_setting(KEY_THEME_MODE, &self.theme_mode)?;
        sqlite.set_setting(
            KEY_SHOW_MAIN_ON_START,
            if self.show_main_on_start { "true" } else { "false" },
        )?;
        sqlite.set_setting(
            KEY_SHOW_NOTES_ON_START,
            if self.show_notes_on_start { "true" } else { "false" },
        )?;

        let current_dirs = sqlite.get_scan_dirs().unwrap_or_default();
        for dir in &current_dirs {
            if !self.scan_dirs.contains(dir) {
                if let Some(idx) = current_dirs.iter().position(|d| d == dir) {
                    sqlite.remove_scan_dir(idx)?;
                }
            }
        }
        for dir in &self.scan_dirs {
            sqlite.add_scan_dir(dir)?;
        }

        info!("配置已保存");
        Ok(())
    }

    /// 保存配置（创建单独的 SQLite 连接）。
    pub fn save(&self) -> Result<()> {
        let sqlite = SqliteDb::open()?;
        self.save_to_sqlite(&sqlite)
    }

    /// 返回扫描目录的 `PathBuf` 列表。
    pub fn scan_dirs_paths(&self) -> Vec<PathBuf> {
        self.scan_dirs.iter().map(PathBuf::from).collect()
    }

    /// 添加扫描目录（自动去重 + 跳过不存在的路径）。
    pub fn add_scan_dir(&mut self, dir: &Path) {
        let s = dir.to_string_lossy().to_string();
        if !self.scan_dirs.contains(&s) && dir.is_dir() {
            self.scan_dirs.push(s.clone());
            if let Err(e) = self.save() {
                tracing::warn!(error = %e, "保存扫描目录失败");
            }
        }
    }

    /// 按索引移除扫描目录。
    pub fn remove_scan_dir(&mut self, index: usize) {
        if index < self.scan_dirs.len() {
            self.scan_dirs.remove(index);
            if let Err(e) = self.save() {
                tracing::warn!(error = %e, "保存扫描目录失败");
            }
        }
    }

    /// 设置主题模式。
    pub fn set_theme(&mut self, mode: &str) {
        self.theme_mode = mode.to_string();
        if let Err(e) = self.save() {
            tracing::warn!(error = %e, "保存主题失败");
        }
    }

    /// 设置启动行为配置。
    pub fn set_show_main_on_start(&mut self, show: bool) {
        self.show_main_on_start = show;
        if let Err(e) = self.save() {
            tracing::warn!(error = %e, "保存启动配置失败");
        }
    }

    /// 设置启动笔记窗口配置。
    pub fn set_show_notes_on_start(&mut self, show: bool) {
        self.show_notes_on_start = show;
        if let Err(e) = self.save() {
            tracing::warn!(error = %e, "保存启动配置失败");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_theme() {
        let config = UserConfig::default();
        assert_eq!(config.theme_mode, THEME_DARK);
    }

    #[test]
    fn test_default_startup_flags() {
        let config = UserConfig::default();
        assert!(!config.show_main_on_start);
        assert!(!config.show_notes_on_start);
    }

    #[test]
    fn test_set_theme() {
        let mut config = UserConfig::default();
        config.set_theme(THEME_LIGHT);
        assert_eq!(config.theme_mode, THEME_LIGHT);
    }

    #[test]
    fn test_add_scan_dir() {
        let mut config = UserConfig::default();
        let temp_dir = std::env::temp_dir().join(format!(
            "nimbus_config_test_{:?}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        config.add_scan_dir(&temp_dir);
        assert!(config
            .scan_dirs
            .contains(&temp_dir.to_string_lossy().to_string()));
    }

    #[test]
    fn test_remove_scan_dir() {
        let mut config = UserConfig::default();
        config.scan_dirs.push("C:\\test1".to_string());
        config.scan_dirs.push("C:\\test2".to_string());

        config.remove_scan_dir(0);
        assert_eq!(config.scan_dirs.len(), 1);
        assert_eq!(config.scan_dirs[0], "C:\\test2");
    }
}