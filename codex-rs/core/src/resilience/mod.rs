//! 容错与弹性机制共享原语。
//!
//! 提供统一的退避策略、熔断器、Agent恢复策略等基础设施，
//! 供 core/session/agent/workflow 等模块共享使用。

mod backoff;
mod circuit_breaker;
mod client;
mod health;
mod recovery;

pub use backoff::ExponentialBackoff;
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerRegistry, CircuitBreakerStats,
    CircuitState, StdCircuitBreaker,
};
pub use client::{
    call_with_breaker, call_with_breaker_async, sleep_backoff, sleep_backoff_async,
    CircuitOpenError, ResilientCallConfig,
};
pub use health::{AgentHealthSummary, CircuitBreakerHealth, ComponentHealth, HealthReport};
pub use recovery::{AgentRecoveryState, RecoveryContext, RecoveryPolicy};
