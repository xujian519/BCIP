use codex_tools::FunctionCallError;

use super::retry_config;

/// 判断错误信息是否可重试。
pub fn is_error_retryable(
    policy: &retry_config::ToolRetryPolicy,
    error: &FunctionCallError,
) -> bool {
    let message = match error {
        FunctionCallError::RespondToModel(msg) => msg,
        FunctionCallError::Fatal(msg) => msg,
    };
    policy.should_retry(message)
}

/// 从 FunctionCallError 中提取错误消息。
#[allow(dead_code)]
pub fn error_message(error: &FunctionCallError) -> &str {
    match error {
        FunctionCallError::RespondToModel(msg) | FunctionCallError::Fatal(msg) => msg,
    }
}

/// 构建降级工具的替换错误消息。
#[allow(dead_code)]
pub fn fallback_exhausted_message(tool_name: &str, original_error: &str) -> String {
    format!("工具 {tool_name} 执行失败且重试/降级均已用尽: {original_error}")
}

/// 构建降级工具调用时使用的提示信息。
#[allow(dead_code)]
pub fn fallback_notice(original_tool: &str, fallback_tool: &str, reason: &str) -> String {
    format!("[降级] {original_tool} 失败({reason})，自动切换至 {fallback_tool}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn retryable_timeout_error() {
        let policy = retry_config::ToolRetryPolicy {
            max_retries: 2,
            base_delay: Duration::from_millis(100),
            retry_on: vec![retry_config::RetryableError::Timeout],
            fallback_chain: vec![],
        };
        let err = FunctionCallError::RespondToModel("request timeout".into());
        assert!(is_error_retryable(&policy, &err));
    }

    #[test]
    fn non_retryable_error() {
        let policy = retry_config::ToolRetryPolicy {
            max_retries: 2,
            base_delay: Duration::from_millis(100),
            retry_on: vec![retry_config::RetryableError::Timeout],
            fallback_chain: vec![],
        };
        let err = FunctionCallError::RespondToModel("invalid parameter".into());
        assert!(!is_error_retryable(&policy, &err));
    }

    #[test]
    fn fallback_message_format() {
        let msg = fallback_notice("patent_search", "knowledge_search", "timeout");
        assert!(msg.contains("patent_search"));
        assert!(msg.contains("knowledge_search"));
    }

    #[test]
    fn exhausted_message_format() {
        let msg = fallback_exhausted_message("patent_search", "timeout");
        assert!(msg.contains("patent_search"));
        assert!(msg.contains("timeout"));
    }
}
