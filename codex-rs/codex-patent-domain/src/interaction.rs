use serde::Deserialize;
use serde::Serialize;

const NEGATIVE_PATTERNS: &[&str] = &[
    r"(?i)不对",
    r"(?i)错了",
    r"(?i)不行",
    r"(?i)搞什么",
    r"(?i)怎么搞的",
    r"(?i)wrong",
    r"(?i)incorrect",
    r"(?i)doesn['’]t work",
    r"(?i)doesn['’]t make sense",
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

pub fn is_frustrated(text: &str) -> bool {
    for pattern in NEGATIVE_PATTERNS {
        if let Ok(re) = regex::Regex::new(pattern)
            && re.is_match(text)
        {
            return true;
        }
    }
    false
}

pub fn wants_continue(text: &str) -> bool {
    if is_frustrated(text) {
        return false;
    }
    for pattern in KEEP_GOING_PATTERNS {
        if let Ok(re) = regex::Regex::new(pattern)
            && re.is_match(text)
        {
            return true;
        }
    }
    false
}

pub fn would_upgrade_effort(text: &str) -> Option<EffortLevel> {
    for pattern in ULTRATHINK_PATTERNS {
        if let Ok(re) = regex::Regex::new(pattern)
            && re.is_match(text)
        {
            return Some(EffortLevel::from_user_prompt(text));
        }
    }
    None
}

pub fn response_strategy(text: &str) -> ResponseStrategy {
    if is_frustrated(text) {
        ResponseStrategy::Reassuring
    } else if wants_continue(text) {
        ResponseStrategy::ContinueBrief
    } else {
        ResponseStrategy::Normal
    }
}

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
