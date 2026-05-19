//! 使用频次存储：记录每个应用的启动次数。
//!
//! 表结构：
//! - key: 应用路径（String）
//! - value: 启动次数（u32）

use std::sync::Arc;

use anyhow::Result;
use redb::{Database, ReadableTable, TableDefinition};
use tracing::{debug, info};

/// 频次表定义。
const FREQUENCY: TableDefinition<&str, u32> = TableDefinition::new("frequency");

/// 频次存储访问器。
///
/// 使用 `Arc<Database>` 共享数据库引用，避免悬垂指针。
#[allow(dead_code)] // v0.2 使用历史展示页面接入后移除
pub struct FrequencyStore {
    db: Arc<Database>,
}

#[allow(dead_code)] // v0.2 使用历史展示页面接入后移除
impl FrequencyStore {
    /// 创建新的频次存储访问器。
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 初始化频次表（仅在首次运行时创建，已有则跳过）。
    pub fn init_table(db: &Database) -> Result<()> {
        // 检查表是否已存在，避免不必要的写事务
        let exists = db
            .begin_read()
            .ok()
            .and_then(|txn| txn.open_table(FREQUENCY).ok())
            .is_some();

        if exists {
            return Ok(());
        }

        let write_txn = db.begin_write()?;
        {
            let _table = write_txn.open_table(FREQUENCY)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// 记录一次应用启动。
    pub fn record_launch(&self, app_path: &str) -> Result<()> {
        let write_txn = self.db.begin_write()?;

        {
            let mut table = write_txn.open_table(FREQUENCY)?;
            let current = table.get(app_path)?.map(|v| v.value()).unwrap_or(0);
            let new_count = current + 1;
            table.insert(app_path, &new_count)?;
        }

        write_txn.commit()?;

        debug!(path = app_path, "记录应用启动");
        Ok(())
    }

    /// 获取应用的启动次数。
    pub fn get_count(&self, app_path: &str) -> Result<u32> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(FREQUENCY)?;

        let count = table.get(app_path)?.map(|v| v.value()).unwrap_or(0);

        Ok(count)
    }

    /// 获取所有应用的频次统计。
    ///
    /// 返回 (应用路径, 启动次数) 的列表。
    pub fn get_all(&self) -> Result<Vec<(String, u32)>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(FREQUENCY)?;

        let mut results = Vec::new();
        for entry in table.iter()? {
            let (key, value) = entry?;
            results.push((key.value().to_string(), value.value()));
        }

        Ok(results)
    }

    /// 批量导入频次数据（用于从缓存恢复）。
    pub fn import(&self, frequencies: Vec<(String, u32)>) -> Result<()> {
        let write_txn = self.db.begin_write()?;

        {
            let mut table = write_txn.open_table(FREQUENCY)?;
            for (path, count) in &frequencies {
                table.insert(path.as_str(), count)?;
            }
        }

        write_txn.commit()?;

        info!(count = frequencies.len(), "导入频次数据");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redb::Database;
    use std::fs;

    /// 创建临时数据库，返回 (store, temp_dir)。
    fn temp_db() -> (FrequencyStore, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "nimbus_freq_test_{:?}_{:?}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        let db = Arc::new(Database::create(&db_path).unwrap());
        FrequencyStore::init_table(&db).unwrap();

        let store = FrequencyStore::new(db);
        (store, temp_dir)
    }

    #[test]
    fn test_record_and_get() {
        let (store, _temp_dir) = temp_db();

        assert_eq!(store.get_count("/app/chrome").unwrap(), 0);

        store.record_launch("/app/chrome").unwrap();
        store.record_launch("/app/chrome").unwrap();

        assert_eq!(store.get_count("/app/chrome").unwrap(), 2);
    }

    #[test]
    fn test_get_all() {
        let (store, _temp_dir) = temp_db();

        store.record_launch("/app/a").unwrap();
        store.record_launch("/app/b").unwrap();
        store.record_launch("/app/a").unwrap();

        let all = store.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_import() {
        let (store, _temp_dir) = temp_db();

        let data = vec![
            ("/app/chrome".to_string(), 5u32),
            ("/app/vscode".to_string(), 3u32),
        ];

        store.import(data).unwrap();

        assert_eq!(store.get_count("/app/chrome").unwrap(), 5);
        assert_eq!(store.get_count("/app/vscode").unwrap(), 3);
    }
}