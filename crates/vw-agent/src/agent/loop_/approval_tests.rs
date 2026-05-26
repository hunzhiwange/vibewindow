use super::*;

#[test]
fn non_cli_approval_context_keeps_sender_and_reply_target() {
    let (prompt_tx, mut prompt_rx) = tokio::sync::mpsc::unbounded_channel();
    let context = NonCliApprovalContext {
        sender: "alice".to_string(),
        reply_target: "thread-1".to_string(),
        prompt_tx,
    };

    context
        .prompt_tx
        .send(NonCliApprovalPrompt {
            request_id: "req-1".to_string(),
            tool_name: "shell".to_string(),
            arguments: serde_json::json!({"command": "pwd"}),
        })
        .unwrap();

    let prompt = prompt_rx.try_recv().unwrap();
    assert_eq!(context.sender, "alice");
    assert_eq!(context.reply_target, "thread-1");
    assert_eq!(prompt.request_id, "req-1");
    assert_eq!(prompt.tool_name, "shell");
    assert_eq!(prompt.arguments["command"], "pwd");
}
