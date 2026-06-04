#![allow(dead_code)] // 注册表已接入 Session；熔断调用路径逐步落地中

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

// ── CircuitState ──

/// 熔断器状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CircuitState {
    /// 正常，请求可通过。
    Closed,
    /// 熔断中，请求被拒绝。
    Open,
    /// 半开，允许探测请求通过以测试恢复。
    HalfOpen,
}

// ── CircuitBreakerConfig ──

/// 熔断器配置。
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 触发熔断的连续失败次数。
    pub failure_threshold: u32,
    /// 触发熔断的滑动窗口内失败率（0.0~1.0），`0.0` 表示不启用窗口检查。
    pub failure_rate_threshold: f64,
    /// 滑动窗口大小（请求次数），`0` 表示不启用窗口。
    pub window_size: usize,
    /// `Open → HalfOpen` 的等待时间。
    pub reset_timeout: Duration,
    /// HalfOpen 状态允许通过的探测请求数。
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_rate_threshold: 0.5,
            window_size: 20,
            reset_timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}

// ── CircuitBreakerStats ──

/// 熔断器统计信息。
#[derive(Debug, Clone, serde::Serialize)]
pub struct CircuitBreakerStats {
    /// 服务名称。
    pub service: String,
    /// 当前状态。
    pub state: CircuitState,
    /// 总调用次数。
    pub total_calls: u64,
    /// 总失败次数。
    pub total_failures: u64,
    /// 连续失败次数。
    pub consecutive_failures: u32,
    /// 最近一次失败时间（Unix 秒）。
    pub last_failure_time: Option<i64>,
}

// ── CircuitBreaker trait ──

/// 熔断器 trait。
pub trait CircuitBreaker: Send + Sync {
    /// 当前是否允许请求通过。
    fn allow_request(&self) -> bool;
    /// 记录成功调用。
    fn record_success(&self);
    /// 记录失败调用。
    fn record_failure(&self);
    /// 获取当前状态。
    fn state(&self) -> CircuitState;
    /// 强制重置为 Closed。
    fn reset(&self);
    /// 获取统计信息。
    fn stats(&self) -> CircuitBreakerStats;
}

// ── Inner state ──

struct CircuitBreakerInner {
    state: CircuitState,
    consecutive_failures: u32,
    total_calls: u64,
    total_failures: u64,
    last_failure_at: Option<Instant>,
    opened_at: Option<Instant>,
    half_open_calls: u32,
    /// 滑动窗口内的成功/失败记录（true=成功, false=失败）。
    recent_results: VecDeque<bool>,
}

impl CircuitBreakerInner {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            total_calls: 0,
            total_failures: 0,
            last_failure_at: None,
            opened_at: None,
            half_open_calls: 0,
            recent_results: VecDeque::new(),
        }
    }
}

// ── StdCircuitBreaker ──

/// 基于 `std::sync::Mutex` 的线程安全熔断器实现。
pub struct StdCircuitBreaker {
    config: CircuitBreakerConfig,
    service_name: String,
    inner: Mutex<CircuitBreakerInner>,
}

