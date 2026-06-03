//! 专利无效/复审（Council）域工具。
//!
//! 通过 Python 桥接脚本调用多模型专家委员会（Council）系统，
//! 提供无效宣告分析、质量门控等服务。
//! 使用外部 Python 进程（council_bridge.py）作为执行引擎。

use std::collections::HashMap;

use crate::ToolHandler;

/// 默认 Python 桥接脚本路径（相对于 codex-patent-tools crate 根目录）。
const DEFAULT_BRIDGE_RELATIVE: &str =
    "../../.codex/skills/patent-council/scripts/council_bridge.py";

/// 环境变量可覆盖桥接脚本路径
const BRIDGE_PATH_ENV: &str = "CODEX_PATENT_COUNCIL_BRIDGE";

fn resolve_bridge_path() -> String {
    if let Ok(path) = std::env::var(BRIDGE_PATH_ENV) {
        return path;
    }
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join(DEFAULT_BRIDGE_RELATIVE)
        .to_string_lossy()
        .to_string()
}

/// 通过子进程调用 Python 桥接脚本，传入 JSON 请求，获取 JSON 响应
async fn call_python_bridge(request: serde_json::Value) -> Result<serde_json::Value, String> {
    let bridge_path = resolve_bridge_path();
    let input = serde_json::to_string(&request).map_err(|e| format!("序列化请求失败: {e}"))?;

    let mut child = tokio::process::Command::new("python3")
        .arg(&bridge_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("启动 Python 桥接失败 ({bridge_path}): {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| format!("写入 Python 输入失败: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("等待 Python 进程失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // 即使 exit code 非零，stdout 可能仍有 JSON 错误消息
        if let Ok(err_val) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
            && let Some(err_msg) = err_val.get("error").and_then(|v| v.as_str())
        {
            return Err(format!("Council 引擎错误: {err_msg}"));
        }
        return Err(format!(
            "Python 桥接失败 (exit={}): {stderr}",
            output.status
        ));
    }

    serde_json::from_slice(&output.stdout).map_err(|e| format!("解析 Python 输出失败: {e}"))
}

/// 注册 Council 域工具
pub fn register_council_tools() -> HashMap<String, ToolHandler> {
    let mut t: HashMap<String, ToolHandler> = HashMap::new();

    t.insert("CouncilDeliberate".into(), |input| {
        Box::pin(async move {
            let task = input
                .get("task")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少必填字段: task".to_string())?;
            let models = input.get("models").and_then(|v| v.as_str()).unwrap_or(
                "deepseek-ai/DeepSeek-V3,Qwen/Qwen2.5-72B-Instruct,deepseek-ai/DeepSeek-R1",
            );
            let chairman = input
                .get("chairman")
                .and_then(|v| v.as_str())
                .unwrap_or("Qwen/Qwen2.5-72B-Instruct");
            let criteria = input
                .get("criteria")
                .and_then(|v| v.as_str())
                .unwrap_or("准确性,法律依据,论证深度,完整性");
            let verbose = input
                .get("verbose")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let api_base = input
                .get("api_base")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .or_else(|| std::env::var("OPENAI_BASE_URL").ok());

            let mut config = serde_json::json!({
                "models": models.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>(),
                "chairman": chairman.trim(),
                "criteria": criteria.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>(),
            });
            if let Some(base) = &api_base {
                config["api_base"] = serde_json::json!(base);
            }

            let request = serde_json::json!({
                "action": "deliberate",
                "task": task,
                "config": config,
                "verbose": verbose,
            });

            call_python_bridge(request).await
        })
    });

    t.insert("CouncilQualityGate".into(), |input| {
        Box::pin(async move {
            let document = input
                .get("document")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少必填字段: document".to_string())?;
            let document_type = input
                .get("document_type")
                .and_then(|v| v.as_str())
                .unwrap_or("权利要求书");
            let threshold = input
                .get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.7);
            let models = input.get("models").and_then(|v| v.as_str()).unwrap_or(
                "deepseek-ai/DeepSeek-V3,Qwen/Qwen2.5-72B-Instruct,deepseek-ai/DeepSeek-R1",
            );
            let chairman = input
                .get("chairman")
                .and_then(|v| v.as_str())
                .unwrap_or("Qwen/Qwen2.5-72B-Instruct");
            let api_base = input
                .get("api_base")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .or_else(|| std::env::var("OPENAI_BASE_URL").ok());

            let mut config = serde_json::json!({
                "models": models.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>(),
                "chairman": chairman.trim(),
                "criteria": ["形式规范", "实质内容", "法律合规"],
            });
            if let Some(base) = &api_base {
                config["api_base"] = serde_json::json!(base);
            }

            let request = serde_json::json!({
                "action": "quality_gate",
                "document": document,
                "document_type": document_type,
                "threshold": threshold,
                "config": config,
            });

            call_python_bridge(request).await
        })
    });

    t
}
