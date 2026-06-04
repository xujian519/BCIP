//! 健康检查报告 — 聚合各子系统状态生成全局健康视图。
//!
//! 整合熔断器状态、Agent活性、DLQ深度、恢复状态等信息，
//! 提供统一的 `HealthReport` 供监控和诊断使用。

use std::collections::HashMap;
use std::sync::Arc;

use crate::resilience::circuit_breaker::CircuitBreakerRegistry;
use crate::resilience::recovery::AgentRecoveryState;

/// 单个组件的健康状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentHealth {
    /// 正常运行。
    Healthy,
    /// 功能降级，仍可服务。
    Degraded(String),
    /// 不可用。
    Unhealthy(String),
}

impl ComponentHealth {
    /// 是否健康（非降级、非不可用）。
    pub fn is_healthy(&self) -> bool {
        matches!(self, ComponentHealth::Healthy)
    }

    /// 是否仍可服务（健康或降级）。
    pub fn is_available(&self) -> bool {
        !matches!(self, ComponentHealth::Unhealthy(_))
    }
}

/// 单个 Agent 的健康摘要。
#[derive(Debug, Clone)]
pub struct AgentHealthSummary {
    /// Agent 标识路径。
    pub agent_path: String,
    /// Agent 角色。
    pub agent_role: String,
    /// 活性状态 (Alive/Unresponsive/Unknown)。
    pub liveness: String,
    /// 恢复状态 (Healthy/PendingRestart/Restarting/Unrecoverable)。
    pub recovery_state: AgentRecoveryState,
    /// 已重启次数。
    pub restart_count: u32,
    /// 健康评估。
    pub health: ComponentHealth,
}

/// 单个熔断器的健康摘要。
#[derive(Debug, Clone)]
pub struct CircuitBreakerHealth {
    /// 熔断器名称（通常为外部服务标识）。
    pub name: String,
    /// 当前状态 (Closed/Open/HalfOpen)。
    pub state: String,
    /// 统计信息。
    pub stats: crate::resilience::CircuitBreakerStats,
    /// 健康评估。
    pub health: ComponentHealth,
}

/// 全局健康报告。
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// 报告生成时间 (Unix 毫秒)。
    pub timestamp_ms: i64,
    /// 总体健康状态。
    pub overall: ComponentHealth,
    /// Agent 健康摘要。
    pub agents: Vec<AgentHealthSummary>,
    /// 熔断器健康摘要。
    pub circuit_breakers: Vec<CircuitBreakerHealth>,
    /// Dead Letter Queue 当前深度。
    pub dlq_depth: usize,
    /// 待处理任务数。
    pub pending_task_count: usize,
}

impl HealthReport {
    /// 从各子系统状态聚合生成健康报告。
    ///
    /// # 参数
    /// - `cb_registry`: 熔断器注册表
    /// - `agent_healths`: Agent 健康信息列表
    /// - `dlq_depth`: Dead Letter Queue 当前深度
    /// - `pending_task_count`: 待处理任务数
    pub fn aggregate(
        cb_registry: &Arc<CircuitBreakerRegistry>,
        agent_healths: Vec<AgentHealthSummary>,
        dlq_depth: usize,
        pending_task_count: usize,
    ) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        // 聚合熔断器状态
        let mut circuit_breakers = Vec::new();
        let all_cb_stats = cb_registry.all_stats();
        for (name, stats) in &all_cb_stats {
            let state_str = format!("{:?}", stats.state);
            let health = match stats.state {
                crate::resilience::CircuitState::Closed => ComponentHealth::Healthy,
                crate::resilience::CircuitState::HalfOpen => {
                    ComponentHealth::Degraded("熔断器半开，探测中".into())
                }
                crate::resilience::CircuitState::Open => ComponentHealth::Unhealthy(format!(
                    "熔断器打开，连续失败 {}",
                    stats.consecutive_failures
                )),
            };
            circuit_breakers.push(CircuitBreakerHealth {
                name: name.clone(),
                state: state_str,
                stats: stats.clone(),
                health,
            });
        }

