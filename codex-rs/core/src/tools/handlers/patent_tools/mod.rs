use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_patent_tools::search_tools::{register_search_tools, ToolHandler};
use codex_tools::ResponsesApiTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde_json::json;
use std::sync::Arc;

/// 将专利工具函数适配为 BCIP ToolExecutor + CoreToolRuntime
pub struct PatentToolHandler {
    name: String,
    spec: ToolSpec,
    handler: ToolHandler,
}

impl PatentToolHandler {
    fn new(name: String, handler: ToolHandler) -> Self {
        let spec = ToolSpec::Function(ResponsesApiTool {
            name: name.clone(),
            description: tool_description(&name),
            strict: false,
            defer_loading: None,
            parameters: serde_json::from_value(tool_parameters(&name)).unwrap_or_default(),
            output_schema: None,
        });
        Self { name, spec, handler }
    }

    pub fn create_all_adapters() -> Vec<Arc<dyn CoreToolRuntime>> {
        let search_tools = register_search_tools();
        let mut adapters: Vec<Arc<dyn CoreToolRuntime>> = Vec::new();
        for (name, handler) in search_tools {
            adapters.push(Arc::new(Self::new(name, handler)));
        }
        adapters
    }
}

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for PatentToolHandler {
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
                    "unsupported payload for patent tool: {}", self.name
                )))
            }
        };

        let args: serde_json::Value = serde_json::from_str(&args_str)
            .map_err(|e| FunctionCallError::RespondToModel(format!("invalid JSON args: {e}")))?;

        match (self.handler)(args).await {
            Ok(value) => Ok(Box::new(FunctionToolOutput::from_text(
                serde_json::to_string(&value).unwrap_or_else(|e| format!("{e}")),
                None,
            ))),
            Err(e) => Err(FunctionCallError::RespondToModel(e)),
        }
    }
}

impl CoreToolRuntime for PatentToolHandler {
    fn matches_kind(&self, _payload: &ToolPayload) -> bool {
        true
    }
}

fn tool_description(name: &str) -> String {
    match name {
        "PatentSearch" => "统一专利检索，支持同义词扩展和跨源搜索".into(),
        "GooglePatentsFetch" => "从 Google Patents 检索专利详细信息".into(),
        "SearchQueryBuilder" => "构建三层渐进式专利检索式（精确→语义→变体）".into(),
        "IterativeSearch" => "多轮迭代式专利检索，每轮扩展同义词".into(),
        "PatentDownload" => "从 Google Patents 下载专利 PDF 原文".into(),
        _ => format!("{name} - 专利工具"),
    }
}

fn tool_parameters(name: &str) -> serde_json::Value {
    match name {
        "PatentSearch" => json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "检索关键词"},
                "limit": {"type": "integer", "description": "返回数量上限", "default": 10},
                "use_synonyms": {"type": "boolean", "description": "使用同义词扩展", "default": true}
            },
            "required": ["query"]
        }),
        "GooglePatentsFetch" => json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "检索关键词或专利号"},
                "limit": {"type": "integer", "default": 10},
                "patent_number": {"type": "string", "description": "具体专利号（可选）"}
            },
            "required": ["query"]
        }),
        "SearchQueryBuilder" => json!({
            "type": "object",
            "properties": {
                "concept": {"type": "string", "description": "检索概念/关键词"},
                "field": {"type": "string", "description": "技术领域限定（可选）"}
            },
            "required": ["concept"]
        }),
        "IterativeSearch" => json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "初始检索词"},
                "rounds": {"type": "integer", "default": 3},
                "limit": {"type": "integer", "default": 10}
            },
            "required": ["query"]
        }),
        "PatentDownload" => json!({
            "type": "object",
            "properties": {
                "patent_number": {"type": "string", "description": "要下载的专利号"}
            },
            "required": ["patent_number"]
        }),
        _ => json!({"type": "object", "properties": {}}),
    }
}
