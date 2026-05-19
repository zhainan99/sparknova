//! 搜索域：把 query 字符串变成排序后的 AppEntry 列表。
//! 对应 Spring Boot 的 Service 层 —— 禁止依赖 slint::/redb::。
//!
//! 演进路线（参见 docs/design.md §四 v0.1 关键决策）：
//! - v0.1 Step 2：`engine` 最小实现，小写 contains 子串匹配
//! - v0.1 Step 3：升级模糊匹配算法
//! - v0.1 Step 4：补 `pinyin` 模块做拼音首字母/全拼
//! - v0.1 Step 5：补 `ranker` 模块做频次 + 时间衰减加权

mod engine;
pub mod matcher;
mod pinyin;
mod ranker;

pub use engine::SearchEngine;
pub use ranker::rerank;