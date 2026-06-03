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
