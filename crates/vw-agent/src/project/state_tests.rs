use super::*;
use std::sync::Arc;

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
