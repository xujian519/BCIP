use std::sync::Arc;
use std::thread;

use codex_patent_core::http::*;
use pretty_assertions::assert_eq;

#[test]
fn circuit_breaker_starts_closed() {
    let cb = CircuitBreaker::new();
    assert!(
        cb.allow_request(),
        "new CircuitBreaker should be closed and allow requests"
    );
}

#[test]
fn circuit_breaker_opens_after_threshold() {
    let cb = CircuitBreaker::with_config(CircuitBreakerConfig {
        failure_threshold: 3,
        reset_timeout_secs: 300,
        half_open_max: 2,
    });

    // Before threshold — still allows requests
    cb.record_failure();
    assert!(cb.allow_request(), "1 failure < threshold 3, still closed");

    cb.record_failure();
    assert!(cb.allow_request(), "2 failures < threshold 3, still closed");

    // 3rd failure trips the breaker
    cb.record_failure();
    assert!(
        !cb.allow_request(),
        "3 failures ≥ threshold 3, circuit should be open"
    );
}

#[test]
fn circuit_breaker_half_open_after_timeout() {
    // reset_timeout_secs = 0 so the transition Open → HalfOpen happens immediately
    let cb = CircuitBreaker::with_config(CircuitBreakerConfig {
        failure_threshold: 2,
        reset_timeout_secs: 0,
        half_open_max: 2,
    });

    // Trip the breaker
    cb.record_failure();
    cb.record_failure();

    // With 0s timeout, allow_request transitions to HalfOpen and allows a probe
    assert!(
        cb.allow_request(),
        "timeout expired, should transition to HalfOpen and allow a probe request"
    );

    // Failure in HalfOpen should re-trip to Open (but 0s timeout → HalfOpen again)
    cb.record_failure();
    assert!(
        cb.allow_request(),
        "re-tripped to Open but 0s timeout → HalfOpen again"
    );
}

#[test]
fn circuit_breaker_resets_on_success() {
    let cb = CircuitBreaker::with_config(CircuitBreakerConfig {
        failure_threshold: 2,
        reset_timeout_secs: 0,
        half_open_max: 1,
    });

    // Trip open
    cb.record_failure();
    cb.record_failure();

    // allow_request transitions to HalfOpen (timeout=0), consumes the one probe slot
    assert!(cb.allow_request(), "should be HalfOpen now");

    // Success in HalfOpen with half_open_max=1 should close the circuit
    cb.record_success();
    assert!(
        cb.allow_request(),
        "after success in HalfOpen, circuit should close"
    );
}

#[test]
fn circuit_breaker_concurrent_access() {
    let cb = Arc::new(CircuitBreaker::with_config(CircuitBreakerConfig {
        failure_threshold: 500,
        reset_timeout_secs: 300,
        half_open_max: 200,
    }));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let cb = Arc::clone(&cb);
            thread::spawn(move || {
                for j in 0..100 {
                    if (i + j) % 2 == 0 {
                        cb.record_failure();
                    } else {
                        cb.record_success();
                    }
                    let _ = cb.allow_request();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("thread should not panic");
    }
}

#[test]
fn shared_http_client_new() {
    let client = SharedHttpClient::new();
    // Verify the inner reqwest client is usable (not a unit test on HTTP itself)
    let _ = client.client();
}
