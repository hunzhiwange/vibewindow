use super::*;
use std::time::Duration;

#[test]
fn sliding_window_rate_limiter_rejects_after_limit() {
    let limiter = SlidingWindowRateLimiter::new(1, Duration::from_secs(60), 10);

    assert!(limiter.allow("client"));
    assert!(!limiter.allow("client"));
}

#[test]
fn sliding_window_rate_limiter_zero_limit_allows_requests() {
    let limiter = SlidingWindowRateLimiter::new(0, Duration::from_secs(60), 1);

    assert!(limiter.allow("client"));
    assert!(limiter.allow("client"));
}
