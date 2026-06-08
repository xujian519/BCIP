//! 统一 HTTP 客户端与熔断器模块。
//!
//! 提供线程安全的共享 HTTP 客户端和熔断器状态机，
//! 支持异步和阻塞两种模式。

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// 熔断器状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CbState {
    /// 正常状态，允许所有请求。
    Closed,
    /// 熔断开启，拒绝所有请求（直到超时）。
    Open,
    /// 半开状态，允许少量探测请求。
    HalfOpen,
}

/// 熔断器配置参数。
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 连续失败阈值，超过此值触发熔断。
    pub failure_threshold: u32,
    /// 熔断后重置超时（秒）。
    pub reset_timeout_secs: u64,
    /// 半开状态最大探测请求数。
    pub half_open_max: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout_secs: 30,
            half_open_max: 3,
        }
    }
}

/// 线程安全的熔断器状态机。
///
/// 状态转换：
/// - Closed (0) → 连续失败 ≥ failure_threshold → Open (1)
/// - Open (1) → reset_timeout_secs 后 → HalfOpen (2)
/// - HalfOpen (2) → half_open_max 次成功 → Closed (0)
/// - HalfOpen (2) → 任意失败 → Open (1)
pub struct CircuitBreaker {
    /// 状态: 0=Closed, 1=Open, 2=HalfOpen
    state: AtomicU32,
    /// 连续失败计数。
    consecutive_failures: AtomicU32,
    /// Open 状态下的开启时间戳（秒）。
    opened_at: AtomicU64,
    /// HalfOpen 状态下的探测调用计数。
    half_open_calls: AtomicU32,
    /// 配置参数。
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// 创建新的熔断器（使用默认配置）。
    pub fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
            consecutive_failures: AtomicU32::new(0),
            opened_at: AtomicU64::new(0),
            half_open_calls: AtomicU32::new(0),
            config: CircuitBreakerConfig::default(),
        }
    }

    /// 使用自定义配置创建熔断器。
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            state: AtomicU32::new(0),
            consecutive_failures: AtomicU32::new(0),
            opened_at: AtomicU64::new(0),
            half_open_calls: AtomicU32::new(0),
            config,
        }
    }

    /// 获取当前 UNIX 时间戳（秒）。
    fn epoch_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// 获取当前状态（含自动状态转换）。
    fn current_state(&self) -> CbState {
        let raw = self.state.load(Ordering::Relaxed);
        match raw {
            0 => CbState::Closed,
            1 => {
                // Open 状态：检查是否超时转换到 HalfOpen
                let opened = self.opened_at.load(Ordering::Relaxed);
                if opened > 0 {
                    let now = Self::epoch_secs();
                    if now.saturating_sub(opened) >= self.config.reset_timeout_secs {
                        self.state.store(2, Ordering::Relaxed);
                        self.half_open_calls.store(0, Ordering::Relaxed);
                        return CbState::HalfOpen;
                    }
                }
                CbState::Open
            }
            2 => CbState::HalfOpen,
            _ => CbState::Closed,
        }
    }

    /// 判断是否允许请求通过。
    pub fn allow_request(&self) -> bool {
        match self.current_state() {
            CbState::Closed => true,
            CbState::Open => false,
            CbState::HalfOpen => {
                let calls = self.half_open_calls.fetch_add(1, Ordering::Relaxed);
                calls < self.config.half_open_max
            }
        }
    }

    /// 记录成功请求。
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        if self.current_state() == CbState::HalfOpen {
            let calls = self.half_open_calls.load(Ordering::Relaxed);
            if calls >= self.config.half_open_max {
                // 半开状态探测成功，恢复到 Closed
                self.state.store(0, Ordering::Relaxed);
                self.opened_at.store(0, Ordering::Relaxed);
                self.half_open_calls.store(0, Ordering::Relaxed);
            }
        }
    }

    /// 记录失败请求。
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        match self.current_state() {
            CbState::Closed => {
                if failures >= self.config.failure_threshold {
                    self.trip_open();
                }
            }
            CbState::HalfOpen => {
                // 半开状态探测失败，立即回退到 Open
                self.trip_open();
            }
            CbState::Open => {}
        }
    }

    /// 触发熔断（Open 状态）。
    fn trip_open(&self) {
        self.state.store(1, Ordering::Relaxed);
        self.opened_at.store(Self::epoch_secs(), Ordering::Relaxed);
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// 进程级共享的异步 HTTP 客户端。
///
/// 使用 OnceLock 确保全局唯一实例，复用连接池。
pub struct SharedHttpClient {
    client: reqwest::Client,
}

impl SharedHttpClient {
    /// 创建新的共享客户端（使用默认配置：30s 超时，连接池复用）。
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// 创建带自定义超时的共享客户端。
    pub fn with_timeout_secs(timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// 获取底层 reqwest::Client 引用。
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl Default for SharedHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 进程级共享的阻塞 HTTP 客户端。
///
/// 使用 OnceLock 确保全局唯一实例，复用连接池。
pub struct SharedBlockingClient {
    client: reqwest::blocking::Client,
}

impl SharedBlockingClient {
    /// 创建新的共享阻塞客户端（使用默认配置：30s 超时，连接池复用）。
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self { client }
    }

    /// 创建带自定义超时的共享阻塞客户端。
    pub fn with_timeout_secs(timeout_secs: u64) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(4)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self { client }
    }

    /// 获取底层 reqwest::blocking::Client 引用。
    pub fn client(&self) -> &reqwest::blocking::Client {
        &self.client
    }
}

impl Default for SharedBlockingClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_breaker_default_config() {
        let cb = CircuitBreaker::new();
        assert!(cb.allow_request());
        cb.record_success();
        assert!(cb.allow_request());
    }

    #[test]
    fn circuit_breaker_custom_config() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout_secs: 1,
            half_open_max: 1,
        };
        let cb = CircuitBreaker::with_config(config);
        assert!(cb.allow_request());

        // 连续失败 2 次，触发熔断
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.allow_request());

        // 等待超时
        std::thread::sleep(std::time::Duration::from_secs(1));
        // 现在应该允许探测请求（HalfOpen）
        assert!(cb.allow_request());
    }

    #[test]
    fn circuit_breaker_half_open_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout_secs: 0,
            half_open_max: 2,
        };
        let cb = CircuitBreaker::with_config(config);

        // 触发熔断
        cb.record_failure();
        cb.record_failure();

        // 等待超时进入 HalfOpen
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(cb.allow_request());

        // 成功探测 2 次
        cb.record_success();
        cb.record_success();

        // 应该回到 Closed
        assert!(cb.allow_request());
    }

    #[test]
    fn circuit_breaker_half_open_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout_secs: 1,
            half_open_max: 5,
        };
        let cb = CircuitBreaker::with_config(config);

        // 触发熔断
        cb.record_failure();
        cb.record_failure();

        // 确认熔断开启
        assert!(!cb.allow_request());

        // 等待超时进入 HalfOpen
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert!(cb.allow_request());

        // HalfOpen 失败，应该回到 Open
        cb.record_failure();
        // 仍在超时窗口内，应该拒绝请求
        assert!(!cb.allow_request());
    }
}
