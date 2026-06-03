mod api;

use std::sync::Arc;

use codex_im_bridge::ImBridge;
use codex_im_protocol::ClientMessage;
use codex_im_protocol::ServerMessage;
use tokio::sync::Mutex;
use tracing::error;
use tracing::info;

use api::DingtalkApi;

#[derive(Debug)]
pub struct DingtalkConfig {
    pub app_key: String,
    pub app_secret: String,
    pub allowed_users: Vec<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DingtalkAdapter {
    api: DingtalkApi,
    bridge: Arc<ImBridge>,
    config: DingtalkConfig,
    access_token: Arc<Mutex<Option<String>>>,
}

impl DingtalkAdapter {
    pub fn new(config: DingtalkConfig, bridge: Arc<ImBridge>) -> Self {
        let client = reqwest::Client::new();
        let api = DingtalkApi::new(client, config.app_key.clone(), config.app_secret.clone());

        Self {
            api,
            bridge,
            config,
            access_token: Arc::new(Mutex::new(None)),
        }
    }

    /// Run the DingTalk adapter.
    ///
    /// Note: DingTalk's robot API primarily uses webhook (outgoing) for inbound messages.
    /// This polling mode is limited — for production use, configure the DingTalk outgoing
    /// robot webhook to forward events to the BCIP server.
    pub async fn run(&self) -> ! {
        info!("钉钉适配器启动");

        loop {
            match self.ensure_token().await {
                Ok(_) => {
                    // DingTalk inbound requires webhook; polling is not available.
                    // This loop keeps the token fresh and logs readiness.
                }
                Err(e) => {
                    error!(%e, "获取钉钉 Token 失败");
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    /// Ensure a valid access token is available.
    pub async fn ensure_token(&self) -> Result<String, String> {
        {
            let token = self.access_token.lock().await;
            if token.is_some() {
                return Ok(token.clone().unwrap());
            }
        }
        let new_token = self.api.get_access_token().await?;
        *self.access_token.lock().await = Some(new_token.clone());
        Ok(new_token)
    }

    /// Handle an incoming DingTalk outgoing robot message.
    ///
    /// Call this from the webhook handler when a DingTalk event is received.
    pub async fn handle_webhook_message(
        &self,
        conversation_id: &str,
        sender_id: &str,
        text: &str,
    ) -> Result<(), String> {
        // Check allowed users
        if !self.config.allowed_users.is_empty()
            && !self.config.allowed_users.contains(&sender_id.to_string())
        {
            return Ok(()); // Silently ignore unauthorized users
        }

        if let Some(command) = self.parse_command(text) {
            let token = self.ensure_token().await?;
            self.handle_command(&token, conversation_id, sender_id, &command)
                .await;
            return Ok(());
        }

        if text.is_empty() {
            return Ok(());
        }

        let user_msg = ClientMessage::UserMessage {
            text: text.to_string(),
            attachments: None,
        };

        match self.bridge.send_message(user_msg) {
            Ok(()) => {
                let mut event_rx = self.bridge.subscribe();
                let token = self.ensure_token().await?;
                self.stream_response(&token, conversation_id, &mut event_rx)
                    .await;
                Ok(())
            }
            Err(e) => {
                error!(%e, "发送消息到 Bridge 失败");
                let token = self.ensure_token().await?;
                self.api
                    .send_message(&token, conversation_id, "发送消息失败，请重试。")
                    .await
                    .ok();
                Err(e.to_string())
            }
        }
    }

    fn parse_command(&self, text: &str) -> Option<DingtalkCommand> {
        let trimmed = text.trim();

        if trimmed == "新会话" || trimmed == "/new" {
            return Some(DingtalkCommand::NewSession(None));
        }
        if let Some(rest) = trimmed
            .strip_prefix("新会话 ")
            .or_else(|| trimmed.strip_prefix("/new "))
        {
            return Some(DingtalkCommand::NewSession(Some(rest.trim().to_string())));
        }
        if trimmed == "帮助" || trimmed == "/help" {
            return Some(DingtalkCommand::Help);
        }
        if trimmed == "状态" || trimmed == "/status" {
            return Some(DingtalkCommand::Status);
        }
        if trimmed == "停止" || trimmed == "/stop" {
            return Some(DingtalkCommand::Stop);
        }
        if trimmed == "清空" || trimmed == "/clear" {
            return Some(DingtalkCommand::Clear);
        }

        None
    }

    async fn handle_command(
        &self,
        token: &str,
        conversation_id: &str,
        _sender_id: &str,
        cmd: &DingtalkCommand,
    ) {
        match cmd {
            DingtalkCommand::Help => {
                let text = "BCIP 专利智能助手\n\n命令: 新会话 / 帮助 / 状态 / 停止 / 清空";
                self.api.send_message(token, conversation_id, text).await.ok();
            }
            DingtalkCommand::NewSession(project) => {
                let text = match project {
                    Some(p) => format!("正在创建新会话，项目: {p}"),
                    None => "正在创建新会话...".into(),
                };
                self.api
                    .send_message(token, conversation_id, &text)
                    .await
                    .ok();
            }
            DingtalkCommand::Status => {
                self.api
                    .send_message(token, conversation_id, "查询当前状态...")
                    .await
                    .ok();
            }
            DingtalkCommand::Stop => {
                self.api
                    .send_message(token, conversation_id, "停止当前生成...")
                    .await
                    .ok();
                self.bridge.send_message(ClientMessage::StopGeneration).ok();
            }
            DingtalkCommand::Clear => {
                self.api
                    .send_message(token, conversation_id, "会话上下文已清空。")
                    .await
                    .ok();
            }
        }
    }

    async fn stream_response(
        &self,
        token: &str,
        conversation_id: &str,
        event_rx: &mut tokio::sync::broadcast::Receiver<ServerMessage>,
    ) {
        let mut text_buffer = String::new();

        loop {
            let event = match event_rx.recv().await {
                Ok(event) => event,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };

            match event {
                ServerMessage::Thinking { text } => {
                    self.api
                        .send_message(token, conversation_id, &format!("思考中: {text}"))
                        .await
                        .ok();
                }

                ServerMessage::ContentDelta { delta, .. } => {
                    text_buffer.push_str(&delta);

                    if text_buffer.chars().count() > 3500 {
                        if let Ok(_) = self
                            .api
                            .send_message(token, conversation_id, &text_buffer)
                            .await
                        {
                            text_buffer.clear();
                        }
                    }
                }

                ServerMessage::ToolUse { tool_name, .. } => {
                    let label = codex_im_common::tool_name_label(
                        codex_im_common::Channel::DingTalk,
                        &tool_name,
                    );
                    self.api
                        .send_message(token, conversation_id, &label)
                        .await
                        .ok();
                }

                ServerMessage::MessageComplete { .. } => {
                    if !text_buffer.is_empty() {
                        self.api
                            .send_message(token, conversation_id, &text_buffer)
                            .await
                            .ok();
                    }
                    break;
                }

                ServerMessage::Error { code, message } => {
                    self.api
                        .send_message(
                            token,
                            conversation_id,
                            &format!("错误 [{code}]: {message}"),
                        )
                        .await
                        .ok();
                    break;
                }

                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DingtalkMessage {
    pub message_id: String,
    pub conversation_id: String,
    pub sender_id: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SentMessage {
    pub message_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DingtalkCommand {
    Help,
    NewSession(Option<String>),
    Status,
    Stop,
    Clear,
}
