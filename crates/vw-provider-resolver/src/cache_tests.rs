use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn get_or_init_runs_initializer_once_for_same_key() {
    super::invalidate("cache-tests-once").await;
    let calls = Arc::new(AtomicUsize::new(0));
    let first_calls = calls.clone();

    let first = super::get_or_init("cache-tests-once", || async move {
        first_calls.fetch_add(1, Ordering::SeqCst);
        Arc::new("first".to_string())
    })
    .await;
    let second_calls = calls.clone();
    let second = super::get_or_init("cache-tests-once", || async move {
        second_calls.fetch_add(1, Ordering::SeqCst);
        Arc::new("second".to_string())
    })
    .await;

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(first.as_str(), "first");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn invalidate_removes_only_matching_key() {
    super::invalidate("cache-tests-left").await;
    super::invalidate("cache-tests-right").await;

    let left = super::get_or_init("cache-tests-left", || async { Arc::new(1_u32) }).await;
    let right = super::get_or_init("cache-tests-right", || async { Arc::new(10_u32) }).await;

    super::invalidate("cache-tests-left").await;

    let new_left = super::get_or_init("cache-tests-left", || async { Arc::new(2_u32) }).await;
    let same_right = super::get_or_init("cache-tests-right", || async { Arc::new(20_u32) }).await;

    assert_eq!(*left, 1);
    assert_eq!(*new_left, 2);
    assert_eq!(*right, 10);
    assert!(Arc::ptr_eq(&right, &same_right));
}

#[tokio::test]
async fn concurrent_get_or_init_shares_initialization() {
    super::invalidate("cache-tests-concurrent").await;
    let calls = Arc::new(AtomicUsize::new(0));
    let first_calls = calls.clone();
    let second_calls = calls.clone();

    let (first, second) = tokio::join!(
        super::get_or_init("cache-tests-concurrent", || async move {
            first_calls.fetch_add(1, Ordering::SeqCst);
            Arc::new(vec![1, 2, 3])
        }),
        super::get_or_init("cache-tests-concurrent", || async move {
            second_calls.fetch_add(1, Ordering::SeqCst);
            Arc::new(vec![4, 5, 6])
        })
    );

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
#[should_panic]
async fn get_or_init_panics_when_key_is_reused_for_different_type() {
    super::invalidate("cache-tests-type-mismatch").await;
    let _ = super::get_or_init("cache-tests-type-mismatch", || async { Arc::new(42_u32) }).await;
    let _ = super::get_or_init("cache-tests-type-mismatch", || async {
        Arc::new("not a u32".to_string())
    })
    .await;
}