impl StdCircuitBreaker {
    /// 创建新的熔断器。
    pub fn new(service_name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            service_name,
            inner: Mutex::new(CircuitBreakerInner::new()),
        }
    }

    fn lock_inner(&self) -> std::sync::MutexGuard<'_, CircuitBreakerInner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl CircuitBreaker for StdCircuitBreaker {
    fn allow_request(&self) -> bool {
        let mut inner = self.lock_inner();
        match inner.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // 检查是否超过 reset_timeout
                if let Some(opened) = inner.opened_at {
                    if opened.elapsed() >= self.config.reset_timeout {
                        inner.state = CircuitState::HalfOpen;
                        // 本次调用计入探测配额
                        inner.half_open_calls = 1;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => {
                if inner.half_open_calls < self.config.half_open_max_calls {
                    inner.half_open_calls += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn record_success(&self) {
        let mut inner = self.lock_inner();
        inner.total_calls += 1;
        match inner.state {
            CircuitState::Closed => {
                inner.consecutive_failures = 0;
                push_recent(&mut inner, &self.config, true);
            }
            CircuitState::HalfOpen => {
                push_recent(&mut inner, &self.config, true);
                // 探测全部成功 → 回到 Closed
                if inner.half_open_calls >= self.config.half_open_max_calls {
                    inner.state = CircuitState::Closed;
                    inner.consecutive_failures = 0;
                    inner.opened_at = None;
                }
            }
            CircuitState::Open => {}
        }
    }

    fn record_failure(&self) {
        let mut inner = self.lock_inner();
        inner.total_calls += 1;
        inner.total_failures += 1;
        inner.consecutive_failures += 1;
        inner.last_failure_at = Some(Instant::now());

        match inner.state {
            CircuitState::Closed => {
                push_recent(&mut inner, &self.config, false);

                // 连续失败达到阈值
                if inner.consecutive_failures >= self.config.failure_threshold {
                    trip_open(&mut inner);
                    return;
                }

                // 窗口失败率达到阈值
                if self.config.window_size > 0 && self.config.failure_rate_threshold > 0.0 {
                    let failures = inner.recent_results.iter().filter(|&&s| !s).count();
                    let window = inner.recent_results.len();
                    if window >= self.config.window_size {
                        let rate = failures as f64 / window as f64;
                        if rate >= self.config.failure_rate_threshold {
                            trip_open(&mut inner);
                        }
                    }
                }
            }
            CircuitState::HalfOpen => {
                // 探测失败 → 立即回到 Open
                inner.state = CircuitState::Open;
                inner.opened_at = Some(Instant::now());
            }
            CircuitState::Open => {}
        }
    }

    fn state(&self) -> CircuitState {
        self.lock_inner().state
    }

    fn reset(&self) {
        let mut inner = self.lock_inner();
        inner.state = CircuitState::Closed;
        inner.consecutive_failures = 0;
        inner.opened_at = None;
        inner.half_open_calls = 0;
        inner.recent_results.clear();
    }

    fn stats(&self) -> CircuitBreakerStats {
        let inner = self.lock_inner();
        CircuitBreakerStats {
            service: self.service_name.clone(),
            state: inner.state,
            total_calls: inner.total_calls,
            total_failures: inner.total_failures,
            consecutive_failures: inner.consecutive_failures,
            last_failure_time: inner.last_failure_at.map(|t| {
                // 近似 Unix 秒（仅用于监控展示，不需要精确）
                let elapsed = t.elapsed().as_secs() as i64;
                let now_ts = chrono::Utc::now().timestamp();
                now_ts.saturating_sub(elapsed)
            }),
        }
    }
}

/// 将最新结果推入滑动窗口。
fn push_recent(inner: &mut CircuitBreakerInner, config: &CircuitBreakerConfig, success: bool) {
    if config.window_size > 0 {
        inner.recent_results.push_back(success);
        while inner.recent_results.len() > config.window_size {
            inner.recent_results.pop_front();
        }
    }
}

/// 将熔断器从 Closed 转为 Open。
fn trip_open(inner: &mut CircuitBreakerInner) {
    inner.state = CircuitState::Open;
    inner.opened_at = Some(Instant::now());
}

// ── CircuitBreakerRegistry ──

/// 熔断器注册表 — 每个外部服务一个实例。
pub struct CircuitBreakerRegistry {
    breakers: Mutex<HashMap<String, Arc<dyn CircuitBreaker>>>,
}

impl CircuitBreakerRegistry {
    /// 创建空的注册表。
    pub fn new() -> Self {
        Self {
            breakers: Mutex::new(HashMap::new()),
        }
    }

    /// 获取指定服务的熔断器，不存在则创建。
    pub fn get_or_create(
        &self,
        service: &str,
        config: CircuitBreakerConfig,
    ) -> Arc<dyn CircuitBreaker> {
        let mut breakers = self.breakers.lock().unwrap_or_else(|e| e.into_inner());
        breakers
            .entry(service.to_string())
            .or_insert_with(|| Arc::new(StdCircuitBreaker::new(service.to_string(), config)))
            .clone()
    }

    /// 获取所有熔断器状态（用于健康检查）。
    pub fn all_stats(&self) -> HashMap<String, CircuitBreakerStats> {
        let breakers = self.breakers.lock().unwrap_or_else(|e| e.into_inner());
        breakers
            .values()
            .map(|b| {
                let s = b.stats();
                (s.service.clone(), s)
            })
            .collect()
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            failure_rate_threshold: 0.5,
            window_size: 10,
            reset_timeout: Duration::from_millis(50),
            half_open_max_calls: 2,
        }
    }

    #[test]
    fn starts_closed() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn opens_after_threshold_failures() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn open_rejects_requests() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        assert!(!cb.allow_request());
    }

    #[test]
    fn transitions_to_half_open_after_timeout() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);

        // 等待超过 reset_timeout
        std::thread::sleep(Duration::from_millis(60));
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_allows_limited_calls() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        std::thread::sleep(Duration::from_millis(60));

        // half_open_max_calls = 2
        assert!(cb.allow_request()); // call 1
        assert!(cb.allow_request()); // call 2
        assert!(!cb.allow_request()); // call 3 → rejected
    }

    #[test]
    fn half_open_to_closed_on_success() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        std::thread::sleep(Duration::from_millis(60));

        assert!(cb.allow_request());
        cb.record_success();
        assert!(cb.allow_request());
        cb.record_success();

        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_to_open_on_failure() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        std::thread::sleep(Duration::from_millis(60));

        assert!(cb.allow_request());
        cb.record_failure();

        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn reset_forces_closed() {
        let cb = StdCircuitBreaker::new("test".into(), test_config());
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn window_failure_rate_triggers_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 100, // 高阈值，不靠连续失败触发
            failure_rate_threshold: 0.5,
            window_size: 10,
            reset_timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        };
        let cb = StdCircuitBreaker::new("test".into(), config);

        // 6 成功 + 4 失败 → 40% 失败率 → 不触发
        for _ in 0..6 {
            cb.record_success();
        }
        for _ in 0..4 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Closed);

        // 再加 2 次失败 → 6成功 + 6失败(窗口内10次) → 60% → 触发
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn registry_get_or_create() {
        let registry = CircuitBreakerRegistry::new();
        let cb1 = registry.get_or_create("svc-a", CircuitBreakerConfig::default());
        let cb2 = registry.get_or_create("svc-a", CircuitBreakerConfig::default());
        let cb3 = registry.get_or_create("svc-b", CircuitBreakerConfig::default());

        // 同名返回同一实例
        assert!(Arc::ptr_eq(&cb1, &cb2));
        // 不同名返回不同实例
        assert!(!Arc::ptr_eq(&cb1, &cb3));
    }

    #[test]
    fn registry_all_stats() {
        let registry = CircuitBreakerRegistry::new();
        let cb = registry.get_or_create("svc-x", CircuitBreakerConfig::default());
        cb.record_failure();

        let stats = registry.all_stats();
        assert!(stats.contains_key("svc-x"));
        assert_eq!(stats["svc-x"].total_failures, 1);
    }
}
