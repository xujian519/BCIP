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
pub(crate) use circuit_breaker::CircuitBreakerRegistry;
pub(crate) use recovery::RecoveryContext;
