//! SQLite 数据库模块：笔记等结构化数据的持久化存储。
//!
//! 使用 rusqlite 实现，支持复杂的查询能力（按类型筛选、按日期范围等）。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use tracing::info;

use crate::domain::{Note, NoteKind};
use crate::paths::app_data_dir;

/// SQLite 数据库封装。
#[derive(Clone)]
pub struct SqliteDb {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteDb {
    /// 打开或创建 SQLite 数据库。
    pub fn open() -> Result<Self> {
        let db_path = get_db_path()?;
        info!(path = ?db_path, "打开 SQLite 数据库");

        let conn = Connection::open(&db_path).context("打开 SQLite 数据库失败")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;

        Ok(db)
    }

    /// 初始化数据库 schema。
    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // 创建笔记表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                kind TEXT NOT NULL,
                done INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .context("创建 notes 表失败")?;

        // 创建设置表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )
        .context("创建 settings 表失败")?;

        // 创建扫描目录表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS scan_dirs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE
            )",
            [],
        )
        .context("创建 scan_dirs 表失败")?;

        info!("SQLite 数据库初始化完成");
        Ok(())
    }

    /// 插入新笔记。
    pub fn insert_note(&self, content: &str, kind: NoteKind) -> Result<Note> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let kind_str = match kind {
            NoteKind::Todo => "todo",
            NoteKind::Memo => "memo",
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notes (content, kind, done, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![content, kind_str, 0, now],
        )
        .context("插入笔记失败")?;

        let id = conn.last_insert_rowid();

        info!(note_id = id, kind = ?kind, "笔记已创建");

        Ok(Note {
            id: id as u64,
            content: content.to_string(),
            kind,
            done: false,
            created_at: now,
        })
    }

    /// 更新笔记内容。
    #[allow(dead_code)] // v0.2 笔记编辑接入后使用
    pub fn update_note(&self, id: u64, content: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notes SET content = ?1 WHERE id = ?2",
            params![content, id as i64],
        )
        .context("更新笔记失败")?;
        Ok(())
    }

    /// 切换 Todo 完成状态。
    pub fn toggle_done(&self, id: u64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notes SET done = NOT done WHERE id = ?1",
            params![id as i64],
        )
        .context("切换完成状态失败")?;
        Ok(())
    }

    /// 删除笔记。
    pub fn delete_note(&self, id: u64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notes WHERE id = ?1", params![id as i64])
            .context("删除笔记失败")?;
        Ok(())
    }

    /// 获取单条笔记。
    pub fn get_note(&self, id: u64) -> Result<Option<Note>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, content, kind, done, created_at FROM notes WHERE id = ?1")
            .context("查询笔记失败")?;

        let result = stmt.query_row(params![id as i64], |row| {
            Ok(Note {
                id: row.get::<_, i64>(0)? as u64,
                content: row.get(1)?,
                kind: {
                    let kind_str: String = row.get(2)?;
                    match kind_str.as_str() {
                        "todo" => NoteKind::Todo,
                        _ => NoteKind::Memo,
                    }
                },
                done: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
            })
        });

        match result {
            Ok(note) => Ok(Some(note)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("查询笔记失败"),
        }
    }

    /// 列出所有笔记，按创建时间倒序，未完成的 Todo 在前。
    pub fn list_all_notes(&self) -> Result<Vec<Note>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, content, kind, done, created_at FROM notes ORDER BY done ASC, created_at DESC",
            )
            .context("查询笔记列表失败")?;

        let notes = stmt
            .query_map([], |row| {
                Ok(Note {
                    id: row.get::<_, i64>(0)? as u64,
                    content: row.get(1)?,
                    kind: {
                        let kind_str: String = row.get(2)?;
                        match kind_str.as_str() {
                            "todo" => NoteKind::Todo,
                            _ => NoteKind::Memo,
                        }
                    },
                    done: row.get::<_, i32>(3)? != 0,
                    created_at: row.get(4)?,
                })
            })
            .context("查询笔记列表失败")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(notes)
    }

    /// 按类型筛选笔记。
    #[allow(dead_code)] // v0.2 笔记 Tab 筛选接入后使用
    pub fn list_notes_by_kind(&self, kind: NoteKind) -> Result<Vec<Note>> {
        let kind_str = match kind {
            NoteKind::Todo => "todo",
            NoteKind::Memo => "memo",
        };

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, content, kind, done, created_at FROM notes WHERE kind = ?1 ORDER BY created_at DESC",
            )
            .context("查询笔记列表失败")?;

        let notes = stmt
            .query_map(params![kind_str], |row| {
                Ok(Note {
                    id: row.get::<_, i64>(0)? as u64,
                    content: row.get(1)?,
                    kind: {
                        let kind_str: String = row.get(2)?;
                        match kind_str.as_str() {
                            "todo" => NoteKind::Todo,
                            _ => NoteKind::Memo,
                        }
                    },
                    done: row.get::<_, i32>(3)? != 0,
                    created_at: row.get(4)?,
                })
            })
            .context("查询笔记列表失败")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(notes)
    }

    /// 获取笔记总数。
    pub fn get_count(&self) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
        Ok(count)
    }

    // ========== 设置相关操作 ==========

    /// 获取单个设置项的值。
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 设置单个配置项。
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// 获取所有扫描目录。
    pub fn get_scan_dirs(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT path FROM scan_dirs ORDER BY id ASC")
            .context("查询扫描目录失败")?;
        let dirs: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(dirs)
    }

    /// 添加扫描目录。
    pub fn add_scan_dir(&self, path: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // 检查是否已存在
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM scan_dirs WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )?;
        if exists > 0 {
            return Ok(()); // 已存在，跳过
        }
        conn.execute("INSERT INTO scan_dirs (path) VALUES (?1)", params![path])?;
        Ok(())
    }

    /// 删除扫描目录。
    pub fn remove_scan_dir(&self, index: usize) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // 获取指定索引的路径
        let path: Option<String> = conn
            .query_row(
                "SELECT path FROM scan_dirs ORDER BY id ASC LIMIT 1 OFFSET ?1",
                params![index as i64],
                |row| row.get(0),
            )
            .ok();
        if let Some(p) = path {
            conn.execute("DELETE FROM scan_dirs WHERE path = ?1", params![p])?;
        }
        Ok(())
    }

    /// 获取所有设置（用于迁移或备份）。
    #[allow(dead_code)] // 备份/调试用；v0.4 数据迁移工具有可能会启用
    pub fn get_all_settings(&self) -> Result<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT key, value FROM settings")
            .context("查询所有设置失败")?;
        let settings: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(settings)
    }
}

