//! 笔记领域模型：跨模块共享的笔记数据结构。

use serde::{Deserialize, Serialize};

/// 笔记类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteKind {
    Todo,
    Memo,
}

/// 单条笔记。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// 自增主键（redb 端分配）。
    pub id: u64,
    /// 笔记正文。
    pub content: String,
    /// 类型。
    pub kind: NoteKind,
    /// Todo 专用：是否已完成。
    pub done: bool,
    /// Unix 时间戳（秒），创建时间。
    pub created_at: i64,
}