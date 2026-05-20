//! 应用索引缓存：存储扫描到的应用列表，实现冷启动秒加载。
//!
//! 支持增量更新：
//! - 保存上次扫描时间 (`last_scan_time`)
//! - 提供方法判断是否需要重新扫描

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::domain::{AppEntry, AppSource};

const INDEX_CACHE: TableDefinition<&str, &str> = TableDefinition::new("index_cache");
const LAST_SCAN_TIME_KEY: &str = "last_scan_time";
const SCAN_INTERVAL_SECS: u64 = 3600; // 默认1小时后才重新全量扫描

#[derive(Serialize, Deserialize)]
struct SerializedEntry {
    name: String,
    path: String,
    source: AppSource,
}

pub struct IndexCache {
    db: Arc<Database>,
}

impl IndexCache {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn init_table(db: &Database) -> Result<()> {
        let exists = db
            .begin_read()
            .ok()
            .and_then(|txn| txn.open_table(INDEX_CACHE).ok())
            .is_some();

        if exists {
            return Ok(());
        }

        let write_txn = db.begin_write()?;
        {
            let _table = write_txn.open_table(INDEX_CACHE)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn save(&self, entries: &[AppEntry]) -> Result<()> {
        let serialized: Vec<SerializedEntry> = entries
            .iter()
            .map(|e| SerializedEntry {
                name: e.name.clone(),
                path: e.path.to_string_lossy().to_string(),
                source: e.source,
            })
            .collect();

        let json = serde_json::to_string(&serialized).context("序列化索引失败")?;

        let write_txn = self.db.begin_write()?;

        {
            let mut table = write_txn.open_table(INDEX_CACHE)?;
            table.insert("index_snapshot", json.as_str())?;
        }

        write_txn.commit()?;

        info!(count = entries.len(), "保存索引快照");
        Ok(())
    }

    /// 获取上次扫描时间（秒级时间戳）
    pub fn get_last_scan_time(&self) -> Result<u64> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(INDEX_CACHE)?;
        
        match table.get(LAST_SCAN_TIME_KEY)? {
            Some(v) => Ok(v.value().to_string().parse().unwrap_or(0)),
            None => Ok(0),
        }
    }
    
    /// 设置上次扫描时间
    pub fn set_last_scan_time(&self, timestamp: u64) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(INDEX_CACHE)?;
            table.insert(LAST_SCAN_TIME_KEY, &*timestamp.to_string())?;
        }
        write_txn.commit()?;
        Ok(())
    }
    
    /// 获取当前时间戳（秒）
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// 检查是否需要进行增量扫描（基于时间间隔）
    /// 如果距上次扫描超过 SCAN_INTERVAL_SECS，返回 true
    pub fn needs_scan(&self) -> Result<bool> {
        let last_scan = self.get_last_scan_time()?;
        let now = Self::current_timestamp();
        
        // 如果从未扫描过（last_scan == 0），需要扫描
        // 或者距上次扫描超过间隔，需要扫描
        Ok(last_scan == 0 || now.saturating_sub(last_scan) >= SCAN_INTERVAL_SECS)
    }
    
    /// 获取距上次扫描的秒数
    pub fn seconds_since_last_scan(&self) -> Result<u64> {
        let last_scan = self.get_last_scan_time()?;
        let now = Self::current_timestamp();
        Ok(now.saturating_sub(last_scan))
    }

    pub fn append(&self, entry: &AppEntry) -> Result<()> {
        let mut entries = self.load().unwrap_or_default();
        if entries.iter().any(|e| e.path == entry.path) {
            return Ok(());
        }
        entries.push(entry.clone());
        self.save(&entries)
    }
    
    /// 合并新条目到缓存（去重，基于路径）
    pub fn merge_new_entries(&self, new_entries: &[AppEntry]) -> Result<Vec<AppEntry>> {
        let mut existing = self.load().unwrap_or_default();
        let existing_paths: std::collections::HashSet<_> = existing.iter()
            .map(|e| e.path.clone())
            .collect();
        
        let mut added_count = 0;
        for entry in new_entries {
            if !existing_paths.contains(&entry.path) {
                existing.push(entry.clone());
                added_count += 1;
            }
        }
        
        if added_count > 0 {
            self.save(&existing)?;
            info!(added = added_count, "合并新条目到索引缓存");
        }
        
        Ok(existing)
    }

    pub fn load(&self) -> Result<Vec<AppEntry>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(INDEX_CACHE)?;

        let json = match table.get("index_snapshot")? {
            Some(v) => v.value().to_string(),
            None => return Ok(Vec::new()),
        };

        let serialized: Vec<SerializedEntry> =
            serde_json::from_str(&json).context("解析索引缓存失败")?;

        let entries: Vec<AppEntry> = serialized
            .into_iter()
            .map(|s| AppEntry {
                name: s.name,
                path: std::path::PathBuf::from(s.path),
                source: s.source,
            })
            .collect();

        info!(count = entries.len(), "加载索引快照");
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redb::Database;
    use std::fs;

    fn temp_db() -> (IndexCache, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "sparknova_cache_test_{:?}_{:?}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let db = Arc::new(Database::create(&db_path).unwrap());
        IndexCache::init_table(&db).unwrap();

        let cache = IndexCache::new(db);
        (cache, temp_dir)
    }

    #[test]
    fn test_save_and_load() {
        let (cache, _temp_dir) = temp_db();

        let entries = vec![
            AppEntry {
                name: "Chrome".to_string(),
                path: std::path::PathBuf::from("/app/chrome"),
                source: AppSource::StartMenu,
            },
            AppEntry {
                name: "VSCode".to_string(),
                path: std::path::PathBuf::from("/app/vscode"),
                source: AppSource::RegistryUninstall,
            },
        ];

        cache.save(&entries).unwrap();

        let loaded = cache.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Chrome");
        assert_eq!(loaded[1].name, "VSCode");
    }

    #[test]
    fn test_load_empty() {
        let (cache, _temp_dir) = temp_db();

        let loaded = cache.load().unwrap();
        assert!(loaded.is_empty());
    }
}