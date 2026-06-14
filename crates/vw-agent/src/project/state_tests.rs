use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

#[tokio::test]
async fn create_reuses_state_for_same_root_and_name() {
    let get_state = create(
        || "root-a".to_string(),
        "state_tests_create_reuses_state",
        || async { 7_u32 },
        None::<fn(Arc<u32>) -> std::future::Ready<()>>,
    );

    let first = get_state().await;
    let second = get_state().await;
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(*first, 7);
}

#[tokio::test]
async fn create_uses_root_and_name_as_cache_key() {
    let root_a = format!("root-a-{}", Uuid::new_v4());
    let root_b = format!("root-b-{}", Uuid::new_v4());
    let get_a = create(
        move || root_a.clone(),
        "state_tests_keyed",
        || async { 1_u32 },
        None::<fn(Arc<u32>) -> std::future::Ready<()>>,
    );
    let get_b = create(
        move || root_b.clone(),
        "state_tests_keyed",
        || async { 2_u32 },
        None::<fn(Arc<u32>) -> std::future::Ready<()>>,
    );

    assert_eq!(*get_a().await, 1);
    assert_eq!(*get_b().await, 2);
}

#[tokio::test]
async fn dispose_calls_initialized_state_disposers_only_once() {
    let root = format!("dispose-root-{}", Uuid::new_v4());
    let disposed = Arc::new(AtomicUsize::new(0));

    let initialized = create(
        {
            let root = root.clone();
            move || root.clone()
        },
        "initialized",
        || async { 9_u32 },
        Some({
            let disposed = disposed.clone();
            move |_value: Arc<u32>| {
                let disposed = disposed.clone();
                async move {
                    disposed.fetch_add(1, Ordering::SeqCst);
                }
            }
        }),
    );
    let _lazy = create(
        {
            let root = root.clone();
            move || root.clone()
        },
        "lazy",
        || async { 10_u32 },
        Some({
            let disposed = disposed.clone();
            move |_value: Arc<u32>| {
                let disposed = disposed.clone();
                async move {
                    disposed.fetch_add(10, Ordering::SeqCst);
                }
            }
        }),
    );

    assert_eq!(*initialized().await, 9);
    dispose(&root).await;
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
    dispose(&root).await;
    assert_eq!(disposed.load(Ordering::SeqCst), 1);
}
