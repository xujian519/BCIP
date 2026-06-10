mod api;

use std::sync::Arc;

use codex_im_bridge::ImBridge;
use codex_im_protocol::platform::{ImCommand, StreamingBuffer};
use codex_im_protocol::{ClientMessage, ServerMessage};
use tokio::sync::Mutex;
use tracing::error;
use tracing::info;

use api::FeishuApi;

#[derive(Debug)]
pub struct FeishuConfig {
    pub app_id: String,
    pub app_secret: String,
    pub allowed_users: Vec<String>,
}

#[derive(Debug)]
pub struct FeishuAdapter {
    api: FeishuApi,
    bridge: Arc<ImBridge>,
    #[allow(dead_code)] // stored for future use (e.g., allowed_users checks)
    config: FeishuConfig,
    tenant_access_token: Arc<Mutex<Option<String>>>,
}

impl FeishuAdapter {
    pub fn new(config: FeishuConfig, bridge: Arc<ImBridge>) -> Self {
        let client = reqwest::Client::new();
        let api = FeishuApi::new(client, config.app_id.clone(), config.app_secret.clone());

        Self {
            api,
            bridge,
            config,
            tenant_access_token: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn run(&self) -> ! {
        info!("飞书适配器启动");

        loop {
            match self.ensure_token().await {
                Ok(_) => match self.poll_messages().await {
                    Ok(messages) => {
                        for msg in messages {
                            self.handle_message(msg).await;
                        }
                    }
                    Err(e) => {
                        error!(%e, "获取飞书消息失败");
                    }
                },
                Err(e) => {
                    error!(%e, "获取飞书 Token 失败");
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    async fn ensure_token(&self) -> Result<String, String> {
        {
            let token = self.tenant_access_token.lock().await;
            if token.is_some() {
                return Ok(token.clone().unwrap());
            }
        }
        let new_token = self.api.get_tenant_access_token().await?;
        *self.tenant_access_token.lock().await = Some(new_token.clone());
        Ok(new_token)
    }

    async fn poll_messages(&self) -> Result<Vec<FeishuMessage>, String> {
        let token = self.ensure_token().await?;
        self.api.list_messages(&token).await
    }

    async fn handle_message(&self, message: FeishuMessage) {
        let text = message.content.as_deref().unwrap_or("");

        let token = match self.ensure_token().await {
            Ok(t) => t,
            Err(_) => return,
        };

        if let Some(command) = codex_im_protocol::platform::parse_command(text) {
            self.handle_command(&token, &message.chat_id, &message.sender_id, &command)
                .await;
            return;
        }

        if text.is_empty() {
            return;
        }

        let user_msg = ClientMessage::UserMessage {
            text: text.to_string(),
            attachments: None,
        };

        match self.bridge.send_message(user_msg) {
            Ok(()) => {
                let mut event_rx = self.bridge.subscribe();
                self.stream_response(&token, &message.chat_id, &mut event_rx)
                    .await;
            }
            Err(e) => {
                error!(%e, "发送消息到 Bridge 失败");
                self.api
                    .send_message(&token, &message.chat_id, "发送消息失败，请重试。")
                    .await
                    .ok();
            }
        }
    }
    async fn handle_command(&self, token: &str, chat_id: &str, _sender_id: &str, cmd: &ImCommand) {
        match cmd {
            ImCommand::Help => {
                let text = codex_im_protocol::platform::help_text("飞书");
                self.api.send_message(token, chat_id, &text).await.ok();
            }
            ImCommand::NewSession(project) => {
                let text = match project {
                    Some(p) => format!("正在创建新会话，项目: {p}"),
                    None => "正在创建新会话...".into(),
                };
                self.api.send_message(token, chat_id, &text).await.ok();
            }
            ImCommand::Status => {
                self.api
                    .send_message(token, chat_id, "查询当前状态...")
                    .await
                    .ok();
            }
            ImCommand::Stop => {
                self.api
                    .send_message(token, chat_id, "停止当前生成...")
                    .await
                    .ok();
                self.bridge.send_message(ClientMessage::StopGeneration).ok();
            }
            ImCommand::Clear => {
                self.api
                    .send_message(token, chat_id, "会话上下文已清空。")
                    .await
                    .ok();
            }
            ImCommand::PlatformSpecific(cmd) => {
                tracing::debug!(%cmd, "飞书不支持的平台特有命令，已忽略");
            }
        }
    }

    async fn stream_response(
        &self,
        token: &str,
        chat_id: &str,
        event_rx: &mut tokio::sync::broadcast::Receiver<ServerMessage>,
    ) {
        let mut current_message_id: Option<String> = None;
        let mut buf = StreamingBuffer::new(4000);

        loop {
            let event = match event_rx.recv().await {
                Ok(event) => event,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };

            match event {
                ServerMessage::Thinking { text } => {
                    self.api
                        .send_message(token, chat_id, &format!("思考中: {text}"))
                        .await
                        .ok();
                }
                ServerMessage::ContentDelta { delta, .. } => {
                    buf.push_delta(&delta);
                    // 飞书消息气泡机制：先清空缓冲区再发送空消息以"断开"前一个气泡，
                    // 与钉钉直接发送内容后清空的行为不同
                    if buf.should_flush() {
                        buf.clear();
                        if let Ok(msg) = self.api.send_message(token, chat_id, "").await {
                            current_message_id = Some(msg.message_id);
                        }
                    } else if let Some(_msg_id) = &current_message_id {
                        self.api
                            .send_message(token, chat_id, buf.content())
                            .await
                            .ok();
                    } else if let Ok(msg) =
                        self.api.send_message(token, chat_id, buf.content()).await
                    {
                        current_message_id = Some(msg.message_id);
                    }
                }
                ServerMessage::ToolUse { tool_name, .. } => {
                    let label = codex_im_common::tool_name_label(
                        codex_im_common::Channel::Feishu,
                        &tool_name,
                    );
                    self.api.send_message(token, chat_id, &label).await.ok();
                }
                ServerMessage::PermissionRequest {
                    request_id,
                    tool_name,
                    tool_input,
                    risk_level,
                    ..
                } => {
                    let text = Self::format_permission_request(
                        &request_id,
                        &tool_name,
                        &tool_input,
                        &risk_level,
                    );
                    self.api.send_message(token, chat_id, &text).await.ok();
                }
                ServerMessage::MessageComplete { .. } => break,
                ServerMessage::Error { code, message } => {
                    self.api
                        .send_message(token, chat_id, &format!("错误 [{code}]: {message}"))
                        .await
                        .ok();
                    break;
                }
                _ => {}
            }
        }
    }

    fn format_permission_request(
        request_id: &str,
        tool_name: &str,
        _tool_input: &serde_json::Value,
        risk_level: &codex_im_protocol::RiskLevel,
    ) -> String {
        let label = codex_im_common::tool_name_label(codex_im_common::Channel::Feishu, tool_name);
        let risk_text = match risk_level {
            codex_im_protocol::RiskLevel::Low => "低",
            codex_im_protocol::RiskLevel::Medium => "中",
            codex_im_protocol::RiskLevel::High => "高",
            codex_im_protocol::RiskLevel::Critical => "极高",
        };

        format!(
            "权限请求 [{request_id}]\n\n{label}\n风险等级: {risk_text}\n\n回复 /allow {request_id} 或 /deny {request_id}"
        )
    }
}

#[derive(Debug, Clone)]
pub struct FeishuMessage {
    pub message_id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SentMessage {
    pub message_id: String,
}
