use codex_im_protocol::PermissionDecision;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionCommand {
    Allow(String),
    AlwaysAllow(String),
    Deny(String),
}

pub fn parse_permission_command(input: &str) -> Option<PermissionCommand> {
    let trimmed = input.trim();

    if let Some(rest) = trimmed
        .strip_prefix("/allow-always ")
        .or_else(|| trimmed.strip_prefix("/always "))
        .or_else(|| trimmed.strip_prefix("/allow-always"))
        .filter(|s| !s.is_empty())
    {
        return Some(PermissionCommand::AlwaysAllow(rest.trim().to_string()));
    }

    if let Some(rest) = trimmed.strip_prefix("/allow ").filter(|s| !s.is_empty()) {
        return Some(PermissionCommand::Allow(rest.trim().to_string()));
    }

    if let Some(rest) = trimmed.strip_prefix("/deny ").filter(|s| !s.is_empty()) {
        return Some(PermissionCommand::Deny(rest.trim().to_string()));
    }

    if let Some(rest) = trimmed.strip_prefix("/allow-always ") {
        return Some(PermissionCommand::AlwaysAllow(rest.trim().to_string()));
    }

    let lower = trimmed.to_lowercase();
    if lower == "yes" || lower == "允许" || lower == "1" {
        return Some(PermissionCommand::Allow(String::new()));
    }
    if lower == "no" || lower == "拒绝" || lower == "2" {
        return Some(PermissionCommand::Deny(String::new()));
    }
    if lower == "always" || lower == "始终允许" || lower == "3" {
        return Some(PermissionCommand::AlwaysAllow(String::new()));
    }

    None
}

impl PermissionCommand {
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Allow(id) | Self::AlwaysAllow(id) | Self::Deny(id) => {
                if id.is_empty() {
                    None
                } else {
                    Some(id)
                }
            }
        }
    }

    pub fn decision(&self) -> PermissionDecision {
        match self {
            Self::Allow(_) => PermissionDecision::Allow,
            Self::AlwaysAllow(_) => PermissionDecision::AlwaysAllow,
            Self::Deny(_) => PermissionDecision::Deny,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_allow() {
        let cmd = parse_permission_command("/allow req-123").unwrap();
        assert_eq!(cmd, PermissionCommand::Allow("req-123".into()));
    }

    #[test]
    fn test_parse_always_allow() {
        let cmd = parse_permission_command("/always req-456").unwrap();
        assert_eq!(cmd, PermissionCommand::AlwaysAllow("req-456".into()));
    }

    #[test]
    fn test_parse_deny() {
        let cmd = parse_permission_command("/deny req-789").unwrap();
        assert_eq!(cmd, PermissionCommand::Deny("req-789".into()));
    }

    #[test]
    fn test_parse_chinese_yes() {
        let cmd = parse_permission_command("允许").unwrap();
        assert_eq!(cmd, PermissionCommand::Allow(String::new()));
    }

    #[test]
    fn test_parse_chinese_no() {
        let cmd = parse_permission_command("拒绝").unwrap();
        assert_eq!(cmd, PermissionCommand::Deny(String::new()));
    }
}
