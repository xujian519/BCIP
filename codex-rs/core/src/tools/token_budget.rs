use codex_tools::ToolSpec;
use codex_utils_output_truncation::approx_token_count;
use tracing::warn;

/// 估算单个工具 spec 的 token 数。
pub fn estimate_tool_spec_tokens(spec: &ToolSpec) -> usize {
    let json = serde_json::to_string(spec).unwrap_or_default();
    approx_token_count(&json)
}

/// 计算一组 spec 的总 token 数。
pub fn total_spec_tokens(specs: &[ToolSpec]) -> usize {
    specs.iter().map(estimate_tool_spec_tokens).sum()
}

/// 检查并输出 token 预算警告。
pub fn check_token_budget(specs: &[ToolSpec], budget: usize, agent_role: Option<&str>) {
    let total = total_spec_tokens(specs);
    if total > budget {
        let role_str = agent_role.unwrap_or("default");
        warn!(
            total_tokens = total,
            budget = budget,
            agent_role = role_str,
            tool_count = specs.len(),
            "tool spec tokens exceed budget"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_tools::JsonSchema;
    use codex_tools::ResponsesApiTool;
    use std::collections::BTreeMap;

    fn make_function_spec(name: &str, desc: &str) -> ToolSpec {
        ToolSpec::Function(ResponsesApiTool {
            name: name.to_string(),
            description: desc.to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(BTreeMap::new(), None, None),
            output_schema: None,
        })
    }

    #[test]
    fn estimate_tokens_for_spec() {
        let spec = make_function_spec("test_tool", "A test tool for unit testing");
        let tokens = estimate_tool_spec_tokens(&spec);
        assert!(tokens > 0, "should estimate some tokens for a tool spec");
    }

    #[test]
    fn total_tokens_empty() {
        assert_eq!(total_spec_tokens(&[]), 0);
    }

    #[test]
    fn total_tokens_multiple_specs() {
        let spec1 = make_function_spec("tool_a", "Tool A");
        let spec2 = make_function_spec("tool_b", "Tool B");
        let total = total_spec_tokens(&[spec1.clone(), spec2.clone()]);
        let individual = estimate_tool_spec_tokens(&spec1) + estimate_tool_spec_tokens(&spec2);
        assert_eq!(total, individual);
    }
}
