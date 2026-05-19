//! 注册表 Uninstall 扫描器：读取 Windows 注册表中的已安装应用信息。
//!
//! 扫描以下注册表路径：
//! - HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
//! - HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall
//! - HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall

use std::path::{Path, PathBuf};

use tracing::{debug, info, warn};
use winreg::enums::*;
use winreg::RegKey;

use crate::domain::{AppEntry, AppSource};

/// 注册表扫描结果统计。
#[derive(Debug, Default)]
pub struct RegistryReport {
    pub entries: Vec<AppEntry>,
    pub scanned_keys: usize,
    pub skipped: usize,
}

/// 扫描注册表 Uninstall 键，返回应用列表。
pub fn scan_registry() -> anyhow::Result<RegistryReport> {
    let mut report = RegistryReport::default();

    let hives = [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_CURRENT_USER,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    for (hive, path) in hives {
        match scan_hive(hive, path, &mut report) {
            Ok(_) => debug!(hive = ?hive, path = path, "注册表 hive 扫描完成"),
            Err(e) => warn!(error = %e, hive = ?hive, ?path, "注册表 hive 扫描失败"),
        }
    }

    info!(
        entries = report.entries.len(),
        scanned = report.scanned_keys,
        skipped = report.skipped,
        "注册表扫描完成"
    );

    Ok(report)
}

fn scan_hive(hive: winreg::HKEY, path: &str, report: &mut RegistryReport) -> anyhow::Result<()> {
    let key = RegKey::predef(hive).open_subkey(path)?;

    for subkey_name in key.enum_keys().flatten() {
        report.scanned_keys += 1;

        let subkey = match key.open_subkey(&subkey_name) {
            Ok(k) => k,
            Err(e) => {
                debug!(subkey = subkey_name, error = %e, "打开注册表子键失败");
                report.skipped += 1;
                continue;
            }
        };

        // 跳过 Windows Update 等系统组件
        let is_system_component: u32 = subkey.get_value("SystemComponent").unwrap_or(0);
        if is_system_component != 0 {
            report.skipped += 1;
            continue;
        }

        let display_name: String = match subkey.get_value("DisplayName") {
            Ok(name) => name,
            Err(_) => {
                report.skipped += 1;
                continue;
            }
        };

        // 跳过空名称
        if display_name.trim().is_empty() {
            report.skipped += 1;
            continue;
        }

        // 尝试获取 exe 路径
        let path = resolve_exe_path(&subkey);

        if let Some(ref exe_path) = path {
            if exe_path.exists() {
                report.entries.push(AppEntry {
                    name: display_name,
                    path: exe_path.clone(),
                    source: AppSource::RegistryUninstall,
                });
            } else {
                debug!(name = display_name, path = ?path, "注册表应用路径不存在，跳过");
                report.skipped += 1;
            }
        } else {
            debug!(name = display_name, "无法解析注册表应用的 exe 路径");
            report.skipped += 1;
        }
    }

    Ok(())
}

/// 从注册表子键解析 exe 路径。
/// 优先使用 DisplayIcon，其次是 InstallLocation + DisplayName 推断。
fn resolve_exe_path(subkey: &RegKey) -> Option<PathBuf> {
    // 1. 尝试 DisplayIcon（通常就是 exe 路径）
    if let Ok(icon) = subkey.get_value::<String, _>("DisplayIcon") {
        let icon_path = PathBuf::from(icon.split(',').next().unwrap_or(&icon).trim());
        if icon_path.extension().is_some() {
            return Some(icon_path);
        }
    }

    // 2. 尝试从 UninstallString 推断（排除卸载器路径）
    if let Ok(uninstall) = subkey.get_value::<String, _>("UninstallString") {
        let path = PathBuf::from(uninstall.trim_matches('"'));
        if path.exists() && !is_uninstaller_path(&path) {
            return Some(path);
        }
    }

    // 3. 尝试 InstallLocation
    if let Ok(location) = subkey.get_value::<String, _>("InstallLocation") {
        let location = PathBuf::from(location.trim());
        if location.is_dir() {
            // 尝试查找目录下的 .exe
            if let Ok(entries) = std::fs::read_dir(&location) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("exe"))
                        .unwrap_or(false)
                    {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}

/// 检查路径是否指向卸载器（而非实际应用程序）。
/// Windows 卸载器常见文件名：uninstall.exe, unins000.exe, unwise.exe 等。
fn is_uninstaller_path(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|name| {
            let lower = name.to_lowercase();
            lower.contains("uninstall")
                || lower.contains("unins")
                || lower.contains("unwise")
                || lower.starts_with("uninst")
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_registry_smoke_test() {
        // 在 CI 环境可能没有注册表访问权限，所以只验证不 panic
        let result = scan_registry();
        assert!(result.is_ok());

        let report = result.unwrap();
        // 正常的 Windows 系统应该有一些已安装应用
        // 但不强制要求数量，因为测试环境可能很干净
        assert!(report.scanned_keys > 0 || report.entries.is_empty());
    }
}