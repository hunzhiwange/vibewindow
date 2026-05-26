use super::*;

#[tokio::test]
async fn progress_helpers_emit_expected_messages() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(4);

    update_thinking_progress(Some(&tx), 0).await;
    update_tool_call_progress(Some(&tx), 2, 7).await;
    send_retry_progress(Some(&tx)).await;

    assert!(rx.recv().await.unwrap().contains(DRAFT_PROGRESS_SENTINEL));
    assert!(rx.recv().await.unwrap().contains("2 tool call"));
    assert!(rx.recv().await.unwrap().contains("Retrying"));
}

#[tokio::test]
async fn final_response_starts_by_clearing_draft() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(4);

    stream_final_response("hello world", Some(&tx), None).await.unwrap();

    assert_eq!(rx.recv().await.unwrap(), DRAFT_CLEAR_SENTINEL);
    let mut final_text = String::new();
    while let Ok(chunk) = rx.try_recv() {
        final_text.push_str(&chunk);
    }
    assert_eq!(final_text, "hello world");
}
