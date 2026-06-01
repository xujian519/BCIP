use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model,
    Ide,
    Permissions,
    Keymap,
    Vim,
    #[strum(serialize = "setup-default-sandbox")]
    ElevateSandbox,
    #[strum(serialize = "sandbox-add-read-dir")]
    SandboxReadRoot,
    Experimental,
    #[strum(to_string = "approve")]
    AutoReview,
    Memories,
    Skills,
    Hooks,
    Review,
    Rename,
    New,
    Resume,
    Fork,
    Init,
    Compact,
    Plan,
    Goal,
    Agent,
    Side,
    Btw,
    Copy,
    Raw,
    Diff,
    Mention,
    Status,
    DebugConfig,
    Title,
    Statusline,
    Theme,
    #[strum(to_string = "pets", serialize = "pet")]
    Pets,
    Mcp,
    Apps,
    Plugins,
    Logout,
    Quit,
    Exit,
    Feedback,
    Rollout,
    Ps,
    #[strum(to_string = "stop", serialize = "clean")]
    Stop,
    Clear,
    Personality,
    Realtime,
    Settings,
    TestApproval,
    #[strum(serialize = "subagents")]
    MultiAgents,
    // Debugging commands.
    #[strum(serialize = "debug-m-drop")]
    MemoryDrop,
    #[strum(serialize = "debug-m-update")]
    MemoryUpdate,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Feedback => "向维护者发送日志",
            SlashCommand::New => "在当前会话中开始新对话",
            SlashCommand::Init => "创建 AGENTS.md 文件提供指令",
            SlashCommand::Compact => "总结对话以避免达到上下文限制",
            SlashCommand::Review => "审查当前更改并发现问题",
            SlashCommand::Rename => "重命名当前线程",
            SlashCommand::Resume => "恢复已保存的对话",
            SlashCommand::Clear => "清空终端并开始新对话",
            SlashCommand::Fork => "分叉当前对话",
            SlashCommand::Quit | SlashCommand::Exit => "退出云熙专利智能体",
            SlashCommand::Copy => "复制上一条回复为 Markdown 格式",
            SlashCommand::Raw => "切换原始回滚模式以便于终端选择复制",
            SlashCommand::Diff => "显示 Git 差异（包括未跟踪文件）",
            SlashCommand::Mention => "引用某个文件",
            SlashCommand::Skills => "使用技能提升特定任务的处理能力",
            SlashCommand::Hooks => "查看和管理生命周期钩子",
            SlashCommand::Status => "显示当前会话配置和令牌使用情况",
            SlashCommand::DebugConfig => "显示配置层次和需求来源以进行调试",
            SlashCommand::Title => "配置终端标题中显示的项目",
            SlashCommand::Statusline => "配置状态行中显示的项目",
            SlashCommand::Theme => "选择语法高亮主题",
            SlashCommand::Pets => "选择或隐藏终端宠物",
            SlashCommand::Ps => "列出后台终端",
            SlashCommand::Stop => "停止所有后台终端",
            SlashCommand::MemoryDrop => "请勿使用",
            SlashCommand::MemoryUpdate => "请勿使用",
            SlashCommand::Model => "选择要使用的模型和推理强度",
            SlashCommand::Ide => "包含当前选中内容、打开的文件以及来自 IDE 的其他上下文",
            SlashCommand::Personality => "选择交流风格",
            SlashCommand::Realtime => "切换实时语音模式（实验性）",
            SlashCommand::Settings => "打开统一设置面板",
            SlashCommand::Plan => "切换到计划模式",
            SlashCommand::Goal => "设置或查看长期运行任务的目标",
            SlashCommand::Agent | SlashCommand::MultiAgents => "切换当前智能体线程",
            SlashCommand::Side | SlashCommand::Btw => "在临时分叉中开始旁路对话",
            SlashCommand::Permissions => "选择允许执行的操作",
            SlashCommand::Keymap => "重新映射 TUI 快捷键",
            SlashCommand::Vim => "为编辑器切换 Vim 模式",
            SlashCommand::ElevateSandbox => "设置升级版智能体沙箱",
            SlashCommand::SandboxReadRoot => "让沙箱读取目录：/sandbox-add-read-dir <绝对路径>",
            SlashCommand::Experimental => "切换实验性功能",
            SlashCommand::AutoReview => "批准最近一次自动审查拒绝的重试",
            SlashCommand::Memories => "配置记忆使用和生成",
            SlashCommand::Mcp => "列出已配置的 MCP 工具；使用 /mcp verbose 查看详情",
            SlashCommand::Apps => "管理应用程序",
            SlashCommand::Plugins => "浏览插件",
            SlashCommand::Logout => "退出登录",
            SlashCommand::Rollout => "打印发布文件路径",
            SlashCommand::TestApproval => "测试审批请求",
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command supports inline args (for example `/review ...`).
    pub fn supports_inline_args(self) -> bool {
        matches!(
            self,
            SlashCommand::Review
                | SlashCommand::Rename
                | SlashCommand::Plan
                | SlashCommand::Goal
                | SlashCommand::Ide
                | SlashCommand::Keymap
                | SlashCommand::Mcp
                | SlashCommand::Raw
                | SlashCommand::Pets
                | SlashCommand::Side
                | SlashCommand::Btw
                | SlashCommand::Resume
                | SlashCommand::SandboxReadRoot
        )
    }

    /// Whether this command remains available inside an active side conversation.
    pub fn available_in_side_conversation(self) -> bool {
        matches!(
            self,
            SlashCommand::Copy
                | SlashCommand::Raw
                | SlashCommand::Diff
                | SlashCommand::Mention
                | SlashCommand::Status
                | SlashCommand::Ide
        )
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::New
            | SlashCommand::Resume
            | SlashCommand::Fork
            | SlashCommand::Init
            | SlashCommand::Compact
            | SlashCommand::Model
            | SlashCommand::Personality
            | SlashCommand::Permissions
            | SlashCommand::Keymap
            | SlashCommand::Vim
            | SlashCommand::ElevateSandbox
            | SlashCommand::SandboxReadRoot
            | SlashCommand::Experimental
            | SlashCommand::Memories
            | SlashCommand::Review
            | SlashCommand::Plan
            | SlashCommand::Clear
            | SlashCommand::Logout
            | SlashCommand::MemoryDrop
            | SlashCommand::MemoryUpdate => false,
            SlashCommand::Diff
            | SlashCommand::Copy
            | SlashCommand::Raw
            | SlashCommand::Rename
            | SlashCommand::Mention
            | SlashCommand::Skills
            | SlashCommand::Hooks
            | SlashCommand::Status
            | SlashCommand::DebugConfig
            | SlashCommand::Ps
            | SlashCommand::Stop
            | SlashCommand::Goal
            | SlashCommand::Mcp
            | SlashCommand::Apps
            | SlashCommand::Plugins
            | SlashCommand::Title
            | SlashCommand::Statusline
            | SlashCommand::AutoReview
            | SlashCommand::Feedback
            | SlashCommand::Ide
            | SlashCommand::Quit
            | SlashCommand::Exit
            | SlashCommand::Side
            | SlashCommand::Btw => true,
            SlashCommand::Rollout => true,
            SlashCommand::TestApproval => true,
            SlashCommand::Realtime => true,
            SlashCommand::Settings => true,
            SlashCommand::Agent | SlashCommand::MultiAgents => true,
            SlashCommand::Theme | SlashCommand::Pets => false,
        }
    }

    fn is_visible(self) -> bool {
        match self {
            SlashCommand::SandboxReadRoot => cfg!(target_os = "windows"),
            SlashCommand::Copy => !cfg!(target_os = "android"),
            SlashCommand::Rollout | SlashCommand::TestApproval => cfg!(debug_assertions),
            _ => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter()
        .filter(|command| command.is_visible())
        .map(|c| (c.command(), c))
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    use super::SlashCommand;

    #[test]
    fn stop_command_is_canonical_name() {
        assert_eq!(SlashCommand::Stop.command(), "stop");
    }

    #[test]
    fn clean_alias_parses_to_stop_command() {
        assert_eq!(SlashCommand::from_str("clean"), Ok(SlashCommand::Stop));
    }

    #[test]
    fn pet_alias_parses_to_pets_command() {
        assert_eq!(SlashCommand::Pets.command(), "pets");
        assert_eq!(SlashCommand::from_str("pet"), Ok(SlashCommand::Pets));
    }

    #[test]
    fn certain_commands_are_available_during_task() {
        assert!(SlashCommand::Goal.available_during_task());
        assert!(SlashCommand::Ide.available_during_task());
        assert!(SlashCommand::Title.available_during_task());
        assert!(SlashCommand::Statusline.available_during_task());
        assert!(SlashCommand::Raw.available_during_task());
        assert!(SlashCommand::Raw.available_in_side_conversation());
        assert!(SlashCommand::Raw.supports_inline_args());
    }

    #[test]
    fn auto_review_command_is_approve() {
        assert_eq!(SlashCommand::AutoReview.command(), "approve");
        assert_eq!(
            SlashCommand::from_str("approve"),
            Ok(SlashCommand::AutoReview)
        );
    }
}
