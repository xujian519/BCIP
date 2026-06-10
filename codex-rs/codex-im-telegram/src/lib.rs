mod api;
mod commands;
mod permissions;

use std::sync::Arc;

use codex_im_bridge::ImBridge;
use codex_im_protocol::ClientMessage;
use codex_im_protocol::ServerMessage;
use reqwest::Client;
use tracing::error;
use tracing::info;
use tracing::warn;

use api::TelegramApi;
use commands::parse_command;

const MAX_TELEGRAM_MESSAGE_LENGTH: usize = 4096;

#[derive(Debug)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub allowed_users: Vec<i64>,
}

#[derive(Debug)]
pub struct TelegramAdapter {
    api: TelegramApi,
    bridge: Arc<ImBridge>,
    config: TelegramConfig,
}

impl TelegramAdapter {
    pub fn new(config: TelegramConfig, bridge: Arc<ImBridge>) -> Self {
        let client = Client::new();
        let api = TelegramApi::new(client, config.bot_token.clone());

        Self {
            api,
            bridge,
            config,
        }
    }

    pub async fn run(&self) -> ! {
        let mut offset = 0i64;

        loop {
            match self.api.get_updates(offset).await {
                Ok(updates) => {
                    for update in updates {
                        offset = offset.max(update.update_id + 1);

                        if let Some(message) = update.message {
                            let chat_id = message.chat.id;

                            if let Some(ref from) = message.from {
                                let user_id = from.id;

                                if !self.config.allowed_users.is_empty()
                                    && !self.config.allowed_users.contains(&user_id)
                                {
                                    self.api
                                        .send_message(chat_id, "未授权用户。请联系管理员配对。")
                                        .await
                                        .ok();
                                    continue;
                                }
                            }

                            self.handle_message(chat_id, &message).await;
                        }

                        if let Some(callback) = update.callback_query
                            && let Some(msg) = callback.message
                        {
                            let chat_id = msg.chat.id;
                            self.handle_callback(
                                chat_id,
                                &callback.id,
                                &callback.data.unwrap_or_default(),
                            )
                            .await;
                        }
                    }
                }
                Err(e) => {
                    error!(%e, "获取 Telegram 更新失败");
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    async fn handle_message(&self, chat_id: i64, message: &TelegramMessage) {
        let text = message.text.clone().unwrap_or_default();

        if let Some(command) = parse_command(&text) {
            self.handle_command(chat_id, &command).await;
            return;
        }

        if text.is_empty() && message.photo.is_none() && message.document.is_none() {
            return;
        }

        let user_msg = ClientMessage::UserMessage {
            text,
            attachments: None,
        };

        info!(chat_id, "发送用户消息");

        match self.bridge.send_message(user_msg) {
            Ok(()) => {
                let mut event_rx = self.bridge.subscribe();
                self.stream_response(chat_id, &mut event_rx).await;
            }
            Err(e) => {
                error!(%e, "发送消息到 Bridge 失败");
                self.api
                    .send_message(chat_id, "发送消息失败，请重试。")
                    .await
                    .ok();
            }
        }
    }

    async fn stream_response(
        &self,
        chat_id: i64,
        event_rx: &mut tokio::sync::broadcast::Receiver<ServerMessage>,
    ) {
        let mut current_message_id = None;
        let mut text_buffer = String::new();
        let mut thinking_message_id = None;

        loop {
            let event = match event_rx.recv().await {
                Ok(event) => event,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!(n, "事件落后");
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };

            match event {
                ServerMessage::Connected { .. } => {}

                ServerMessage::Thinking { text } => {
                    if thinking_message_id.is_none() {
                        let sent = self.api.send_message(chat_id, &format!("🤔 {text}")).await;
                        if let Ok(msg) = sent {
                            thinking_message_id = Some(msg.message_id);
                        }
                    } else if let Some(id) = thinking_message_id {
                        self.api
                            .edit_message_text(chat_id, id, &format!("🤔 {text}"))
                            .await
                            .ok();
                    }
                }

                ServerMessage::ContentDelta { delta, .. } => {
                    if let Some(id) = thinking_message_id.take() {
                        self.api.delete_message(chat_id, id).await.ok();
                    }

                    text_buffer.push_str(&delta);

                    if let Some(msg_id) = current_message_id {
                        if text_buffer.chars().count() > MAX_TELEGRAM_MESSAGE_LENGTH {
                            self.api
                                .edit_message_text(chat_id, msg_id, &text_buffer)
                                .await
                                .ok();
                            text_buffer.clear();
                            let sent = self.api.send_message(chat_id, "").await;
                            if let Ok(msg) = sent {
                                current_message_id = Some(msg.message_id);
                            }
                        } else {
                            self.api
                                .edit_message_text(chat_id, msg_id, &text_buffer)
                                .await
                                .ok();
                        }
                    } else {
                        let sent = self.api.send_message(chat_id, &text_buffer).await;
                        if let Ok(msg) = sent {
                            current_message_id = Some(msg.message_id);
                        }
                    }
                }

                ServerMessage::ToolUse {
                    tool_name,
                    tool_input,
                    ..
                } => {
                    let label = codex_im_common::tool_name_label(
                        codex_im_common::Channel::Telegram,
                        &tool_name,
                    );
                    let detail = codex_im_common::truncate_text(
                        &serde_json::to_string(&tool_input).unwrap_or_default(),
                        200,
                    );
                    self.api
                        .send_message(chat_id, &format!("{label}\n{detail}"))
                        .await
                        .ok();
                }

                ServerMessage::PermissionRequest {
                    request_id,
                    tool_name,
                    tool_input,
                    risk_level,
                    ..
                } => {
                    let text = permissions::format_permission_request(
                        &request_id,
                        &tool_name,
                        &tool_input,
                        &risk_level,
                    );
                    self.api
                        .send_message_with_keyboard(
                            chat_id,
                            &text,
                            &permissions::build_permission_keyboard(&request_id),
                        )
                        .await
                        .ok();
                }

                ServerMessage::MessageComplete { .. } => {
                    let _ = current_message_id.is_some();
                    break;
                }

                ServerMessage::Error { code, message } => {
                    self.api
                        .send_message(chat_id, &format!("❌ 错误 [{code}]: {message}"))
                        .await
                        .ok();
                    break;
                }

                ServerMessage::Image {
                    mime_type,
                    data_base64,
                    ..
                } => {
                    self.api
                        .send_photo(chat_id, &data_base64, &mime_type)
                        .await
                        .ok();
                }

                ServerMessage::Status { state, detail: _ } => {
                    if matches!(state, codex_im_protocol::SessionState::Compacting) {
                        self.api.send_message(chat_id, "上下文压缩中...").await.ok();
                    }
                }

                _ => {}
            }
        }
    }

    async fn send_permission_response(
        &self,
        request_id: &str,
        decision: codex_im_protocol::PermissionDecision,
    ) {
        let msg = ClientMessage::PermissionResponse {
            request_id: request_id.to_string(),
            decision,
        };
        self.bridge.send_message(msg).ok();
    }
}

#[derive(Debug, serde::Deserialize)]
struct TelegramUpdate {
    update_id: i64,
    message: Option<TelegramMessage>,
    callback_query: Option<TelegramCallbackQuery>,
}

#[derive(Debug, serde::Deserialize)]
struct TelegramMessage {
    #[allow(dead_code)] // part of Telegram API response, may be needed for reply threading
    message_id: i64,
    chat: TelegramChat,
    from: Option<TelegramUser>,
    text: Option<String>,
    photo: Option<Vec<TelegramPhotoSize>>,
    document: Option<TelegramDocument>,
}

#[derive(Debug, serde::Deserialize)]
struct TelegramChat {
    id: i64,
}

#[derive(Debug, serde::Deserialize)]
struct TelegramUser {
    id: i64,
    #[allow(dead_code)] // part of Telegram API response, may be used for display
    first_name: Option<String>,
    #[allow(dead_code)] // part of Telegram API response, may be used for display
    username: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
// Kept for complete Telegram API deserialization; fields used for presence checks only
#[allow(dead_code)]
struct TelegramPhotoSize {
    file_id: String,
}

#[derive(Debug, serde::Deserialize)]
// Kept for complete Telegram API deserialization; fields used for presence checks only
#[allow(dead_code)]
struct TelegramDocument {
    file_id: String,
    file_name: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TelegramCallbackQuery {
    id: String,
    message: Option<TelegramMessage>,
    data: Option<String>,
}
