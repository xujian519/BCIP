use std::time::Duration;

use chrono::Utc;

use super::backoff::ExponentialBackoff;

// ── AgentRecoveryState ──

/// Agent 恢复状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AgentRecoveryState {
    /// 正常运行。
    Healthy,
    /// 等待重启。
    PendingRestart,
    /// 正在重启中。
    Restarting,
    /// 超过最大重启次数，不可恢复。
    Unrecoverable,
}

// ── RecoveryPolicy ──

/// Agent 恢复策略配置。
#[derive(Debug, Clone)]
pub struct RecoveryPolicy {
    /// 最大自动重启次数（超过后标记为 Unrecoverable）。
    pub max_restarts: u32,
    /// 重启间隔退避策略。
    pub restart_backoff: ExponentialBackoff,
    /// Unresponsive 检测后是否自动重启。
    pub restart_on_unresponsive: bool,
    /// Unresponsive 检测超时。
    pub unresponsive_timeout: Duration,
    /// 是否允许任务转移。
    pub allow_task_reassignment: bool,
}

impl RecoveryPolicy {
    /// 检查是否可以重启。
    pub fn can_restart(&self, context: &RecoveryContext) -> bool {
        context.state != AgentRecoveryState::Unrecoverable
            && context.restart_count < self.max_restarts
    }

    /// Agent 默认恢复策略。
    pub fn default_for_agents() -> Self {
        Self::default()
    }
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_backoff: ExponentialBackoff::standard(),
            restart_on_unresponsive: true,
            unresponsive_timeout: Duration::from_secs(30),
            allow_task_reassignment: true,
        }
    }
}

// ── RecoveryContext ──

/// Agent 恢复上下文，存储在 AgentMetadata 中。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecoveryContext {
    /// 当前重启计数。
    pub restart_count: u32,
    /// 上次重启时间（Unix 秒）。
    pub last_restart_at: Option<i64>,
    /// 上次失败原因。
    pub last_failure_reason: Option<String>,
    /// 当前恢复状态。
    pub state: AgentRecoveryState,
}

impl RecoveryContext {
    /// 创建初始恢复上下文（Healthy, restart_count=0）。
    pub fn new() -> Self {
        Self {
            restart_count: 0,
            last_restart_at: None,
            last_failure_reason: None,
            state: AgentRecoveryState::Healthy,
        }
    }

    /// 记录一次重启。
    pub fn record_restart(&mut self, reason: String) {
        self.restart_count += 1;
        self.last_restart_at = Some(Utc::now().timestamp());
        self.last_failure_reason = Some(reason);
        self.state = AgentRecoveryState::Restarting;
    }

    /// 标记为健康。
    pub fn mark_healthy(&mut self) {
        self.state = AgentRecoveryState::Healthy;
    }

    /// 标记为不可恢复。
    pub fn mark_unrecoverable(&mut self, reason: String) {
        self.state = AgentRecoveryState::Unrecoverable;
        self.last_failure_reason = Some(reason);
    }

    /// 检查是否应该重启（委托给 RecoveryPolicy）。
    pub fn should_restart(&self, policy: &RecoveryPolicy) -> bool {
        policy.can_restart(self)
    }
}

impl Default for RecoveryContext {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_context_is_healthy() {
        let ctx = RecoveryContext::new();
        assert_eq!(ctx.state, AgentRecoveryState::Healthy);
        assert_eq!(ctx.restart_count, 0);
        assert!(ctx.last_restart_at.is_none());
        assert!(ctx.last_failure_reason.is_none());
    }

    #[test]
    fn record_restart_increments_count() {
        let mut ctx = RecoveryContext::new();
        ctx.record_restart("panic".into());
        assert_eq!(ctx.restart_count, 1);
        ctx.record_restart("timeout".into());
        assert_eq!(ctx.restart_count, 2);
    }

    #[test]
    fn record_restart_updates_state() {
        let mut ctx = RecoveryContext::new();
        ctx.record_restart("panic".into());
        assert_eq!(ctx.state, AgentRecoveryState::Restarting);
    }

    #[test]
    fn record_restart_updates_reason() {
        let mut ctx = RecoveryContext::new();
        ctx.record_restart("OOM".into());
        assert_eq!(ctx.last_failure_reason.as_deref(), Some("OOM"));
    }

    #[test]
    fn mark_healthy_sets_state() {
        let mut ctx = RecoveryContext::new();
        ctx.record_restart("panic".into());
        ctx.mark_healthy();
        assert_eq!(ctx.state, AgentRecoveryState::Healthy);
    }

    #[test]
    fn mark_unrecoverable_sets_state() {
        let mut ctx = RecoveryContext::new();
        ctx.mark_unrecoverable("exceeded max".into());
        assert_eq!(ctx.state, AgentRecoveryState::Unrecoverable);
        assert_eq!(ctx.last_failure_reason.as_deref(), Some("exceeded max"));
    }

    #[test]
    fn can_restart_true_when_below_limit() {
        let policy = RecoveryPolicy::default();
        let mut ctx = RecoveryContext::new();
        assert!(policy.can_restart(&ctx));

        ctx.record_restart("panic".into());
        assert!(policy.can_restart(&ctx));

        ctx.record_restart("panic".into());
        assert!(policy.can_restart(&ctx));
    }

    #[test]
    fn can_restart_false_when_at_limit() {
        let policy = RecoveryPolicy {
            max_restarts: 2,
            ..RecoveryPolicy::default()
        };
        let mut ctx = RecoveryContext::new();
        ctx.record_restart("panic".into());
        ctx.record_restart("panic".into());
        // restart_count=2 >= max_restarts=2
        assert!(!policy.can_restart(&ctx));
    }

    #[test]
    fn can_restart_false_when_unrecoverable() {
        let policy = RecoveryPolicy::default();
        let mut ctx = RecoveryContext::new();
        ctx.mark_unrecoverable("fatal".into());
        assert!(!policy.can_restart(&ctx));
    }

    #[test]
    fn default_policy_max_restarts() {
        let policy = RecoveryPolicy::default();
        assert_eq!(policy.max_restarts, 3);
    }

    #[test]
    fn should_restart_delegates_to_policy() {
        let policy = RecoveryPolicy {
            max_restarts: 1,
            ..RecoveryPolicy::default()
        };
        let mut ctx = RecoveryContext::new();
        assert!(ctx.should_restart(&policy));

        ctx.record_restart("panic".into());
        assert!(!ctx.should_restart(&policy));
    }

    #[test]
    fn last_restart_at_updated() {
        let before = Utc::now().timestamp();
        let mut ctx = RecoveryContext::new();
        ctx.record_restart("panic".into());
        let after = Utc::now().timestamp();

        let ts = ctx.last_restart_at.unwrap();
        assert!(ts >= before);
        assert!(ts <= after);
    }

    #[test]
    fn default_for_agents_matches_default() {
        let a = RecoveryPolicy::default_for_agents();
        let b = RecoveryPolicy::default();
        assert_eq!(a.max_restarts, b.max_restarts);
        assert_eq!(a.restart_on_unresponsive, b.restart_on_unresponsive);
        assert_eq!(a.unresponsive_timeout, b.unresponsive_timeout);
        assert_eq!(a.allow_task_reassignment, b.allow_task_reassignment);
    }
}
