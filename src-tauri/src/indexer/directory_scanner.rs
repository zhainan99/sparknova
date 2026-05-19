//! 用户自定义目录扫描器：递归扫描用户配置的目录，收集 `.exe` 文件。
//!
//! 对应 docs/design.md §4.1 Layer 2：覆盖绿色软件/便携版应用。
//!
//! 过滤规则（同 §4.1）：
//! - 仅 `.exe` 扩展名
//! - 文件名黑名单（安装器/卸载器特征）
//! - 排除系统目录
//! - 体积下限 100 KB（过滤 stub）

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::{fs, io};
use tracing::{debug, info, warn};

use crate::domain::{AppEntry, AppSource};

/// 递归最大深度，防止极深目录或符号链接环导致栈溢出。
const MAX_DEPTH: u32 = 5;

/// 体积下限（字节）：小于此值的 `.exe` 通常是 stub，跳过。
const MIN_SIZE_BYTES: u64 = 100 * 1024;

/// 排除的目录名（大小写不敏感）。
const EXCLUDED_DIRS: &[&str] = &[
    "$Recycle.Bin",
    "Windows",
    "WinSxS",
    "System Volume Information",
    "Temp",
    "tmp",
];

/// 文件名黑名单前缀（大小写不敏感）。
const NAME_BLACKLIST_PREFIXES: &[&str] = &[
    "setup",
    "unins",
    "uninstall",
    "unwise",
    "install",
    "vc_redist",
    "vcredist",
];

/// 单次扫描结果。
#[derive(Debug, Default)]
pub struct ScanReport {
    pub entries: Vec<AppEntry>,
    /// 遍历到的 `.exe` 总数
    pub scanned_exes: usize,
    /// 被过滤规则跳过的数量
    pub skipped: usize,
    /// 无法访问的目录数
    pub err_dirs: usize,
}

/// 扫描用户自定义目录列表，返回收集到的应用条目。
///
/// 每个目录独立处理，一个目录的失败不影响其他目录。
pub fn scan_user_dirs(dirs: &[PathBuf]) -> Result<ScanReport> {
    let mut report = ScanReport::default();

    if dirs.is_empty() {
        info!("未配置用户扫描目录，跳过 Layer 2 扫描");
        return Ok(report);
    }

    for dir in dirs {
        if !dir.exists() {
            debug!(?dir, "用户目录不存在，跳过");
            continue;
        }
        scan_dir(dir, 0, &mut report);
    }

    info!(
        entries = report.entries.len(),
        scanned = report.scanned_exes,
        skipped = report.skipped,
        err_dirs = report.err_dirs,
        "用户目录扫描完成"
    );
    Ok(report)
}

fn scan_dir(dir: &Path, depth: u32, report: &mut ScanReport) {
    if depth > MAX_DEPTH {
        return;
    }

    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                warn!(error = %e, ?dir, "读取用户目录失败，跳过子树");
                report.err_dirs += 1;
            }
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
            let skip = entry
                .file_name()
                .to_str()
                .map(|n| EXCLUDED_DIRS.iter().any(|ex| ex.eq_ignore_ascii_case(n)))
                .unwrap_or(false);
            if skip {
                continue;
            }
            scan_dir(&path, depth + 1, report);
        } else if is_exe(&path) {
            report.scanned_exes += 1;
            if should_skip_exe(&path) {
                report.skipped += 1;
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());
            report.entries.push(AppEntry {
                name,
                path,
                source: AppSource::UserDirectory,
            });
        }
    }
}

/// 检查 `.exe` 是否应被过滤。
fn should_skip_exe(path: &Path) -> bool {
    // 体积检查
    if let Ok(meta) = fs::metadata(path) {
        if meta.len() < MIN_SIZE_BYTES {
            debug!(?path, size = meta.len(), "体积过小，跳过");
            return true;
        }
    }

    // 文件名黑名单检查
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let lower = stem.to_lowercase();
    for prefix in NAME_BLACKLIST_PREFIXES {
        if lower.starts_with(prefix) {
            debug!(?path, "命中黑名单规则，跳过");
            return true;
        }
    }

    false
}

fn is_exe(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("exe"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_dirs_returns_empty() {
        let report = scan_user_dirs(&[]).unwrap();
        assert!(report.entries.is_empty());
    }

    #[test]
    fn nonexistent_dir_is_skipped() {
        let dirs = vec![PathBuf::from("Z:\\NimbusNonExistentDir_12345")];
        let report = scan_user_dirs(&dirs).unwrap();
        assert!(report.entries.is_empty());
    }

    #[test]
    fn should_skip_installer_names() {
        assert!(should_skip_exe(Path::new("setup.exe")));
        assert!(should_skip_exe(Path::new("Setup_v2.exe")));
        assert!(should_skip_exe(Path::new("unins000.exe")));
        assert!(should_skip_exe(Path::new("Uninstall.exe")));
        assert!(should_skip_exe(Path::new("vc_redist.x64.exe")));
    }
}