pub enum Channel {
    Telegram,
    Feishu,
    DingTalk,
}

pub fn tool_name_label(channel: Channel, tool_name: &str) -> String {
    match channel {
        Channel::Telegram => match tool_name {
            "Bash" => "🖥 执行命令".into(),
            "Write" => "📝 写入文件".into(),
            "Read" => "📖 读取文件".into(),
            "Edit" => "✏️ 编辑文件".into(),
            "Grep" => "🔍 搜索代码".into(),
            "Glob" => "📂 查找文件".into(),
            "WebSearch" => "🌐 搜索网页".into(),
            "WebFetch" => "📄 抓取网页".into(),
            _ => format!("🛠 执行 {}", tool_name),
        },
        Channel::Feishu => match tool_name {
            "Bash" => "执行命令".into(),
            "Write" => "写入文件".into(),
            "Read" => "读取文件".into(),
            "Edit" => "编辑文件".into(),
            "Grep" => "搜索代码".into(),
            "Glob" => "查找文件".into(),
            "WebSearch" => "搜索网页".into(),
            "WebFetch" => "抓取网页".into(),
            _ => format!("执行 {}", tool_name),
        },
        Channel::DingTalk => match tool_name {
            "Bash" => "执行命令".into(),
            "Write" => "写入文件".into(),
            "Read" => "读取文件".into(),
            "Edit" => "编辑文件".into(),
            _ => format!("执行 {}", tool_name),
        },
    }
}

pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars - 3).collect();
        format!("{truncated}...")
    }
}
