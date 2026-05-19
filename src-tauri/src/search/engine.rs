//! 搜索引擎核心。v0.1 实现：自定义模糊匹配 + 拼音支持。
//!
//! 预计算 lowercase + 拼音字段 + 字符向量，支持中英文混合搜索。

use std::path::Path;
use std::sync::RwLock;

use crate::domain::AppEntry;
use crate::search::matcher;
use crate::search::pinyin::{to_initials, to_pinyin};

/// 搜索引擎：持有内存索引，暴露 `search` API。
///
/// 内部使用 `RwLock` 支持运行时动态添加条目（Layer 3 "用过即学"）。
pub struct SearchEngine {
    /// (原始条目, 预计算的搜索字段)
    apps: RwLock<Vec<(AppEntry, SearchFields)>>,
}

/// 预计算的搜索字段。
struct SearchFields {
    /// 小写名称（保留 String 用于 contains 检查）
    lowercase_name: String,
    /// 小写名称的字符向量（避免搜索时重复分配）
    lowercase_chars: Vec<char>,
    /// 拼音全拼的字符向量
    pinyin_full_chars: Vec<char>,
    /// 拼音首字母的字符向量
    pinyin_initials_chars: Vec<char>,
}

/// 从 AppEntry 预计算搜索字段。
fn compute_fields(app: &AppEntry) -> SearchFields {
    let lowercase_name = app.name.to_lowercase();
    let pinyin_full = to_pinyin(&app.name);
    let pinyin_initials = to_initials(&app.name);
    let lowercase_chars: Vec<char> = lowercase_name.chars().collect();
    let pinyin_full_chars: Vec<char> = pinyin_full.chars().collect();
    let pinyin_initials_chars: Vec<char> = pinyin_initials.chars().collect();
    SearchFields {
        lowercase_name,
        lowercase_chars,
        pinyin_full_chars,
        pinyin_initials_chars,
    }
}

impl SearchEngine {
    pub fn new(apps: Vec<AppEntry>) -> Self {
        let apps = apps
            .into_iter()
            .map(|app| {
                let fields = compute_fields(&app);
                (app, fields)
            })
            .collect();
        Self {
            apps: RwLock::new(apps),
        }
    }

    /// 索引里的应用数量（日志/诊断用）。
    pub fn len(&self) -> usize {
        self.apps.read().unwrap_or_else(|e| e.into_inner()).len()
    }

    /// 索引是否为空。
    #[allow(dead_code)] // 诊断/断言用；v0.3 Layer 3 增长后可能移除
    pub fn is_empty(&self) -> bool {
        self.apps
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }

    /// 运行时动态添加条目（Layer 3 "用过即学"）。
    pub fn add(&self, entry: AppEntry) {
        let fields = compute_fields(&entry);
        self.apps
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .push((entry, fields));
    }

    /// 检查索引中是否已有指定路径的条目。
    pub fn contains_path(&self, path: &Path) -> bool {
        self.apps
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .any(|(app, _)| app.path == path)
    }

    /// 按 query 匹配应用名，返回前 `limit` 条结果。
    ///
    /// 匹配策略：
    /// - 空 query / 纯空白 → 空列表
    /// - 模糊匹配（名称、拼音全拼、拼音首字母）
    /// - 按匹配质量排序
    pub fn search(&self, query: &str, limit: usize) -> Vec<AppEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let query_chars: Vec<char> = q.chars().collect();

