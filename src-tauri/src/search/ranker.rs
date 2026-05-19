//! 频次加权排序器：对搜索引擎的结果做二次排序。
//!
//! 纯函数，不依赖 IO —— 由 bridge 层组装 frequency_map 后传入。

use std::collections::HashMap;

use crate::domain::AppEntry;

/// 对搜索结果按频次加权重排序。
///
/// 算法：`final_score = bonus = 1 + log2(count + 1)`
/// - 无频次记录 → count=0 → bonus=1.0 → 保持原始顺序
/// - 有频次 → bonus > 1 → 乘性提升，对数衰减避免垄断
///
/// # 参数
/// * `results` — 搜索引擎的原始结果（已按文本分数排序）
/// * `frequency_map` — 应用路径 → 启动次数 的映射
pub fn rerank<'a>(
    results: &'a [AppEntry],
    frequency_map: &HashMap<String, u32>,
) -> Vec<&'a AppEntry> {
    let mut scored: Vec<(&AppEntry, f64)> = results
        .iter()
        .map(|app| {
            let count = frequency_map
                .get(app.path.to_string_lossy().as_ref())
                .copied()
                .unwrap_or(0);
            let bonus = 1.0 + (count as f64 + 1.0).log2();
            (app, bonus)
        })
        .collect();

    // 如果所有条目的频次 bonus 都相同，跳过排序
    let needs_sort = scored
        .windows(2)
        .any(|w| (w[0].1 - w[1].1).abs() > f64::EPSILON);

    if needs_sort {
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    scored.into_iter().map(|(app, _)| app).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AppSource;
    use std::path::PathBuf;

    fn make(name: &str, path: &str) -> AppEntry {
        AppEntry {
            name: name.into(),
            path: PathBuf::from(path),
            source: AppSource::StartMenu,
        }
    }

    #[test]
    fn empty_results_returns_empty() {
        let map = HashMap::new();
        let results: Vec<AppEntry> = vec![];
        assert!(rerank(&results, &map).is_empty());
    }

    #[test]
    fn no_frequency_preserves_order() {
        let a = make("A", "/a");
        let b = make("B", "/b");
        let results = vec![a, b];
        let map = HashMap::new();
        let out = rerank(&results, &map);
        assert_eq!(out[0].name, "A");
        assert_eq!(out[1].name, "B");
    }

    #[test]
    fn frequent_app_moves_up() {
        let a = make("A", "/a");
        let b = make("B", "/b");
        let results = vec![a, b];
        let mut map = HashMap::new();
        map.insert("/b".to_string(), 100);
        let out = rerank(&results, &map);
        assert_eq!(out[0].name, "B");
        assert_eq!(out[1].name, "A");
    }

    #[test]
    fn log_dampening_prevents_monopoly() {
        let a = make("A", "/a");
        let b = make("B", "/b");
        let results = vec![a, b];
        let mut map = HashMap::new();
        map.insert("/a".to_string(), 2);
        map.insert("/b".to_string(), 10000);
        let out = rerank(&results, &map);
        assert_eq!(out[0].name, "B");
    }
}