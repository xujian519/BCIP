use codex_im_protocol::RiskLevel;

pub fn format_permission_request(
    request_id: &str,
    tool_name: &str,
    tool_input: &serde_json::Value,
    risk_level: &RiskLevel,
) -> String {
    let label = codex_im_common::tool_name_label(codex_im_common::Channel::Telegram, tool_name);

    let risk_emoji = match risk_level {
        RiskLevel::Low => "🟢",
        RiskLevel::Medium => "🟡",
        RiskLevel::High => "🟠",
        RiskLevel::Critical => "🔴",
    };

    let detail = match tool_name {
        "Bash" => tool_input["command"]
            .as_str()
            .map(|c| codex_im_common::truncate_text(c, 300))
            .unwrap_or_default(),
        "Write" | "Edit" => tool_input["file_path"]
            .as_str()
            .map(|p| format!("文件: {p}"))
            .unwrap_or_default(),
        _ => codex_im_common::truncate_text(
            &serde_json::to_string(tool_input).unwrap_or_default(),
            200,
        ),
    };

    format!("{risk_emoji} *权限请求* `{request_id}`\n\n{label}\n{detail}\n\n请选择操作:",)
}

pub fn build_permission_keyboard(request_id: &str) -> serde_json::Value {
    serde_json::json!({
        "inline_keyboard": [
            [
                {
                    "text": "✅ 允许",
                    "callback_data": format!("/allow {request_id}")
                },
                {
                    "text": "🔄 始终允许",
                    "callback_data": format!("/always {request_id}")
                },
                {
                    "text": "❌ 拒绝",
                    "callback_data": format!("/deny {request_id}")
                }
            ]
        ]
    })
}
