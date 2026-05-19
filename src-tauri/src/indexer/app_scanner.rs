//! 开始菜单扫描器：遍历用户级和系统级开始菜单，解析 `.lnk` 得到 [`AppEntry`]。
//!
//! v0.1 仅覆盖开始菜单 `.lnk`；注册表 Uninstall 源会在 v0.1 后期补 `registry_scanner` 子模块。

use anyhow::{Context, Result};
use lnk::{
    encoding::{UTF_16LE, WINDOWS_1252},
    ShellLink,
};
use std::path::{Path, PathBuf};
use std::{env, fs};
use tracing::{debug, info, warn};

use crate::domain::{AppEntry, AppSource};

/// 开始菜单相对子路径，同时适用于 APPDATA（用户级）和 PROGRAMDATA（系统级）。
const START_MENU_SUBDIR: &str = r"Microsoft\Windows\Start Menu\Programs";

/// 单次扫描结果与统计。字段为 pub 供调用方日志化；后续 search 引擎取 `entries`。
#[derive(Debug, Default)]
#[allow(dead_code)] // scanned_lnk/skipped 仅日志/诊断用；v0.2 引入 history_learner 时可能扩展
pub struct ScanReport {
    pub entries: Vec<AppEntry>,
    /// 遍历到的 `.lnk` 总数
    pub scanned_lnk: usize,
    /// 解析失败或目标非 `.exe` 被丢弃的数量
    pub skipped: usize,
}

/// 扫描开始菜单（用户级 + 系统级），返回命中的应用列表。
///
/// 失败策略：单个 `.lnk` 解析失败只记 `skipped` 并打 debug 日志，不中断整次扫描。
/// 根目录不存在（比如家庭版缺少某个路径）也只是跳过。
pub fn scan_start_menu() -> Result<ScanReport> {
    let mut report = ScanReport::default();

    for root in start_menu_roots() {
        if !root.exists() {
            debug!(?root, "开始菜单根目录不存在，跳过");
            continue;
        }
        debug!(?root, "扫描开始菜单根目录");
        scan_dir(&root, &mut report);
    }

    info!(
        entries = report.entries.len(),
        scanned = report.scanned_lnk,
        skipped = report.skipped,
        "开始菜单扫描完成"
    );
    Ok(report)
}

/// 开始菜单根目录：用户级（`%APPDATA%\...`）+ 系统级（`%PROGRAMDATA%\...`）。
/// 两个环境变量在任何现代 Windows 上都存在；非 Windows（测试环境）会全部跳过。
fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for key in ["APPDATA", "PROGRAMDATA"] {
        if let Ok(base) = env::var(key) {
            roots.push(PathBuf::from(base).join(START_MENU_SUBDIR));
        }
    }
    roots
}

/// 递归扫描目录。读目录失败（权限）只 warn 跳过，不中断整体扫描。
fn scan_dir(dir: &Path, report: &mut ScanReport) {
    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
            warn!(error = %e, ?dir, "读取目录失败，跳过子树");
            return;
        }
    };

    for entry in read {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, "读取目录项失败，跳过");
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, report);
        } else if is_lnk(&path) {
            report.scanned_lnk += 1;
            match parse_lnk(&path) {
                Ok(Some(app)) => report.entries.push(app),
                Ok(None) => report.skipped += 1, // 目标非 .exe 或无 link_target
                Err(e) => {
                    report.skipped += 1;
                    debug!(?path, error = %e, "解析 .lnk 失败");
                }
            }
        }
    }
}

/// 解析单个 `.lnk`。
/// 成功但目标不是 `.exe`（比如指向 URL、文件夹）→ `Ok(None)`，交由调用方计 `skipped`。
///
/// Windows 快捷方式的编码不统一：部分中文软件虽然设了 `IS_UNICODE` 标志，实际路径却是 ANSI 编码。
/// 因此先尝试 UTF-16LE，若目标路径不存在则回退到 WINDOWS_1252。
fn parse_lnk(lnk_path: &Path) -> Result<Option<AppEntry>> {
    // 先尝试 UTF-16LE（标准 Unicode 编码）
    let link = ShellLink::open(lnk_path, UTF_16LE).context("ShellLink::open 失败")?;
    let target_from_utf16 = link.link_target().map(PathBuf::from);

    // 如果 UTF-16LE 解码出的路径不存在，尝试 WINDOWS_1252（ANSI 编码）
    let target = match target_from_utf16 {
        Some(ref p) if p.exists() => Some(p.clone()),
        _ => {
            // 回退到 ANSI 编码
            match ShellLink::open(lnk_path, WINDOWS_1252) {
                Ok(link) => link.link_target().map(PathBuf::from),
                Err(_) => target_from_utf16, // 两种编码都失败，保留 UTF-16LE 结果
            }
        }
    };

    let target = match target {
        Some(t) => t,
        None => return Ok(None),
    };

    if !is_exe(&target) {
        return Ok(None);
    }

    // Windows 开始菜单惯例：显示名就是 .lnk 文件名（用户眼里看到的就是这个）。
    let name = lnk_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            target
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });

    Ok(Some(AppEntry {
        name,
        path: target,
        source: AppSource::StartMenu,
    }))
}

fn is_lnk(path: &Path) -> bool {
    has_ext(path, "lnk")
}

fn is_exe(path: &Path) -> bool {
    has_ext(path, "exe")
}

fn has_ext(path: &Path, want: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case(want))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn extension_matching_is_case_insensitive() {
        assert!(is_lnk(Path::new("a.lnk")));
        assert!(is_lnk(Path::new("a.LNK")));
        assert!(is_lnk(Path::new("dir/a.Lnk")));
        assert!(!is_lnk(Path::new("a.exe")));
        assert!(!is_lnk(Path::new("no_ext")));

        assert!(is_exe(Path::new("a.exe")));
        assert!(is_exe(Path::new("a.EXE")));
        assert!(!is_exe(Path::new("a.dll")));
    }

    #[test]
    fn scan_dir_on_empty_returns_zero() {
        let tmp = std::env::temp_dir().join("nimbus_scan_test_empty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let mut report = ScanReport::default();
        scan_dir(&tmp, &mut report);
        assert_eq!(report.entries.len(), 0);
        assert_eq!(report.scanned_lnk, 0);
        assert_eq!(report.skipped, 0);

        let _ = fs::remove_dir_all(&tmp);
    }
}