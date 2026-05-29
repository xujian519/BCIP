use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::registry::CoreToolRuntime;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use std::sync::Arc;

/// 专利工具适配器桩
pub struct PatentToolAdapter;

impl PatentToolAdapter {
    /// 创建所有专利工具的适配器列表
    pub fn create_all_adapters() -> Vec<Arc<dyn CoreToolRuntime>> {
        Vec::new()
    }
}

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for PatentToolAdapter {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("patent_tool_stub")
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::builder()
            .name("patent_tool_stub")
            .description("Patent tool stub - not implemented")
            .build()
            .expect("valid tool spec")
    }

    async fn handle(
        &self,
        _invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        Err(FunctionCallError::RespondToModel(
            "patent tools not yet integrated - use codex_patent_tools library directly".to_string(),
        ))
    }
}

impl CoreToolRuntime for PatentToolAdapter {}
