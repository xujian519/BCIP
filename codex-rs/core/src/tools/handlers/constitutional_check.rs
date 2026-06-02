use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde_json::Value;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

/// 宪法规则检查工具 — 检查工具调用是否违反专利法规则
pub struct ConstitutionalCheckHandler {
    name: String,
    spec: ToolSpec,
    engine: codex_patent_constitutional::ConstitutionalEngine,
}

impl ConstitutionalCheckHandler {
    pub fn new() -> Option<Self> {
        let assets_dir = Self::find_assets_dir()?;
        let rules_dir = assets_dir.join("constitutional");
        if !rules_dir.is_dir() {
            tracing::warn!(
                "constitutional rules dir not found: {}",
                rules_dir.display()
            );
            return None;
        }

        let rules = codex_patent_constitutional::RuleLoader::load_dir(&rules_dir).ok()?;
        if rules.is_empty() {
            tracing::warn!(
                "no constitutional rules loaded from {}",
                rules_dir.display()
            );
            return None;
        }

        tracing::info!(
            "loaded {} constitutional rule files from {}",
            rules.len(),
            rules_dir.display()
        );

        let engine = codex_patent_constitutional::ConstitutionalEngine::new(rules);

        let spec = ToolSpec::Function(ResponsesApiTool {
            name: "ConstitutionalCheck".to_string(),
            description: "检查专利工具调用是否符合专利法及各阶段法律约束。支持两种模式：explicit（指定工具名+文本进行逐条检查）和 auto（自动扫描当前阶段所有工具）。在调用专利工具前后调用此工具进行合规检查。".to_string(),
            strict: false,
            defer_loading: None,
            parameters: serde_json::from_value(tool_parameters()).unwrap_or_default(),
            output_schema: None,
        });

        Some(Self {
            name: "ConstitutionalCheck".to_string(),
            spec,
            engine,
        })
    }

    pub fn create_checkers() -> Vec<Arc<dyn CoreToolRuntime>> {
        match Self::new() {
            Some(handler) => vec![Arc::new(handler)],
            None => {
                tracing::warn!("ConstitutionalCheck not available (rules not found)");
                Vec::new()
            }
        }
    }

    fn find_assets_dir() -> Option<PathBuf> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut candidates = vec![
            manifest_dir.join("../codex-patent-assets"),
            manifest_dir.join("../../codex-patent-assets"),
            PathBuf::from("codex-patent-assets"),
        ];

        if let Some(exe_dir) = std::env::current_exe()
            .ok()
            .and_then(|e| e.canonicalize().ok())
            .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        {
            candidates.push(exe_dir.join("../lib/codex-patent-assets"));
            candidates.push(exe_dir.join("../share/codex-patent-assets"));
            candidates.push(exe_dir.join("codex-patent-assets"));
        }

        if let Ok(home) = std::env::var("BCIP_HOME").or_else(|_| std::env::var("CODEX_HOME")) {
            let home = PathBuf::from(home);
            candidates.push(home.join("assets/codex-patent-assets"));
            candidates.push(home.join("codex-patent-assets"));
        }

