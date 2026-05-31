use super::TelegramAdapter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BotCommand {
    Start,
    Help,
    NewSession(Option<String>),
    Projects,
    Status,
    Stop,
    Clear,
    Allow(String),
    AlwaysAllow(String),
    Deny(String),
}

pub fn parse_command(input: &str) -> Option<BotCommand> {
    let text = input.trim();

    if text == "/start" {
        return Some(BotCommand::Start);
    }
    if text == "/help" || text == "帮助" {
        return Some(BotCommand::Help);
    }
    if text == "/projects" || text == "项目列表" {
        return Some(BotCommand::Projects);
    }
    if text == "/status" || text == "状态" {
        return Some(BotCommand::Status);
    }
    if text == "/stop" || text == "停止" {
        return Some(BotCommand::Stop);
    }
    if text == "/clear" || text == "清空" {
        return Some(BotCommand::Clear);
    }

    if let Some(rest) = text.strip_prefix("/new ") {
        let project = if rest.is_empty() {
            None
        } else {
            Some(rest.trim().to_string())
        };
        return Some(BotCommand::NewSession(project));
    }
    if text.starts_with("/new") {
        return Some(BotCommand::NewSession(None));
    }

    if let Some(rest) = text.strip_prefix("/allow ") {
        return Some(BotCommand::Allow(rest.trim().to_string()));
    }
    if let Some(rest) = text.strip_prefix("/always ") {
        return Some(BotCommand::AlwaysAllow(rest.trim().to_string()));
    }
    if let Some(rest) = text.strip_prefix("/deny ") {
        return Some(BotCommand::Deny(rest.trim().to_string()));
    }

    None
}

impl TelegramAdapter {
    pub async fn handle_command(&self, chat_id: i64, cmd: &BotCommand) {
        match cmd {
            BotCommand::Start | BotCommand::Help => {
                let text = r#"🤖 BCIP 专利智能助手

命令列表:
/start — 开始对话
/help — 显示帮助
/new [项目名] — 创建新会话
/projects — 查看项目列表
/status — 查看当前状态
/stop — 停止当前生成
/clear — 清空会话上下文
/allow <id> — 批准操作
/deny <id> — 拒绝操作"#;
                self.api.send_message(chat_id, text).await.ok();
            }

            BotCommand::NewSession(project) => {
                let text = match project {
                    Some(p) => format!("正在创建新会话，项目: {p}"),
                    None => "正在创建新会话...".into(),
                };
                self.api.send_message(chat_id, &text).await.ok();
            }

            BotCommand::Projects => {
                self.api.send_message(chat_id, "列出最近项目...").await.ok();
            }

            BotCommand::Status => {
                self.api.send_message(chat_id, "查询当前状态...").await.ok();
            }

            BotCommand::Stop => {
                self.api.send_message(chat_id, "停止当前生成...").await.ok();
                self.bridge
                    .send_message(codex_im_protocol::ClientMessage::StopGeneration)
                    .ok();
            }

            BotCommand::Clear => {
                self.api
                    .send_message(chat_id, "会话上下文已清空。")
                    .await
                    .ok();
            }

            BotCommand::Allow(id) => {
                self.send_permission_response(id, codex_im_protocol::PermissionDecision::Allow)
                    .await;
            }

            BotCommand::AlwaysAllow(id) => {
                self.send_permission_response(
                    id,
                    codex_im_protocol::PermissionDecision::AlwaysAllow,
                )
                .await;
            }

            BotCommand::Deny(id) => {
                self.send_permission_response(id, codex_im_protocol::PermissionDecision::Deny)
                    .await;
            }
        }
    }

    pub async fn handle_callback(&self, _chat_id: i64, callback_id: &str, data: &str) {
        let result = codex_im_common::parse_permission_command(data);

        match result {
            Some(cmd) => {
                let decision = cmd.decision();
                if let Some(request_id) = cmd.request_id() {
                    self.send_permission_response(request_id, decision).await;
                }
                self.api
                    .answer_callback_query(callback_id, "已处理")
                    .await
                    .ok();
            }
            None => {
                self.api
                    .answer_callback_query(callback_id, "未知操作")
                    .await
                    .ok();
            }
        }
    }
}
