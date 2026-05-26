//! 开始菜单扫描器：遍历用户级和系统级开始菜单，解析 `.lnk` 得到 [`AppEntry`]。

#![allow(dead_code)]

use anyhow::Result;
use lnk::ShellLink;
use std::path::{Path, PathBuf};
use std::{env, fs};
use tracing::{debug, info};

use crate::domain::{AppEntry, AppSource};

/// 单次扫描结果与统计。字段为 pub 供调用方日志化；后续 search 引擎取 `entries`。
#[derive(Debug, Default)]
pub struct ScanReport {
    pub entries: Vec<AppEntry>,
    /// 遍历到的 `.lnk` 总数
    pub scanned_lnk: usize,
    /// 解析失败或目标非 `.exe` 被丢弃的数量
    pub skipped: usize,
}

/// 扫描开始菜单（用户级 + 系统级），返回命中的应用列表。
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

fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    // 用户级开始菜单
    if let Some(apdata) = env::var_os("APPDATA") {
        let user_root = PathBuf::from(apdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        roots.push(user_root);
    }

    // 系统级开始菜单
    if let Some(progdata) = env::var_os("PROGRAMDATA") {
        let sys_root = PathBuf::from(progdata)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs");
        roots.push(sys_root);
    }

    roots
}

fn scan_dir(dir: &Path, report: &mut ScanReport) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, report);
        } else if is_lnk(&path) {
            match parse_lnk(&path) {
                Ok(Some(app)) => {
                    report.entries.push(app);
                    report.scanned_lnk += 1;
                }
                Ok(None) => {
                    report.skipped += 1;
                }
                Err(e) => {
                    debug!(?path, ?e, "解析 lnk 失败");
                    report.skipped += 1;
                }
            }
        }
    }
}

fn parse_lnk(lnk_path: &Path) -> Result<Option<AppEntry>> {
    let shell_link = match ShellLink::open(lnk_path, lnk::encoding::UTF_16LE) {
        Ok(sl) => sl,
        Err(e) => {
            // 大量 lnk 文件解析失败是正常的（网络路径、特殊快捷方式等），仅 debug
            debug!(lnk_path = ?lnk_path, error = %e, "打开 lnk 文件失败");
            return Ok(None);
        }
    };

    let target = if let Some(ref info) = shell_link.link_info() {
        match info.local_base_path() {
            Some(t) => t.to_string(),
            None => {
                // 大部分快捷方式没有 local_base_path 是正常的
                debug!(lnk_path = ?lnk_path, "lnk 文件无 local_base_path");
                return Ok(None);
            }
        }
    } else {
        debug!(lnk_path = ?lnk_path, "lnk 文件无 link_info");
        return Ok(None);
    };

    let target_path = PathBuf::from(target);

    // 必须是 .exe 文件
    if !is_exe(&target_path) {
        debug!(lnk_path = ?lnk_path, target_path = ?target_path, "lnk 目标不是 exe");
        return Ok(None);
    }

    let name = lnk_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(Some(AppEntry {
        name,
        path: target_path,
        source: AppSource::StartMenu,
    }))
}

fn is_lnk(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("lnk")
}

fn is_exe(path: &Path) -> bool {
    has_ext(path, "exe")
}

fn has_ext(path: &Path, want: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase() == want)
        .unwrap_or(false)
}