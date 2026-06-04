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
pub use circuit_breaker::CircuitBreaker;
pub use circuit_breaker::CircuitBreakerConfig;
pub use circuit_breaker::CircuitBreakerRegistry;
pub use circuit_breaker::CircuitBreakerStats;
pub use circuit_breaker::CircuitState;
pub use circuit_breaker::StdCircuitBreaker;
pub use client::CircuitOpenError;
pub use client::ResilientCallConfig;
pub use client::call_with_breaker;
pub use client::call_with_breaker_async;
pub use client::sleep_backoff;
pub use client::sleep_backoff_async;
pub use health::AgentHealthSummary;
pub use health::CircuitBreakerHealth;
pub use health::ComponentHealth;
pub use health::HealthReport;
pub use recovery::AgentRecoveryState;
pub use recovery::RecoveryContext;
pub use recovery::RecoveryPolicy;
