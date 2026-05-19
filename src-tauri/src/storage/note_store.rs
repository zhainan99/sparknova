//! 笔记持久化存储：使用 SQLite 数据库。

#![allow(dead_code)]

use anyhow::Result;
use tracing::info;

use crate::domain::{Note, NoteKind};
use crate::storage::sqlite_db::SqliteDb;

#[derive(Clone)]
pub struct NoteStore {
    sqlite: SqliteDb,
}

impl NoteStore {
    pub fn new(sqlite: SqliteDb) -> Self {
        Self { sqlite }
    }

    pub fn migrate_from_redb(&self, entries: Vec<Note>) -> Result<()> {
        if self.sqlite.get_count().unwrap_or(0) > 0 {
            info!("SQLite 中已有笔记数据，跳过迁移");
            return Ok(());
        }

        let count = entries.len();
        for note in entries {
            let _ = self.sqlite.insert_note(&note.content, note.kind);
        }

        info!(count = count, "从 redb 迁移笔记数据完成");
        Ok(())
    }

    pub fn insert(&self, content: &str, kind: NoteKind) -> Result<Note> {
        self.sqlite.insert_note(content, kind)
    }

    pub fn update(&self, id: u64, content: &str) -> Result<()> {
        self.sqlite.update_note(id, content)
    }

    pub fn toggle_done(&self, id: u64) -> Result<()> {
        self.sqlite.toggle_done(id)
    }

    pub fn delete(&self, id: u64) -> Result<()> {
        self.sqlite.delete_note(id)
    }

    pub fn get(&self, id: u64) -> Result<Option<Note>> {
        self.sqlite.get_note(id)
    }

    pub fn list_all(&self) -> Result<Vec<Note>> {
        self.sqlite.list_all_notes()
    }

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
            std::thread::current::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

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

        store.toggle_done(done_todo.id).unwrap();

        let notes = store.list_all().unwrap();
        assert_eq!(notes.len(), 3);

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