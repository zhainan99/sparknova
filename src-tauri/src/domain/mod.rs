//! 领域模型：跨模块共享的数据结构（对应 Spring Boot Entity/DTO）。
//!
//! 本层禁止 IO，禁止依赖 UI / slint / redb。
//! 新数据结构先落在这里，再被 indexer / search / storage / ui 引用。

mod app_entry;
mod note;

pub use app_entry::{AppEntry, AppSource};
pub use note::{Note, NoteKind};