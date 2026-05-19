//! 匹配算法原语：子序列模糊匹配、Smith-Waterman 局部序列比对。
//!
//! 从 engine.rs 拆分出纯函数，方便独立测试和维护。

// ── 评分权重常量 ──
/// 名称严格子序列匹配 bonus。
pub const NAME_MATCH_BONUS: u32 = 100;
/// 拼音全拼匹配 bonus。
pub const PINYIN_FULL_BONUS: u32 = 50;
/// 拼音首字母匹配 bonus。
pub const PINYIN_INITIALS_BONUS: u32 = 75;
/// 反向拼音匹配 bonus（中文查询转拼音后匹配英文名）。
pub const REVERSE_PINYIN_BONUS: u32 = 60;
/// 目标名包含完整查询拼音的 bonus（极高置信度）。
pub const CONTAINS_BONUS: u32 = 1150;
/// 跨语言 Smith-Waterman 匹配 bonus。
pub const CROSS_LANG_SIM_BONUS: u32 = 40;
/// 跨语言 Smith-Waterman 归一化阈值（宽松）。
pub const CROSS_LANG_THRESHOLD: u32 = 25;
/// 通用 Smith-Waterman 归一化阈值（严格）。
pub const GENERAL_SIM_THRESHOLD: u32 = 50;

// ── fuzzy_match 评分常量 ──
/// 匹配率缩放因子。
const MATCH_RATIO_SCALE: u32 = 100;
/// 连续匹配 bonus 乘数。
const CONSECUTIVE_MULTIPLIER: u32 = 10;
/// 早期位置奖励基数。
const BASE_SCORE: u32 = 1000;
/// 首匹配位置惩罚乘数。
const POSITION_PENALTY: u32 = 10;

// ── Smith-Waterman 算法参数 ──
/// 字符匹配得分。
const SW_MATCH: i32 = 3;
/// 字符错配罚分。
const SW_MISMATCH: i32 = -1;
/// 空位罚分。
const SW_GAP: i32 = -1;
/// 最少匹配字符数（低于此值视为噪音）。
const SW_MIN_MATCHES: i32 = 2;

/// 检查文本是否包含中文字符。
pub fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
}

/// 严格子序列匹配：所有 query 字符必须按序出现在 target 中。
/// 返回匹配分数，分数越高越好。
pub fn fuzzy_match(query_chars: &[char], target_chars: &[char]) -> Option<u32> {
    if query_chars.is_empty() {
        return None;
    }

    let mut query_idx = 0;
    let mut target_idx = 0;
    let mut consecutive_matches = 0;
    let mut total_matches = 0;
    let mut first_match_pos = None;

    while query_idx < query_chars.len() && target_idx < target_chars.len() {
        let qc = query_chars[query_idx];
        let tc = target_chars[target_idx];

        if qc == tc {
            if first_match_pos.is_none() {
                first_match_pos = Some(target_idx);
            }
            total_matches += 1;
            consecutive_matches += 1;
            query_idx += 1;
        } else if consecutive_matches > 0 {
            consecutive_matches = 0;
        }
        target_idx += 1;
    }

    if query_idx == query_chars.len() {
        let first_pos = first_match_pos.unwrap_or(0) as u32;
        let match_ratio = (total_matches * MATCH_RATIO_SCALE as usize / query_chars.len()) as u32;
        let consecutive_bonus = consecutive_matches as u32 * CONSECUTIVE_MULTIPLIER;
        let score = match_ratio
            + consecutive_bonus
            + BASE_SCORE.saturating_sub(first_pos * POSITION_PENALTY);
        Some(score)
    } else {
        None
    }
}

/// Smith-Waterman 局部序列比对。
///
/// 生物信息学经典算法，用于在两条序列中找到最优局部对齐。
/// 容忍字符缺失、插入和替换，自动处理罗马化差异（如 "weixin" vs "wechat"）。
///
/// 评分参数：
/// - 匹配: +3  |  错配: -1  |  空位: -1
///
/// `threshold`：归一化分数的最低阈值 (0-100)。跨语言匹配场景可降低阈值（如 25），通用匹配保持较高阈值（如 50）。
pub fn similarity_match(
    query_chars: &[char],
    target_chars: &[char],
    threshold: u32,
) -> Option<u32> {
    if query_chars.is_empty() || target_chars.is_empty() {
        return None;
    }

    let rows = query_chars.len() + 1;
    let cols = target_chars.len() + 1;
    let mut prev = vec![0i32; cols];
    let mut max_score = 0i32;

    for i in 1..rows {
        let mut curr = vec![0i32; cols];
        for j in 1..cols {
            let s = if query_chars[i - 1] == target_chars[j - 1] {
                SW_MATCH
            } else {
                SW_MISMATCH
            };

            curr[j] = (prev[j - 1] + s)
                .max(prev[j] + SW_GAP)
                .max(curr[j - 1] + SW_GAP)
                .max(0);

            max_score = max_score.max(curr[j]);
        }
        prev = curr;
    }

    // 至少匹配 N 个字符，避免单字符误匹配（如 "guge" 匹配 "Adobe Photoshop" 中的 'e'）
    if max_score < SW_MATCH * SW_MIN_MATCHES {
        return None;
    }

    let max_possible = (query_chars.len() as i32).saturating_mul(SW_MATCH);
    if max_possible == 0 {
        return None;
    }
    let normalized = (max_score as f64 / max_possible as f64 * 100.0) as u32;

    if normalized >= threshold {
        Some(normalized)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn similarity_match_basic() {
        // 完全相同 → 应匹配
        assert_eq!(
            similarity_match(&chars("chrome"), &chars("chrome"), GENERAL_SIM_THRESHOLD),
            Some(100)
        );
        // "weixin" vs "wechat"：共享 "we" 前缀，跨语言匹配阈值应匹配
        let score = similarity_match(&chars("weixin"), &chars("wechat"), CROSS_LANG_THRESHOLD);
        assert!(score.is_some(), "weixin vs wechat 应匹配, got {:?}", score);
        // 通用匹配阈值下不应匹配（33% < 50%）
        assert!(
            similarity_match(&chars("weixin"), &chars("wechat"), GENERAL_SIM_THRESHOLD).is_none()
        );
        // 完全不相关 → 不应匹配
        assert!(
            similarity_match(&chars("weixin"), &chars("whatsapp"), CROSS_LANG_THRESHOLD).is_none()
        );
    }
}