        // 计算总体健康
        let overall = Self::compute_overall(
            &agent_healths,
            &circuit_breakers,
            dlq_depth,
            pending_task_count,
        );

        Self {
            timestamp_ms,
            overall,
            agents: agent_healths,
            circuit_breakers,
            dlq_depth,
            pending_task_count,
        }
    }

    /// 计算总体健康状态。
    ///
    /// 规则：
    /// - 任一组件 Unhealthy → 总体 Unhealthy（附原因）
    /// - 任一组件 Degraded → 总体 Degraded（附原因）
    /// - 全部 Healthy → 总体 Healthy
    fn compute_overall(
        agents: &[AgentHealthSummary],
        circuit_breakers: &[CircuitBreakerHealth],
        dlq_depth: usize,
        pending_task_count: usize,
    ) -> ComponentHealth {
        let mut degraded_reasons = Vec::new();
        let mut unhealthy_reasons = Vec::new();

        // Agent 状态
        let total_agents = agents.len();
        let unhealthy_agents = agents
            .iter()
            .filter(|a| matches!(a.health, ComponentHealth::Unhealthy(_)))
            .count();
        let degraded_agents = agents
            .iter()
            .filter(|a| matches!(a.health, ComponentHealth::Degraded(_)))
            .count();

        if unhealthy_agents > 0 {
            unhealthy_reasons.push(format!(
                "{}/{} agents unhealthy",
                unhealthy_agents, total_agents
            ));
        } else if degraded_agents > 0 {
            degraded_reasons.push(format!(
                "{}/{} agents degraded",
                degraded_agents, total_agents
            ));
        }

        // 熔断器状态
        let open_cbs: Vec<_> = circuit_breakers
            .iter()
            .filter(|cb| matches!(cb.health, ComponentHealth::Unhealthy(_)))
            .collect();
        let half_open_cbs: Vec<_> = circuit_breakers
            .iter()
            .filter(|cb| matches!(cb.health, ComponentHealth::Degraded(_)))
            .collect();

        if !open_cbs.is_empty() {
            let names: Vec<&str> = open_cbs.iter().map(|cb| cb.name.as_str()).collect();
            unhealthy_reasons.push(format!("circuit breakers open: {}", names.join(", ")));
        } else if !half_open_cbs.is_empty() {
            let names: Vec<&str> = half_open_cbs.iter().map(|cb| cb.name.as_str()).collect();
            degraded_reasons.push(format!("circuit breakers half-open: {}", names.join(", ")));
        }

        // DLQ 深度警告（超过容量 50% 视为降级）
        if dlq_depth > 250 {
            unhealthy_reasons.push(format!("DLQ depth {dlq_depth} exceeds 50% capacity"));
        } else if dlq_depth > 100 {
            degraded_reasons.push(format!("DLQ depth {dlq_depth} elevated"));
        }

        // 孤儿任务（有 pending 但无 healthy agent）
        let _ = pending_task_count; // 仅记录，不影响健康判定

        if !unhealthy_reasons.is_empty() {
            ComponentHealth::Unhealthy(unhealthy_reasons.join("; "))
        } else if !degraded_reasons.is_empty() {
            ComponentHealth::Degraded(degraded_reasons.join("; "))
        } else {
            ComponentHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resilience::CircuitBreakerRegistry;
    use crate::resilience::RecoveryContext;

    fn test_agent_health(
        path: &str,
        role: &str,
        liveness: &str,
        recovery_state: AgentRecoveryState,
        restart_count: u32,
        health: ComponentHealth,
    ) -> AgentHealthSummary {
        AgentHealthSummary {
            agent_path: path.to_string(),
            agent_role: role.to_string(),
            liveness: liveness.to_string(),
            recovery_state,
            restart_count,
            health,
        }
    }

    #[test]
    fn component_health_predicates() {
        assert!(ComponentHealth::Healthy.is_healthy());
        assert!(ComponentHealth::Healthy.is_available());
        assert!(!ComponentHealth::Degraded("slow".into()).is_healthy());
        assert!(ComponentHealth::Degraded("slow".into()).is_available());
        assert!(!ComponentHealth::Unhealthy("down".into()).is_healthy());
        assert!(!ComponentHealth::Unhealthy("down".into()).is_available());
    }

    #[test]
    fn aggregate_all_healthy() {
        let registry = Arc::new(CircuitBreakerRegistry::new());
        let agents = vec![test_agent_health(
            "agent/1",
            "search",
            "Alive",
            AgentRecoveryState::Healthy,
            0,
            ComponentHealth::Healthy,
        )];

        let report = HealthReport::aggregate(&registry, agents, 0, 0);
        assert!(report.overall.is_healthy());
        assert!(report.circuit_breakers.is_empty());
        assert_eq!(report.dlq_depth, 0);
    }

    #[test]
    fn aggregate_with_degraded_agent() {
        let registry = Arc::new(CircuitBreakerRegistry::new());
        let agents = vec![test_agent_health(
            "agent/1",
            "search",
            "Unresponsive",
            AgentRecoveryState::PendingRestart,
            1,
            ComponentHealth::Degraded("agent unresponsive".into()),
        )];

        let report = HealthReport::aggregate(&registry, agents, 0, 0);
        assert!(!report.overall.is_healthy());
        assert!(report.overall.is_available());
        assert!(matches!(report.overall, ComponentHealth::Degraded(_)));
    }

    #[test]
    fn aggregate_with_unhealthy_agent() {
        let registry = Arc::new(CircuitBreakerRegistry::new());
        let agents = vec![test_agent_health(
            "agent/1",
            "search",
            "Unknown",
            AgentRecoveryState::Unrecoverable,
            3,
            ComponentHealth::Unhealthy("agent unrecoverable".into()),
        )];

        let report = HealthReport::aggregate(&registry, agents, 0, 0);
        assert!(!report.overall.is_available());
        assert!(matches!(report.overall, ComponentHealth::Unhealthy(_)));
    }

    #[test]
    fn aggregate_with_open_circuit_breaker() {
        let registry = Arc::new(CircuitBreakerRegistry::new());
        // 触发熔断器打开
        let cb = registry.get_or_create("test_api", Default::default());
        for _ in 0..5 {
            cb.record_failure();
        }

        let report = HealthReport::aggregate(&registry, vec![], 0, 0);
        assert!(!report.overall.is_available());
        assert_eq!(report.circuit_breakers.len(), 1);
        assert!(matches!(
            report.circuit_breakers[0].health,
            ComponentHealth::Unhealthy(_)
        ));
    }

    #[test]
    fn aggregate_with_elevated_dlq() {
        let registry = Arc::new(CircuitBreakerRegistry::new());
        let report = HealthReport::aggregate(&registry, vec![], 150, 0);
        // 150 > 100 但 < 250 → Degraded
        assert!(matches!(report.overall, ComponentHealth::Degraded(_)));
    }

    #[test]
    fn aggregate_with_full_dlq() {
        let registry = Arc::new(CircuitBreakerRegistry::new());
        let report = HealthReport::aggregate(&registry, vec![], 300, 0);
        // 300 > 250 → Unhealthy
        assert!(matches!(report.overall, ComponentHealth::Unhealthy(_)));
    }

    #[test]
    fn timestamp_is_recent() {
        let registry = Arc::new(CircuitBreakerRegistry::new());
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let report = HealthReport::aggregate(&registry, vec![], 0, 0);
        // 报告时间应在当前时间的 ±2 秒内
        assert!((report.timestamp_ms - now_ms).unsigned_abs() < 2000);
    }
}
