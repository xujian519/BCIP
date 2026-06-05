use std::time::Duration;

use rand::Rng;

/// 统一的指数退避策略，替代现有5处分散实现。
///
/// 公式: `delay = base_delay_ms × multiplier^attempt × jitter`
///
/// jitter 范围: `[1.0 - jitter_range, 1.0 + jitter_range]`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExponentialBackoff {
    /// 初始延迟（毫秒）。
    pub base_delay_ms: u64,
    /// 最大延迟（毫秒）。
    pub max_delay_ms: u64,
    /// 退避倍数，通常为 2.0。
    pub multiplier: f64,
    /// jitter 范围 `[1.0 - jitter_range, 1.0 + jitter_range]`，`0.0` 表示无 jitter。
    pub jitter_range: f64,
}

impl ExponentialBackoff {
    pub fn new(base_delay_ms: u64, max_delay_ms: u64, multiplier: f64, jitter_range: f64) -> Self {
        Self {
            base_delay_ms,
            max_delay_ms,
            multiplier,
            jitter_range,
        }
    }

    /// 计算第 `attempt` 次尝试的延迟。
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let exp = self.multiplier.powi(attempt as i32);
        let raw = self.base_delay_ms as f64 * exp;
        let jitter_factor = if self.jitter_range > 0.0 {
            let r: f64 = rand::rng().random_range(-1.0..1.0);
            1.0 + r * self.jitter_range
        } else {
            1.0
        };
        let final_ms = (raw * jitter_factor) as u64;
        Duration::from_millis(final_ms.min(self.max_delay_ms))
    }

    /// 工具调用预设：短间隔、快速退避。
    ///
    /// base=100ms, max=5s, multiplier=2.0, jitter=±10%
    pub fn aggressive() -> Self {
        Self {
            base_delay_ms: 100,
            max_delay_ms: 5_000,
            multiplier: 2.0,
            jitter_range: 0.1,
        }
    }

    /// LLM/API 预设：中等间隔。
    ///
    /// base=1000ms, max=30s, multiplier=2.0, jitter=±10%
    pub fn standard() -> Self {
        Self {
            base_delay_ms: 1_000,
            max_delay_ms: 30_000,
            multiplier: 2.0,
            jitter_range: 0.1,
        }
    }

    /// 外部服务预设：长间隔、保守退避。
    ///
    /// base=2000ms, max=60s, multiplier=2.0, jitter=±15%
    pub fn conservative() -> Self {
        Self {
            base_delay_ms: 2_000,
            max_delay_ms: 60_000,
            multiplier: 2.0,
            jitter_range: 0.15,
        }
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_increases_with_attempts() {
        let bo = ExponentialBackoff::aggressive();
        let mut prev = Duration::ZERO;
        for attempt in 0..8 {
            let _d = bo.delay_for_attempt(attempt);
            // 允许 jitter 导致的偶尔回落，但整体趋势应递增
            // 多次采样取最小值来确保单调性
            let samples: Vec<Duration> = (0..20).map(|_| bo.delay_for_attempt(attempt)).collect();
            let min_for_attempt = samples.into_iter().min().unwrap();
            assert!(
                min_for_attempt >= prev,
                "attempt {attempt}: min_delay {min_for_attempt:?} < prev {prev:?}"
            );
            prev = min_for_attempt;
        }
    }

    #[test]
    fn aggressive_preset_first_delay() {
        let bo = ExponentialBackoff::aggressive();
        let d = bo.delay_for_attempt(0);
        // base=100ms, jitter=±10% → [90ms, 110ms]
        assert!(d >= Duration::from_millis(85));
        assert!(d <= Duration::from_millis(120));
    }

    #[test]
    fn standard_preset_first_delay() {
        let bo = ExponentialBackoff::standard();
        let d = bo.delay_for_attempt(0);
        // base=1000ms, jitter=±10% → [900ms, 1100ms]
        assert!(d >= Duration::from_millis(850));
        assert!(d <= Duration::from_millis(1200));
    }

    #[test]
    fn conservative_preset_first_delay() {
        let bo = ExponentialBackoff::conservative();
        let d = bo.delay_for_attempt(0);
        // base=2000ms, jitter=±15% → [1700ms, 2300ms]
        assert!(d >= Duration::from_millis(1600));
        assert!(d <= Duration::from_millis(2400));
    }

    #[test]
    fn delay_capped_at_max() {
        let bo = ExponentialBackoff {
            base_delay_ms: 1000,
            max_delay_ms: 5000,
            multiplier: 10.0,
            jitter_range: 0.0,
        };
        let d = bo.delay_for_attempt(100);
        assert!(
            d <= Duration::from_millis(5000),
            "delay {d:?} exceeds max 5000ms"
        );
    }

    #[test]
    fn no_jitter_when_zero() {
        let bo = ExponentialBackoff {
            base_delay_ms: 100,
            max_delay_ms: 100_000,
            multiplier: 2.0,
            jitter_range: 0.0,
        };
        let d1 = bo.delay_for_attempt(3);
        let d2 = bo.delay_for_attempt(3);
        assert_eq!(
            d1, d2,
            "with jitter_range=0, same attempt should yield same delay"
        );
    }

    #[test]
    fn jitter_within_range() {
        let bo = ExponentialBackoff {
            base_delay_ms: 1000,
            max_delay_ms: 100_000,
            multiplier: 1.0, // 不增长
            jitter_range: 0.1,
        };
        let lower = Duration::from_millis(890); // 1000 * 0.9
        let upper = Duration::from_millis(1110); // 1000 * 1.1
        for _ in 0..100 {
            let d = bo.delay_for_attempt(0);
            assert!(d >= lower, "delay {d:?} below lower bound {lower:?}");
            assert!(d <= upper, "delay {d:?} above upper bound {upper:?}");
        }
    }
}
