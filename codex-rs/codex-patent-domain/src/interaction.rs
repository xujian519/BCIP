//! 用户交互意图识别
//!
//! 通过正则模式匹配分析用户输入文本，判断用户情绪（满意/沮丧）、
//! 意图（继续/停止）、以及所需分析深度（普通/深度/超级思考）。
//! 用于 Agent 响应策略的动态调整。

use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::sync::LazyLock;

const NEGATIVE_PATTERNS: &[&str] = &[
    r"(?i)不对",
    r"(?i)错了",
    r"(?i)不行",
    r"(?i)搞什么",
    r"(?i)怎么搞的",
    r"(?i)wrong",
    r"(?i)incorrect",
    r"(?i)doesn['']t work",
    r"(?i)doesn['']t make sense",
];

const KEEP_GOING_PATTERNS: &[&str] = &[
    r"(?i)继续",
    r"(?i)接着",
    r"(?i)然后呢",
    r"(?i)往下",
    r"(?i)下一步",
    r"(?i)continue",
    r"(?i)keep going",
    r"(?i)go on",
    r"(?i)next",
];

const ULTRATHINK_PATTERNS: &[&str] = &[
    r"(?i)ultrathink",
    r"(?i)深度分析",
    r"(?i)超级思考",
    r"(?i)仔细分析",
    r"(?i)深入分析",
];

static NEGATIVE_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    NEGATIVE_PATTERNS
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});

static KEEP_GOING_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    KEEP_GOING_PATTERNS
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});

static ULTRATHINK_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    ULTRATHINK_PATTERNS
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EffortLevel {
    Low,
    #[default]
    Medium,
    High,
    Max,
}

impl EffortLevel {
    pub fn from_user_prompt(prompt: &str) -> Self {
        let lower = prompt.to_lowercase();

        if lower.contains("ultrathink") || lower.contains("超级思考") {
            return Self::Max;
        }
        if lower.contains("深度分析") || lower.contains("深入") || lower.contains("仔细分析")
        {
            return Self::High;
        }

        Self::Medium
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Max => "max",
        }
    }
}

/// 判断用户输入是否表达沮丧/不满情绪
pub fn is_frustrated(text: &str) -> bool {
    NEGATIVE_REGEXES.iter().any(|re| re.is_match(text))
}

/// 判断用户输入是否表达继续执行的意愿
pub fn wants_continue(text: &str) -> bool {
    if is_frustrated(text) {
        return false;
    }
    KEEP_GOING_REGEXES.iter().any(|re| re.is_match(text))
}

/// 判断用户输入是否要求升级分析深度
///
/// 返回建议的 EffortLevel，如果不需要升级则返回 None。
pub fn would_upgrade_effort(text: &str) -> Option<EffortLevel> {
    if ULTRATHINK_REGEXES.iter().any(|re| re.is_match(text)) {
        Some(EffortLevel::from_user_prompt(text))
    } else {
        None
    }
}

/// 根据用户输入选择最合适的响应策略
pub fn response_strategy(text: &str) -> ResponseStrategy {
    if is_frustrated(text) {
        ResponseStrategy::Reassuring
    } else if wants_continue(text) {
        ResponseStrategy::ContinueBrief
    } else {
        ResponseStrategy::Normal
    }
}

/// 响应策略枚举
///
/// 根据用户意图匹配结果，选择合适的 Agent 响应方式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseStrategy {
    Normal,
    Reassuring,
    ContinueBrief,
}

impl ResponseStrategy {
    pub fn prefix_hint(&self) -> Option<&'static str> {
        match self {
            Self::Reassuring => {
                Some("(用户可能感到沮丧，回答需更加谨慎、细致、富有同理心，多解释你的推理过程)")
            }
            Self::ContinueBrief => Some("(用户希望继续执行，保持简洁，直接给出下一步)"),
            Self::Normal => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effort_from_prompt() {
        assert_eq!(
            EffortLevel::from_user_prompt("ultrathink"),
            EffortLevel::Max
        );
        assert_eq!(
            EffortLevel::from_user_prompt("深度分析这个专利"),
            EffortLevel::High
        );
        assert_eq!(EffortLevel::from_user_prompt("你好"), EffortLevel::Medium);
    }

    #[test]
    fn test_is_frustrated() {
        assert!(is_frustrated("不对，你的分析错了"));
        assert!(is_frustrated("wrong, this is incorrect"));
        assert!(!is_frustrated("继续分析"));
    }

    #[test]
    fn test_wants_continue() {
        assert!(wants_continue("继续"));
        assert!(wants_continue("然后呢"));
        assert!(!wants_continue("不对，不要继续"));
    }

    #[test]
    fn test_response_strategy() {
        assert_eq!(response_strategy("不对"), ResponseStrategy::Reassuring);
        assert_eq!(response_strategy("继续"), ResponseStrategy::ContinueBrief);
        assert_eq!(response_strategy("分析这个专利"), ResponseStrategy::Normal);
    }

    #[test]
    fn test_would_upgrade_effort() {
        assert_eq!(would_upgrade_effort("ultrathink"), Some(EffortLevel::Max));
        assert_eq!(would_upgrade_effort("你好"), None);
    }
}
