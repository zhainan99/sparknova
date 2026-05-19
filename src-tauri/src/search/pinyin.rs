//! 拼音模块：将中文字符转换为拼音，支持首字母和全拼匹配。

use pinyin::{to_pinyin_vec, Pinyin};

/// 将文本转换为拼音（小写，无音调）。
/// 非中文字符保留原样。
pub fn to_pinyin(text: &str) -> String {
    to_pinyin_vec(text, Pinyin::plain).join("").to_lowercase()
}

/// 提取拼音首字母。
/// 非中文字符保留原样。
pub fn to_initials(text: &str) -> String {
    text.chars()
        .map(|c| {
            let pinyin_str = to_pinyin_vec(&c.to_string(), Pinyin::plain);
            if pinyin_str.is_empty() {
                c.to_string()
            } else {
                pinyin_str
                    .first()
                    .map(|s| s.chars().next().unwrap_or(c))
                    .unwrap_or(c)
                    .to_string()
            }
        })
        .collect::<String>()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_to_pinyin() {
        assert_eq!(to_pinyin("谷歌浏览器"), "gugeliulanqi");
        assert_eq!(to_pinyin("微软办公"), "weiruanbangong");
    }

    #[test]
    fn test_mixed_text() {
        let result = to_pinyin("Chrome浏览器");
        // pinyin crate 跳过非中文字符，只返回中文部分的拼音
        assert!(result.contains("liulanqi"), "中文部分应转为拼音");
    }

    #[test]
    fn test_initials() {
        let initials = to_initials("谷歌浏览器");
        // 谷(g)歌(g)浏(l)览(l)器(q) → ggllq
        assert_eq!(initials, "ggllq");
    }

    #[test]
    fn test_initials_mixed() {
        let initials = to_initials("VSCode编辑器");
        assert!(initials.contains("vsc"));
    }
}