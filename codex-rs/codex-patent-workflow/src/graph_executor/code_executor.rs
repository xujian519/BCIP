/// 代码执行器 trait — 允许工作流执行任意编程语言代码
pub trait CodeExecutor: Send {
    fn execute(&mut self, language: &str, code: &str) -> Result<CodeExecutionResult, String>;
}

/// 代码执行结果
#[derive(Debug, Clone)]
pub struct CodeExecutionResult {
    pub output: String,
    pub language: String,
    pub success: bool,
    pub error: Option<String>,
}

/// 空操作代码执行器（测试/禁用时使用）
pub struct NoopCodeExecutor;

impl CodeExecutor for NoopCodeExecutor {
    fn execute(&mut self, language: &str, code: &str) -> Result<CodeExecutionResult, String> {
        Ok(CodeExecutionResult {
            output: format!("[NOOP] 执行了 {} 代码，长度={}", language, code.len()),
            language: language.to_string(),
            success: true,
            error: None,
        })
    }
}