        // 中文查询 → 提前转拼音，避免在循环内重复转换
        let (query_pinyin, query_pinyin_chars) = if matcher::contains_chinese(&q) {
            let pinyin = to_pinyin(&q);
            if pinyin != q {
                let chars: Vec<char> = pinyin.chars().collect();
                (Some(pinyin), Some(chars))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let apps = self.apps.read().unwrap_or_else(|e| e.into_inner());
        let mut matches: Vec<(AppEntry, u32)> = apps
            .iter()
            .filter_map(|(app, fields)| {
                calculate_match_score(
                    &query_chars,
                    query_pinyin.as_deref(),
                    query_pinyin_chars.as_deref(),
                    fields,
                )
                .map(|score| (app.clone(), score))
            })
            .collect();
        drop(apps);

        // 按匹配分数降序排列
        matches.sort_by_key(|b| std::cmp::Reverse(b.1));

        matches
            .into_iter()
            .take(limit)
            .map(|(app, _)| app)
            .collect()
    }
}

/// 计算匹配分数（委托到 matcher 模块的算法原语）。
/// 返回 Some(score)，分数越高表示匹配越好。
fn calculate_match_score(
    query_chars: &[char],
    query_pinyin: Option<&str>,
    query_pinyin_chars: Option<&[char]>,
    fields: &SearchFields,
) -> Option<u32> {
    use matcher::*;

    let mut best_score: u32 = 0;

    // 1. 严格子序列匹配（所有字符按序出现）
    if let Some(score) = fuzzy_match(query_chars, &fields.lowercase_chars) {
        best_score = best_score.max(score + NAME_MATCH_BONUS);
    }
    if let Some(score) = fuzzy_match(query_chars, &fields.pinyin_full_chars) {
        best_score = best_score.max(score + PINYIN_FULL_BONUS);
    }
    if let Some(score) = fuzzy_match(query_chars, &fields.pinyin_initials_chars) {
        best_score = best_score.max(score + PINYIN_INITIALS_BONUS);
    }

    // 2. 反向拼音匹配：中文查询 → 拼音 → 匹配英文应用名
    if let (Some(query_pinyin), Some(query_pinyin_chars)) = (query_pinyin, query_pinyin_chars) {
        if let Some(score) = fuzzy_match(query_pinyin_chars, &fields.lowercase_chars) {
            best_score = best_score.max(score + REVERSE_PINYIN_BONUS);
        }
        // 目标名包含查询拼音（如 "weixinshurufa" 包含 "weixin"）
        if fields.lowercase_name.contains(query_pinyin) {
            best_score = best_score.max(CONTAINS_BONUS);
        }
        // Smith-Waterman 处理拼音罗马化差异（如 "weixin" vs "wechat"）
        // 跨语言匹配降低阈值，因为 "weixin" vs "wechat" 仅有 "we" 前缀匹配
        if best_score == 0 {
            if let Some(score) = similarity_match(
                query_pinyin_chars,
                &fields.lowercase_chars,
                CROSS_LANG_THRESHOLD,
            ) {
                best_score = best_score.max(score + CROSS_LANG_SIM_BONUS);
            }
        }
    }

    // 3. Smith-Waterman 局部序列比对：处理拼写变体、缩写、罗马化差异
    //    仅在严格匹配无结果时启用，避免过度匹配
    if best_score == 0 {
        if let Some(score) =
            similarity_match(query_chars, &fields.lowercase_chars, GENERAL_SIM_THRESHOLD)
        {
            best_score = best_score.max(score);
        }
    }

    if best_score > 0 {
        Some(best_score)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AppSource;
    use std::path::PathBuf;

    fn make(name: &str) -> AppEntry {
        AppEntry {
            name: name.into(),
            path: PathBuf::from("/fake"),
            source: AppSource::StartMenu,
        }
    }

    #[test]
    fn empty_query_returns_empty() {
        let e = SearchEngine::new(vec![make("Chrome")]);
        assert!(e.search("", 10).is_empty());
        assert!(e.search("   ", 10).is_empty());
    }

    #[test]
    fn fuzzy_match_substring() {
        let e = SearchEngine::new(vec![
            make("Google Chrome"),
            make("Visual Studio Code"),
            make("Firefox"),
        ]);

        let results = e.search("chrome", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(hits.contains(&"Google Chrome"));

        let results = e.search("CODE", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(hits.contains(&"Visual Studio Code"));
    }

    #[test]
    fn fuzzy_match_fuzzy() {
        let e = SearchEngine::new(vec![
            make("Visual Studio Code"),
            make("Visual Studio"),
            make("VS Code"),
        ]);

        // 模糊匹配：vsc 应该匹配 VS Code 和 Visual Studio Code
        let results = e.search("vsc", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(
            hits.iter()
                .any(|h| h.contains("VS") || h.contains("Visual")),
            "vsc 应该匹配 VS Code 或 Visual Studio Code"
        );
    }

    #[test]
    fn limit_caps_results() {
        let e = SearchEngine::new(vec![
            make("App 1"),
            make("App 2"),
            make("App 3"),
            make("App 4"),
        ]);
        assert_eq!(e.search("app", 2).len(), 2);
    }

    #[test]
    fn unicode_chinese_matching() {
        let e = SearchEngine::new(vec![
            make("谷歌浏览器"),
            make("微软办公"),
            make("Adobe Photoshop"),
        ]);

        // 直接中文匹配
        let results = e.search("谷歌", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(hits.contains(&"谷歌浏览器"));

        // 拼音首字母匹配
        let results = e.search("gllq", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(hits.contains(&"谷歌浏览器"), "gllq 应该匹配 谷歌浏览器");
    }

    #[test]
    fn empty_index_returns_empty() {
        let e = SearchEngine::new(vec![]);
        assert!(e.search("anything", 10).is_empty());
    }

    #[test]
    fn special_characters_in_query() {
        let e = SearchEngine::new(vec![make("App (v1.0)"), make("Test*File")]);

        let results = e.search("(v1", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(hits.contains(&"App (v1.0)"));
    }

    #[test]
    fn pinyin_full_match() {
        let e = SearchEngine::new(vec![make("谷歌浏览器"), make("微软办公")]);

        // 拼音全拼匹配
        let results = e.search("guge", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(hits.contains(&"谷歌浏览器"), "guge 应该匹配 谷歌浏览器");
    }

    #[test]
    fn reverse_pinyin_chinese_query_matches_english_name() {
        let e = SearchEngine::new(vec![make("WeChat"), make("微信输入法"), make("WhatsApp")]);

        // 中文查询 "微信" 的拼音是 "weixin"，Smith-Waterman 应能匹配 "WeChat"
        let results = e.search("微信", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(hits.contains(&"微信输入法"), "微信 应直接匹配 微信输入法");
        assert!(hits.contains(&"WeChat"), "微信 应通过序列比对匹配 WeChat");
    }

    #[test]
    fn reverse_pinyin_substring_match() {
        let e = SearchEngine::new(vec![make("WeixinShuruFa"), make("OtherApp")]);

        // "微信" → pinyin "weixin" → 包含在 "weixinshurufa" 中
        let results = e.search("微信", 10);
        let hits: Vec<&str> = results.iter().map(|a| a.name.as_str()).collect();
        assert!(
            hits.contains(&"WeixinShuruFa"),
            "微信 的拼音 weixin 应匹配 WeixinShuruFa"
        );
    }
}