        for candidate in &candidates {
            if candidate.join("constitutional").is_dir() {
                return Some(
                    candidate
                        .canonicalize()
                        .ok()
                        .unwrap_or_else(|| candidate.clone()),
                );
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for ConstitutionalCheckHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(&self.name)
    }

    fn spec(&self) -> ToolSpec {
        self.spec.clone()
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let args_str = match &invocation.payload {
            ToolPayload::Function { arguments } => arguments.clone(),
            _ => {
                return Err(FunctionCallError::RespondToModel(format!(
                    "unsupported payload for constitutional check"
                )));
            }
        };

        let args: Value = serde_json::from_str(&args_str)
            .map_err(|e| FunctionCallError::RespondToModel(format!("invalid JSON args: {e}")))?;

        let mode = args
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("explicit");
        let phase = args.get("phase").and_then(Value::as_str).unwrap_or("撰写");

        let response = match mode {
            "auto" => {
                // 自动扫描模式：列出当前阶段所有工具的适用规则
                let known_phases = self.engine.known_phases();
                let scan_results = self.engine.auto_scan_for_phase(phase);
                let rules_context = self.engine.rules_context_for_phase(phase);

                json!({
                    "mode": "auto",
                    "phase": phase,
                    "known_phases": known_phases,
                    "scanned_tools": scan_results.iter().map(|t| json!({
                        "tool_name": t.tool_name,
                        "active_rules": t.active_rules.iter().map(|r| json!({
                            "rule_id": r.rule_id,
                            "rule_name": r.rule_name,
                            "action": format!("{:?}", r.action),
                            "severity": format!("{:?}", r.severity),
                            "legal_basis": r.legal_basis,
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                    "rules_context": rules_context,
                })
            }
            _ => {
                // 显式检查模式（向后兼容）
                let tool_name = args.get("tool_name").and_then(Value::as_str).unwrap_or("");
                let input_text = args.get("input_text").and_then(Value::as_str).unwrap_or("");
                let output_text = args.get("output_text").and_then(Value::as_str);

                if tool_name.is_empty() {
                    return Ok(Box::new(FunctionToolOutput::from_text(
                        serde_json::to_string(&json!({
                            "error": "explicit 模式需要提供 tool_name 和 input_text",
                            "hint": "使用 mode: \"auto\" 来扫描当前阶段所有工具的合规规则，无需指定 tool_name"
                        })).unwrap_or_default(),
                        None,
                    )));
                }

                let results = self
                    .engine
                    .check_all(tool_name, input_text, output_text, phase);

                let failures: Vec<&codex_patent_constitutional::RuleCheckResult> =
                    results.iter().filter(|r| !r.passed).collect();

                let passed_count = results.len() - failures.len();

                json!({
                    "mode": "explicit",
                    "phase": phase,
                    "summary": {
                        "total_checks": results.len(),
                        "passed": passed_count,
                        "failed": failures.len(),
                        "has_violations": !failures.is_empty(),
                    },
                    "results": results.iter().map(|r| json!({
                        "rule_id": r.rule_id,
                        "rule_name": r.rule_name,
                        "severity": format!("{:?}", r.severity),
                        "action": format!("{:?}", r.action),
                        "legal_basis": r.legal_basis,
                        "passed": r.passed,
                        "details": r.details,
                        "confidence": r.confidence,
                    })).collect::<Vec<_>>(),
                    "blocking_violations": failures.iter()
                        .filter(|r| matches!(r.action, codex_patent_constitutional::RuleAction::Block))
                        .map(|r| json!({
                            "rule_id": r.rule_id,
                            "rule_name": r.rule_name,
                            "legal_basis": r.legal_basis,
                            "details": r.details,
                        }))
                        .collect::<Vec<_>>(),
                    "rules_context": self.engine.rules_context_for_phase(phase),
                })
            }
        };

        Ok(Box::new(FunctionToolOutput::from_text(
            serde_json::to_string(&response).unwrap_or_else(|e| format!("{e}")),
            None,
        )))
    }
}

impl CoreToolRuntime for ConstitutionalCheckHandler {
    fn matches_kind(&self, _payload: &ToolPayload) -> bool {
        true
    }
}

fn tool_parameters() -> Value {
    json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "enum": ["explicit", "auto"],
                "description": "检查模式：explicit（指定工具+文本逐条检查，默认）或 auto（自动扫描当前阶段所有工具）",
                "default": "explicit"
            },
            "tool_name": {
                "type": "string",
                "description": "要检查的工具名称，如 claim_generator, specification_drafter 等（mode=explicit 时必需）"
            },
            "input_text": {
                "type": "string",
                "description": "要检查的输入或输出文本内容（mode=explicit 时必需）"
            },
            "phase": {
                "type": "string",
                "description": "专利生命周期阶段: 申请前/撰写/审查/答复/无效/维权",
                "default": "撰写"
            },
            "output_text": {
                "type": "string",
                "description": "（可选）工具输出文本，用于输出后检查"
            }
        }
    })
}