/// 获取 SQLite 数据库文件路径。
fn get_db_path() -> Result<PathBuf> {
    let nimbus_dir = app_data_dir();
    std::fs::create_dir_all(&nimbus_dir)?;
    Ok(nimbus_dir.join("nimbus.db"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_db() -> (SqliteDb, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "nimbus_sqlite_test_{:?}_{:?}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let _db_path = temp_dir.join("test.db");

        // 临时替换路径（绕过 app_data_dir）
        std::env::set_var("LOCALAPPDATA", &temp_dir);

        let db = SqliteDb::open().expect("创建测试数据库失败");
        (db, temp_dir)
    }

    #[test]
    fn test_insert_and_get() {
        let (db, _temp_dir) = temp_db();

        let note = db.insert_note("测试内容", NoteKind::Memo).unwrap();
        assert_eq!(note.content, "测试内容");
        assert_eq!(note.kind, NoteKind::Memo);

        let retrieved = db.get_note(note.id).unwrap().unwrap();
        assert_eq!(retrieved.content, "测试内容");
    }

    #[test]
    fn test_toggle_done() {
        let (db, _temp_dir) = temp_db();

        let note = db.insert_note("待办事项", NoteKind::Todo).unwrap();
        assert!(!note.done);

        db.toggle_done(note.id).unwrap();

        let updated = db.get_note(note.id).unwrap().unwrap();
        assert!(updated.done);
    }

    #[test]
    fn test_list_all() {
        let (db, _temp_dir) = temp_db();

        db.insert_note("笔记1", NoteKind::Memo).unwrap();
        db.insert_note("笔记2", NoteKind::Memo).unwrap();
        let todo = db.insert_note("待办", NoteKind::Todo).unwrap();

        // 默认应返回所有笔记
        let notes = db.list_all_notes().unwrap();
        assert_eq!(notes.len(), 3);

        // 切换 todo 完成状态后，应该排在最后
        db.toggle_done(todo.id).unwrap();
        let notes = db.list_all_notes().unwrap();
        assert!(!notes[0].done); // 未完成的在前
    }

    #[test]
    fn test_delete() {
        let (db, _temp_dir) = temp_db();

        let note = db.insert_note("删除测试", NoteKind::Memo).unwrap();
        db.delete_note(note.id).unwrap();

        assert!(db.get_note(note.id).unwrap().is_none());
    }
}