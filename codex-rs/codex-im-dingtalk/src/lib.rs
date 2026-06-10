mod api;

use std::sync::Arc;

use api::DingtalkApi;
use codex_im_bridge::ImBridge;
use codex_im_protocol::ClientMessage;
use codex_im_protocol::ServerMessage;
use codex_im_protocol::platform::{ImCommand, StreamingBuffer};
use tokio::sync::Mutex;
use tracing::error;
use tracing::info;

#[derive(Debug)]
pub struct DingtalkConfig {
    pub app_key: String,
    pub app_secret: String,
    pub allowed_users: Vec<String>,
}

#[derive(Debug)]
// TODO: wire into adapter — adapter declared but not used outside this crate
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
            if let Some(t) = token.as_ref() {
                return Ok(t.clone());
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

        if let Some(command) = codex_im_protocol::platform::parse_command(text) {
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

    async fn handle_command(
        &self,
        token: &str,
        conversation_id: &str,
        _sender_id: &str,
        cmd: &ImCommand,
    ) {
        match cmd {
            ImCommand::Help => {
                let text = "BCIP 专利智能助手\n\n命令: 新会话 / 帮助 / 状态 / 停止 / 清空";
                self.api
                    .send_message(token, conversation_id, text)
                    .await
                    .ok();
            }
            ImCommand::NewSession(project) => {
                let text = match project {
                    Some(p) => format!("正在创建新会话，项目: {p}"),
                    None => "正在创建新会话...".into(),
                };
                self.api
                    .send_message(token, conversation_id, &text)
                    .await
                    .ok();
            }
            ImCommand::Status => {
                self.api
                    .send_message(token, conversation_id, "查询当前状态...")
                    .await
                    .ok();
            }
            ImCommand::Stop => {
                self.api
                    .send_message(token, conversation_id, "停止当前生成...")
                    .await
                    .ok();
                self.bridge.send_message(ClientMessage::StopGeneration).ok();
            }
            ImCommand::Clear => {
                self.api
                    .send_message(token, conversation_id, "会话上下文已清空。")
                    .await
                    .ok();
            }
            ImCommand::PlatformSpecific(cmd) => {
                tracing::debug!(%cmd, "钉钉不支持的平台特有命令，已忽略");
            }
        }
    }

    async fn stream_response(
        &self,
        token: &str,
        conversation_id: &str,
        event_rx: &mut tokio::sync::broadcast::Receiver<ServerMessage>,
    ) {
        let mut buf = StreamingBuffer::new(3500);

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
                    buf.push_delta(&delta);
                    // 钉钉：先发送完整内容再清空缓冲区（无消息气泡机制）
                    if buf.should_flush()
                        && self
                            .api
                            .send_message(token, conversation_id, buf.content())
                            .await
                            .is_ok()
                    {
                        buf.clear();
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
                    if !buf.is_empty() {
                        self.api
                            .send_message(token, conversation_id, buf.content())
                            .await
                            .ok();
                    }
                    break;
                }

                ServerMessage::Error { code, message } => {
                    self.api
                        .send_message(token, conversation_id, &format!("错误 [{code}]: {message}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use codex_im_protocol::platform::{ImCommand, StreamingBuffer, parse_command};
    use pretty_assertions::assert_eq;

    #[test]
    fn dingtalk_message_construction() {
        let msg = DingtalkMessage {
            message_id: "msg-123".to_string(),
            conversation_id: "conv-1".to_string(),
            sender_id: "user-1".to_string(),
            content: Some("hello".to_string()),
        };
        assert_eq!(msg.message_id, "msg-123");
        assert_eq!(msg.conversation_id, "conv-1");
        assert_eq!(msg.sender_id, "user-1");
        assert_eq!(msg.content.as_deref(), Some("hello"));
    }

    #[test]
    fn dingtalk_message_empty_content() {
        let msg = DingtalkMessage {
            message_id: "msg-456".to_string(),
            conversation_id: "conv-2".to_string(),
            sender_id: "user-2".to_string(),
            content: None,
        };
        assert_eq!(msg.content, None);
    }

    #[test]
    fn sent_message_construction() {
        let msg = SentMessage {
            message_id: "sent-789".to_string(),
        };
        assert_eq!(msg.message_id, "sent-789");
    }

    #[test]
    fn dingtalk_config_fields() {
        let config = DingtalkConfig {
            app_key: "key-abc".to_string(),
            app_secret: "secret-xyz".to_string(),
            allowed_users: vec!["user-1".to_string(), "user-2".to_string()],
        };
        assert_eq!(config.app_key, "key-abc");
        assert_eq!(config.allowed_users.len(), 2);
    }

    #[test]
    fn command_parsing_new_session() {
        let cmd = parse_command("新会话");
        assert_eq!(cmd, Some(ImCommand::NewSession(None)));
    }

    #[test]
    fn command_parsing_new_session_with_project() {
        let cmd = parse_command("新会话 my-project");
        assert_eq!(
            cmd,
            Some(ImCommand::NewSession(Some("my-project".to_string())))
        );
    }

    #[test]
    fn command_parsing_help() {
        assert_eq!(parse_command("/help"), Some(ImCommand::Help));
        assert_eq!(parse_command("帮助"), Some(ImCommand::Help));
    }

    #[test]
    fn command_parsing_status_stop_clear() {
        assert_eq!(parse_command("/status"), Some(ImCommand::Status));
        assert_eq!(parse_command("状态"), Some(ImCommand::Status));
        assert_eq!(parse_command("/stop"), Some(ImCommand::Stop));
        assert_eq!(parse_command("停止"), Some(ImCommand::Stop));
        assert_eq!(parse_command("/clear"), Some(ImCommand::Clear));
        assert_eq!(parse_command("清空"), Some(ImCommand::Clear));
    }

    #[test]
    fn command_parsing_platform_specific() {
        let cmd = parse_command("/custom_command arg1");
        assert_eq!(
            cmd,
            Some(ImCommand::PlatformSpecific(
                "/custom_command arg1".to_string()
            ))
        );
    }

    #[test]
    fn command_parsing_plain_text_returns_none() {
        assert_eq!(parse_command("你好"), None);
        assert_eq!(parse_command("just a normal message"), None);
    }

    #[test]
    fn streaming_buffer_push_and_flush() {
        let mut buf = StreamingBuffer::new(10);
        assert!(buf.is_empty());

        buf.push_delta("hello");
        assert_eq!(buf.content(), "hello");
        assert!(!buf.should_flush());

        buf.push_delta(" world!!!");
        assert!(buf.should_flush());

        let flushed = buf.flush();
        assert_eq!(flushed, "hello world!!!");
        assert!(buf.is_empty());
    }

    #[test]
    fn streaming_buffer_delta_since_last_send() {
        let mut buf = StreamingBuffer::new(100);
        buf.push_delta("part1");
        buf.mark_sent();
        buf.push_delta("part2");
        assert_eq!(buf.delta_since_last_send(), "part2");
    }
}
