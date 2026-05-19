use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用的来源类型。详见 docs/design.md §4.1「应用来源分层」。
///
/// v0.1 只会产出 [`AppSource::StartMenu`] 和 [`AppSource::RegistryUninstall`]，
/// 其余变体是 v0.2/v0.3 预留（存储 schema 先占位，避免后续迁移）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)] // 预留变体；v0.2 移除 UserDirectory，v0.3 移除 UserHistory
pub enum AppSource {
    /// 开始菜单 `.lnk`
    StartMenu,
    /// 注册表 Uninstall 键
    RegistryUninstall,
    /// 用户自定义目录扫描（v0.2）
    UserDirectory,
    /// 用户启动历史，"用过即学"（v0.3）
    UserHistory,
}

/// 单个可启动应用的元数据。
///
/// 不同来源填充字段的来路：
/// - `StartMenu`：`name` 取自 `.lnk` 文件名（Windows 开始菜单惯例），`path` 取自 `link_target()`
/// - `RegistryUninstall`：`name` 取自 `DisplayName`，`path` 取自 `DisplayIcon` 或 `InstallLocation`
#[derive(Debug, Clone)]
pub struct AppEntry {
    /// UI 显示名
    pub name: String,
    /// 可执行文件绝对路径
    pub path: PathBuf,
    /// 来源标签，供排序权重和用户过滤使用
    pub source: AppSource,
}