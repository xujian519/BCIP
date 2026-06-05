//! 统一 IM 平台抽象
//!
//! 为钉钉、飞书、Telegram 等平台提供统一的 trait 和共享逻辑：
//! - 命令解析与处理
//! - 消息流缓冲
//! - 平台能力声明

/// 统一平台 trait：所有 IM 适配器实现此接口
#[async_trait::async_trait]
pub trait ImPlatform: Send + Sync {
    /// 平台唯一标识（如 "dingtalk", "feishu", "telegram"）
    fn platform_name(&self) -> &str;

    /// 平台单条消息最大字符数
    fn max_message_length(&self) -> usize {
        4000
    }

    /// 发送文本消息到指定聊天
    async fn send_text(&self, chat_id: &str, text: &str) -> Result<(), String>;

    /// 发送错误消息（平台可覆写以定制格式）
    async fn send_error(&self, chat_id: &str, error: &str) -> Result<(), String> {
        self.send_text(chat_id, &format!("❌ {error}")).await
    }

    /// 检查用户是否有权限使用
    fn is_user_allowed(&self, user_id: &str) -> bool;

    /// 启动平台的消息轮询/Webhook 监听循环
    async fn run(&self) -> Result<(), String>;
}

/// 统一命令类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImCommand {
    Help,
    NewSession,
    Status,
    Stop,
    Clear,
    /// 平台特有命令（如 Telegram 的 /allow, 飞书的 /projects）
    PlatformSpecific(String),
}

/// 命令解析器：将文本消息解析为统一的 ImCommand
pub fn parse_command(text: &str) -> Option<ImCommand> {
    let trimmed = text.trim();
    match trimmed {
        "/help" | "帮助" | "/帮助" => Some(ImCommand::Help),
        "/new" | "新会话" | "/新会话" | "/start" => Some(ImCommand::NewSession),
        "/status" | "状态" | "/状态" => Some(ImCommand::Status),
        "/stop" | "停止" | "/停止" => Some(ImCommand::Stop),
        "/clear" | "清空" | "/清空" => Some(ImCommand::Clear),
        _ if trimmed.starts_with('/') => Some(ImCommand::PlatformSpecific(trimmed.to_string())),
        _ => None,
    }
}

/// 生成统一的帮助文本
pub fn help_text(platform_name: &str) -> String {
    format!(
        "🤖 BCIP 专利智能助手 ({platform_name})\n\n\
         📋 可用命令:\n\
         /help  - 显示帮助\n\
         /new   - 开始新会话\n\
         /status - 查看状态\n\
         /stop  - 停止生成\n\
         /clear - 清空上下文\n\n\
         💡 直接发送消息即可与助手对话"
    )
}

/// 消息流缓冲区：管理分段发送长文本
#[derive(Debug)]
pub struct StreamingBuffer {
    buffer: String,
    max_length: usize,
    last_sent_len: usize,
}

impl StreamingBuffer {
    pub fn new(max_length: usize) -> Self {
        Self {
            buffer: String::new(),
            max_length,
            last_sent_len: 0,
        }
    }

    /// 追加增量文本
    pub fn push_delta(&mut self, delta: &str) {
        self.buffer.push_str(delta);
    }

    /// 判断是否需要发送（缓冲区是否超过最大长度）
    pub fn should_flush(&self) -> bool {
        self.buffer.chars().count() >= self.max_length
    }

    /// 取出并清空缓冲区内容
    pub fn flush(&mut self) -> String {
        self.last_sent_len = 0;
        std::mem::take(&mut self.buffer)
    }

    /// 获取当前缓冲区内容（不消耗）
    pub fn content(&self) -> &str {
        &self.buffer
    }

    /// 获取自上次发送以来的增量内容
    pub fn delta_since_last_send(&self) -> &str {
        if self.last_sent_len >= self.buffer.len() {
            return "";
        }
        &self.buffer[self.last_sent_len..]
    }

    /// 标记当前内容为已发送
    pub fn mark_sent(&mut self) {
        self.last_sent_len = self.buffer.len();
    }

    /// 缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.last_sent_len = 0;
    }
}

/// 工具名称标签生成（统一格式）
pub fn tool_label(tool_name: &str) -> String {
    format!("🔧 [{tool_name}]")
}

/// 将 ServerMessage 流式事件转换为平台友好的文本
pub fn format_server_event(event: &super::ServerMessage) -> Option<String> {
    match event {
        super::ServerMessage::Thinking { text } => {
            if text.is_empty() {
                Some("🤔 思考中...".to_string())
            } else {
                Some(format!("🤔 {text}"))
            }
        }
        super::ServerMessage::ToolUse { tool_name, .. } => Some(tool_label(tool_name)),
        super::ServerMessage::Error { message, .. } => Some(format!("❌ {message}")),
        super::ServerMessage::Status { state, detail } => {
            let state_str = match state {
                super::SessionState::Idle => "空闲",
                super::SessionState::Thinking => "思考中",
                super::SessionState::Streaming => "生成中",
                super::SessionState::ToolExecuting => "工具执行中",
                super::SessionState::PermissionPending => "等待权限",
                super::SessionState::Compacting => "压缩上下文中",
                super::SessionState::Error => "错误",
            };
            Some(if let Some(d) = detail {
                format!("{state_str}: {d}")
            } else {
                state_str.to_string()
            })
        }
        super::ServerMessage::SystemNotification { text, .. } => Some(format!("📢 {text}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_chinese() {
        assert_eq!(parse_command("帮助"), Some(ImCommand::Help));
        assert_eq!(parse_command("新会话"), Some(ImCommand::NewSession));
        assert_eq!(parse_command("状态"), Some(ImCommand::Status));
        assert_eq!(parse_command("停止"), Some(ImCommand::Stop));
        assert_eq!(parse_command("清空"), Some(ImCommand::Clear));
    }

    #[test]
    fn test_parse_command_english() {
        assert_eq!(parse_command("/help"), Some(ImCommand::Help));
        assert_eq!(parse_command("/new"), Some(ImCommand::NewSession));
        assert_eq!(parse_command("/status"), Some(ImCommand::Status));
        assert_eq!(parse_command("/stop"), Some(ImCommand::Stop));
        assert_eq!(parse_command("/clear"), Some(ImCommand::Clear));
    }

    #[test]
    fn test_parse_command_platform_specific() {
        assert_eq!(
            parse_command("/allow @user"),
            Some(ImCommand::PlatformSpecific("/allow @user".to_string()))
        );
        assert_eq!(
            parse_command("/projects"),
            Some(ImCommand::PlatformSpecific("/projects".to_string()))
        );
    }

    #[test]
    fn test_parse_command_none() {
        assert_eq!(parse_command("你好"), None);
        assert_eq!(parse_command("分析一下这个专利"), None);
    }

    #[test]
    fn test_streaming_buffer() {
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
    fn test_help_text() {
        let text = help_text("test");
        assert!(text.contains("BCIP"));
        assert!(text.contains("/help"));
        assert!(text.contains("/new"));
    }
}
