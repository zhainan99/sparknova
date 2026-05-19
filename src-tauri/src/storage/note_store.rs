//! 笔记持久化存储：使用 SQLite 数据库。
//!
//! 迁移自 redb 版本，提供更强的查询能力（按类型筛选、按日期范围等）。

use anyhow::Result;
use tracing::info;

use crate::domain::{Note, NoteKind};
use crate::storage::sqlite_db::SqliteDb;

/// NoteStore 封装，提供和之前 redb 版本相同的 API 接口。
#[derive(Clone)]
pub struct NoteStore {
    sqlite: SqliteDb,
}

impl NoteStore {
    /// 创建新的 NoteStore。
    pub fn new(sqlite: SqliteDb) -> Self {
        Self { sqlite }
    }

    /// 从 redb 迁移数据到 SQLite。
    ///
    /// 如果 SQLite 中已有数据（迁移过），则跳过。
    /// 此方法主要用于首次迁移，需要在 Storage 初始化时调用。
    #[allow(dead_code)] // 从 redb 到 SQLite 的一键迁移；v0.2 确认无旧数据后移除
    pub fn migrate_from_redb(&self, entries: Vec<Note>) -> Result<()> {
        // 如果 SQLite 中已有数据，跳过迁移
        if self.sqlite.get_count().unwrap_or(0) > 0 {
            info!("SQLite 中已有笔记数据，跳过迁移");
            return Ok(());
        }

        let count = entries.len();
        // 批量插入
        for note in entries {
            // 直接使用 SQLite 的 insert_note 方法创建笔记
            // 注意：这里我们重新创建，因为 SQLite 会分配新的 ID
            let _ = self.sqlite.insert_note(&note.content, note.kind);
        }

        info!(count = count, "从 redb 迁移笔记数据完成");
        Ok(())
    }

    /// 创建新笔记，自动分配递增 ID。返回完整 Note。
    pub fn insert(&self, content: &str, kind: NoteKind) -> Result<Note> {
        self.sqlite.insert_note(content, kind)
    }

    /// 更新笔记内容。不存在则静默跳过。
    #[allow(dead_code)] // v0.2 笔记编辑 UI 接入后使用
    pub fn update(&self, id: u64, content: &str) -> Result<()> {
        self.sqlite.update_note(id, content)
    }

    /// 切换 Todo 完成状态。
    pub fn toggle_done(&self, id: u64) -> Result<()> {
        self.sqlite.toggle_done(id)
    }

    /// 删除笔记。
    pub fn delete(&self, id: u64) -> Result<()> {
        self.sqlite.delete_note(id)
    }

    /// 获取单条笔记。
    pub fn get(&self, id: u64) -> Result<Option<Note>> {
        self.sqlite.get_note(id)
    }

    /// 列出所有笔记，按创建时间倒序。完成的 Todo 沉底。
    pub fn list_all(&self) -> Result<Vec<Note>> {
        self.sqlite.list_all_notes()
    }

    /// 按类型筛选笔记。
    #[allow(dead_code)] // v0.2 笔记 Tab 筛选 UI 接入后使用
    pub fn list_by_kind(&self, kind: NoteKind) -> Result<Vec<Note>> {
        self.sqlite.list_notes_by_kind(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_db() -> (NoteStore, std::path::PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "nimbus_note_test_{:?}_{:?}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        // 临时替换路径
        std::env::set_var("LOCALAPPDATA", &temp_dir);

        let sqlite = SqliteDb::open().expect("创建测试数据库失败");
        let store = NoteStore::new(sqlite);
        (store, temp_dir)
    }

    #[test]
    fn test_insert_todo() {
        let (store, _temp_dir) = temp_db();

        let note = store.insert("买牛奶", NoteKind::Todo).unwrap();
        assert_eq!(note.content, "买牛奶");
        assert_eq!(note.kind, NoteKind::Todo);
        assert!(!note.done);
    }

    #[test]
    fn test_insert_memo() {
        let (store, _temp_dir) = temp_db();

        let note = store.insert("这是一个备忘录", NoteKind::Memo).unwrap();
        assert_eq!(note.content, "这是一个备忘录");
        assert_eq!(note.kind, NoteKind::Memo);
    }

    #[test]
    fn test_toggle_done() {
        let (store, _temp_dir) = temp_db();

        let note = store.insert("待办事项", NoteKind::Todo).unwrap();
        store.toggle_done(note.id).unwrap();

        let updated = store.get(note.id).unwrap().unwrap();
        assert!(updated.done);
    }

    #[test]
    fn test_delete() {
        let (store, _temp_dir) = temp_db();

        let note = store.insert("删除测试", NoteKind::Memo).unwrap();
        store.delete(note.id).unwrap();

        assert!(store.get(note.id).unwrap().is_none());
    }

    #[test]
    fn test_list_all() {
        let (store, _temp_dir) = temp_db();

        store.insert("笔记1", NoteKind::Memo).unwrap();
        store.insert("待办1", NoteKind::Todo).unwrap();
        let done_todo = store.insert("已完成", NoteKind::Todo).unwrap();

        // 切换完成状态
        store.toggle_done(done_todo.id).unwrap();

        let notes = store.list_all().unwrap();
        assert_eq!(notes.len(), 3);

        // 已完成的 Todo 应该在最后
        assert!(!notes[0].done || notes[0].kind == NoteKind::Memo);
    }

    #[test]
    fn test_list_by_kind() {
        let (store, _temp_dir) = temp_db();

        store.insert("备忘录1", NoteKind::Memo).unwrap();
        store.insert("备忘录2", NoteKind::Memo).unwrap();
        store.insert("待办1", NoteKind::Todo).unwrap();

        let memos = store.list_by_kind(NoteKind::Memo).unwrap();
        assert_eq!(memos.len(), 2);
    }
}