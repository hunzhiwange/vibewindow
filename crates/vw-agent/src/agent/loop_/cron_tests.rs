use super::*;

#[test]
fn injects_delivery_for_supported_agent_cron() {
    let mut args = serde_json::json!({"prompt": "check status"});

    maybe_inject_cron_add_delivery("cron_add", &mut args, "telegram", Some("chat-1"));

    assert_eq!(args["delivery"]["mode"], "announce");
    assert_eq!(args["delivery"]["channel"], "telegram");
    assert_eq!(args["delivery"]["to"], "chat-1");
}

#[test]
fn preserves_explicit_non_announce_delivery_mode() {
    let mut args = serde_json::json!({
        "prompt": "check status",
        "delivery": {"mode": "silent"}
    });

    maybe_inject_cron_add_delivery("cron_add", &mut args, "telegram", Some("chat-1"));

    assert_eq!(args["delivery"]["mode"], "silent");
    assert!(args["delivery"].get("channel").is_none());
}
