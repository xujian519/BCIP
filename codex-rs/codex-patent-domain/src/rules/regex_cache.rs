//! 共享正则缓存，消除热路径重复编译。

use regex::Regex;

/// 全局正则缓存。
static REGEX_CACHE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, Regex>>,
> = std::sync::OnceLock::new();

/// 获取或编译正则，编译失败返回 `None`。
pub fn get_or_compile_regex(pattern: &str) -> Option<Regex> {
    let cache = REGEX_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let mut guard = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(re) = guard.get(pattern) {
        return Some(re.clone());
    }
    match Regex::new(pattern) {
        Ok(re) => {
            guard.insert(pattern.to_string(), re.clone());
            Some(re)
        }
        Err(_e) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pattern_returns_regex() {
        let re = get_or_compile_regex(r"\d+").unwrap();
        assert!(re.is_match("123"));
    }

    #[test]
    fn invalid_pattern_returns_none() {
        let result = get_or_compile_regex(r"[invalid");
        assert!(result.is_none());
    }

    #[test]
    fn cache_returns_same_pattern() {
        let re1 = get_or_compile_regex(r"test\d+").unwrap();
        let re2 = get_or_compile_regex(r"test\d+").unwrap();
        assert!(re1.is_match("test42"));
        assert!(re2.is_match("test42"));
    }

    #[test]
    fn empty_pattern_returns_regex() {
        let re = get_or_compile_regex("").unwrap();
        assert!(re.is_match(""));
    }
}
