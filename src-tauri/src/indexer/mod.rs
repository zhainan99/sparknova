//! 索引构建层：扫描系统、产出 [`crate::domain::AppEntry`] 列表。
//!
//! 规划（参见 docs/design.md §4.1 应用来源分层）：
//! - `app_scanner`   v0.1 —— 开始菜单 `.lnk` + 注册表 Uninstall
//! - `user_dir_scanner` v0.2 —— 用户自定义目录（绿色软件）
//! - `history_learner`  v0.3 —— Nimbus 内手动启动过的路径（"用过即学"）

mod app_scanner;
mod directory_scanner;
mod registry_scanner;