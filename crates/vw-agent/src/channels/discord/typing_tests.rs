use super::*;

#[test]
fn new_typing_handles_starts_empty() {
    let handles = new_typing_handles();
    assert!(handles.lock().is_empty());
}

#[tokio::test]
async fn stop_typing_without_existing_handle_is_noop() {
    let handles = new_typing_handles();

    stop_typing(&handles, "channel-1").await.expect("stop should be idempotent");

    assert!(handles.lock().is_empty());
}

#[tokio::test]
async fn start_typing_registers_handle_and_stop_removes_it() {
    let handles = new_typing_handles();

    start_typing(&handles, reqwest::Client::new(), "token".to_string(), "channel-1".to_string())
        .await
        .expect("typing task should start");

    assert!(handles.lock().contains_key("channel-1"));

    stop_typing(&handles, "channel-1").await.expect("typing task should stop");

    assert!(!handles.lock().contains_key("channel-1"));
}

#[tokio::test]
async fn start_typing_replaces_existing_handle_for_same_recipient() {
    let handles = new_typing_handles();

    start_typing(&handles, reqwest::Client::new(), "token-a".to_string(), "channel-1".to_string())
        .await
        .expect("first typing task should start");
    let first_handle = {
        let mut guard = handles.lock();
        guard.remove("channel-1").expect("first handle should be stored")
    };
    let first_id = first_handle.id();
    handles.lock().insert("channel-1".to_string(), first_handle);

    start_typing(&handles, reqwest::Client::new(), "token-b".to_string(), "channel-1".to_string())
        .await
        .expect("second typing task should replace first");

    let second_id = handles.lock().get("channel-1").expect("second handle").id();
    assert_ne!(first_id, second_id);

    stop_typing(&handles, "channel-1").await.expect("cleanup should stop replacement");
}
