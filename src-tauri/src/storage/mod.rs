//! 存储层：redb + SQLite 双数据库架构。
//!
//! - redb: 高性能 KV 存储，用于 index_cache（搜索索引）、frequency（启动频次）
//! - SQLite: 结构化数据存储，用于 notes（笔记），支持复杂查询

mod frequency;
mod index_cache;
mod note_store;
mod sqlite_db;

use std::sync::Arc;

use anyhow::{Context, Result};
use redb::{Database, TableDefinition};
use tracing::info;

use crate::paths::app_data_dir;

pub use frequency::FrequencyStore;
pub use index_cache::IndexCache;
pub use note_store::NoteStore;
pub use sqlite_db::SqliteDb;

/// 数据库版本号，用于 schema 变更时的迁移检查。
const DB_VERSION: u32 = 1;

/// 元数据表：存储版本等信息。
const META: TableDefinition<&str, u32> = TableDefinition::new("meta");

/// 存储管理器：封装 redb 数据库操作。
pub struct Storage {
    db: Arc<Database>,
    sqlite: SqliteDb,
}

impl Storage {
    /// 打开或创建数据库。
    ///
    /// - redb 数据库：应用索引、启动频次
    /// - SQLite 数据库：笔记等结构化数据
    pub fn open() -> Result<Self> {
        // 打开 redb
        let redb_path = get_redb_path()?;
        info!(path = ?redb_path, "打开 redb 数据库");

        let db = Arc::new(Database::create(&redb_path).context("创建/打开 redb 数据库失败")?);

        // 打开 SQLite
        let sqlite = SqliteDb::open().context("打开 SQLite 数据库失败")?;

        let storage = Self { db, sqlite };
        storage.init_schema()?;

        Ok(storage)
    }

    /// 初始化 redb 数据库 schema（仅在首次运行时写入版本号）。
    fn init_schema(&self) -> Result<()> {
        // 检查是否已初始化：读事务无开销，跳过重复写入
        let needs_init = self
            .db
            .begin_read()
            .ok()
            .and_then(|txn| txn.open_table(META).ok())
            .and_then(|table| table.get("version").ok().flatten())
            .map(|v| v.value())
            != Some(DB_VERSION);

        if needs_init {
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(META)?;
                table.insert("version", &DB_VERSION)?;
            }
            write_txn.commit()?;
        }

        // 初始化 redb 子模块的表
        FrequencyStore::init_table(&self.db)?;
        IndexCache::init_table(&self.db)?;

        // SQLite 的 notes 表在 SqliteDb::init_schema() 中已创建

        Ok(())
    }

    /// 获取频次存储访问器。
    #[allow(dead_code)]
    pub fn frequency(&self) -> FrequencyStore {
        FrequencyStore::new(self.db.clone())
    }

    /// 获取索引缓存访问器。
    pub fn index_cache(&self) -> IndexCache {
        IndexCache::new(self.db.clone())
    }

    /// 获取笔记存储访问器（基于 SQLite）。
    pub fn note_store(&self) -> NoteStore {
        NoteStore::new(self.sqlite.clone())
    }

    /// 获取设置存储访问器（基于 SQLite）。
    #[allow(dead_code)] // v0.3 设置 UI 配置页扩展后用上
    pub fn settings(&self) -> &SqliteDb {
        &self.sqlite
    }

    /// 获取 SQLite 数据库引用（用于 Config 等模块直接访问）。
    #[allow(dead_code)]
    pub fn sqlite(&self) -> &SqliteDb {
        &self.sqlite
    }

    /// 获取自身的 Arc 引用（用于跨模块共享）。
    #[allow(dead_code)] // v0.2 事件总线引入后使用
    pub fn arc(&self) -> Arc<Self> {
        Arc::new(Storage {
            db: self.db.clone(),
            sqlite: self.sqlite.clone(),
        })
    }
}

/// 获取 redb 数据库文件路径。
fn get_redb_path() -> Result<std::path::PathBuf> {
    let nimbus_dir = app_data_dir();
    std::fs::create_dir_all(&nimbus_dir)?;
    Ok(nimbus_dir.join("nimbus_index.db"))
}

/// 获取 SQLite 数据库文件路径（在 sqlite_db.rs 中定义，这里仅用于说明）
/// 注意：SQLite 使用 nimbus.db（和 redb 分离）
const _: &str = "SQLite 数据库路径: {app_data_dir}/nimbus.db";

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_storage() -> (Storage, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "nimbus_test_{:?}_{:?}",
            std::thread::current::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        // 临时替换路径
        std::env::set_var("LOCALAPPDATA", &temp_dir);

        let storage = Storage::open().expect("创建测试存储失败");
        (storage, temp_dir)
    }

    #[test]
    fn test_storage_init() {
        let (storage, _temp_dir) = temp_storage();

        // 验证可以正常访问
        let _freq = storage.frequency();
        let _cache = storage.index_cache();
        let _notes = storage.note_store();
    }

    #[test]
    fn test_note_store_sqlite() {
        let (storage, _temp_dir) = temp_storage();

        let notes = storage.note_store();

        // 测试插入
        let note = notes
            .insert("测试笔记", crate::domain::NoteKind::Memo)
            .unwrap();
        assert_eq!(note.content, "测试笔记");

        // 测试查询
        let retrieved = notes.get(note.id).unwrap().unwrap();
        assert_eq!(retrieved.content, "测试笔记");

        // 测试列表
        let all = notes.list_all().unwrap();
        assert_eq!(all.len(), 1);
    }
}