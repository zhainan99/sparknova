//! 应用索引缓存：存储扫描到的应用列表，实现冷启动秒加载。
//!
//! 表结构：
//! - key: "index_snapshot"（固定键）
//! - value: 序列化后的应用列表（JSON）

use std::sync::Arc;

use anyhow::{Context, Result};
use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::domain::{AppEntry, AppSource};

/// 索引缓存表定义。
const INDEX_CACHE: TableDefinition<&str, &str> = TableDefinition::new("index_cache");

/// 序列化后的应用条目（用于存储）。
#[derive(Serialize, Deserialize)]
struct SerializedEntry {
    name: String,
    path: String,
    source: AppSource,
}

/// 索引缓存访问器。
///
/// 使用 `Arc<Database>` 共享数据库引用，避免悬垂指针。
pub struct IndexCache {
    db: Arc<Database>,
}

impl IndexCache {
    /// 创建新的索引缓存访问器。
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 初始化索引缓存表（仅在首次运行时创建，已有则跳过）。
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

    /// 保存索引快照。
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

    /// 追加单个条目到索引缓存（Layer 3 "用过即学"）。
    ///
    /// 加载现有条目、追加、再保存。索引量级小（~500 条），全量重写开销可忽略。
    pub fn append(&self, entry: &AppEntry) -> Result<()> {
        let mut entries = self.load().unwrap_or_default();
        // 去重：同路径已存在则跳过
        if entries.iter().any(|e| e.path == entry.path) {
            return Ok(());
        }
        entries.push(entry.clone());
        self.save(&entries)
    }

    /// 加载索引快照。
    ///
    /// 如果缓存不存在或解析失败，返回空列表。
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

    /// 创建临时数据库，返回 (cache, temp_dir)。
    fn temp_db() -> (IndexCache, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "nimbus_cache_test_{:?}_{:?}",
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