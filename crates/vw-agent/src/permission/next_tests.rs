use super::*;
use serde_json::json;
use std::sync::{LazyLock, Mutex};

static STATE_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn wildcard_match_respects_simple_globs() {
    assert!(wildcard_match("shell.read", "shell.*"));
    assert!(wildcard_match("abc", "a*c"));
    assert!(!wildcard_match("abc", "a*d"));
}

#[test]
fn evaluate_defaults_to_ask_without_matching_rule() {
    let ruleset = from_config(&json!({"shell.rm": "deny", "shell.read": "ask"}));
    assert_eq!(evaluate("shell.rm", "anything", &[ruleset.clone()]).action, Action::Deny);
    assert_eq!(evaluate("shell.read", "anything", &[ruleset]).action, Action::Ask);
}

#[test]
fn from_config_parses_simple_and_pattern_rules() {
    let home_pattern = expand_home("~/secret/**");
    let dollar_home_pattern = expand_home("$HOME/cache/**");
    let ruleset = from_config(&json!({
        "read": "allow",
        "edit": {
            "src/**": "ask",
            "~/secret/**": "deny",
            "$HOME/cache/**": "allow",
            "ignored": 42
        },
        "bad": true
    }));

    assert!(
        ruleset
            .iter()
            .any(|r| { r.permission == "read" && r.pattern == "*" && r.action == Action::Allow })
    );
    assert!(
        ruleset.iter().any(|r| {
            r.permission == "edit" && r.pattern == "src/**" && r.action == Action::Ask
        })
    );
    assert!(ruleset.iter().any(|r| {
        r.permission == "edit" && r.pattern == home_pattern && r.action == Action::Deny
    }));
    assert!(ruleset.iter().any(|r| {
        r.permission == "edit" && r.pattern == dollar_home_pattern && r.action == Action::Allow
    }));
    assert!(!ruleset.iter().any(|r| r.permission == "bad"));
}

#[test]
fn evaluate_uses_later_matching_rule_priority() {
    let base = vec![Rule {
        permission: "edit".to_string(),
        pattern: "src/**".to_string(),
        action: Action::Deny,
    }];
    let override_rules = vec![Rule {
        permission: "edit".to_string(),
        pattern: "src/main.rs".to_string(),
        action: Action::Allow,
    }];

    assert_eq!(evaluate("edit", "src/main.rs", &[base, override_rules]).action, Action::Allow);
}

#[test]
fn disabled_maps_edit_tools_and_requires_global_deny() {
    let ruleset = vec![
        Rule { permission: "edit".to_string(), pattern: "*".to_string(), action: Action::Deny },
        Rule { permission: "bash".to_string(), pattern: "rm *".to_string(), action: Action::Deny },
        Rule { permission: "read".to_string(), pattern: "*".to_string(), action: Action::Allow },
    ];
    let tools =
        vec!["write".to_string(), "patch".to_string(), "bash".to_string(), "read".to_string()];

    let disabled = disabled(&tools, &ruleset);
    assert!(disabled.contains("write"));
    assert!(disabled.contains("patch"));
    assert!(!disabled.contains("bash"));
    assert!(!disabled.contains("read"));
}

fn request(patterns: Vec<&str>, always: Vec<&str>) -> Request {
    Request {
        id: String::new(),
        session_id: "session-1".to_string(),
        permission: "edit".to_string(),
        patterns: patterns.into_iter().map(str::to_string).collect(),
        metadata: Map::new(),
        always: always.into_iter().map(str::to_string).collect(),
        tool: Some(ToolInfo { message_id: "m1".to_string(), call_id: "c1".to_string() }),
    }
}

#[test]
fn ask_allows_denies_and_lists_pending_requests() {
    let _guard = STATE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset();

    let allow = vec![Rule {
        permission: "edit".to_string(),
        pattern: "src/**".to_string(),
        action: Action::Allow,
    }];
    assert!(ask(request(vec!["src/lib.rs"], vec![]), &allow).is_ok());

    let deny = vec![Rule {
        permission: "edit".to_string(),
        pattern: "secret/**".to_string(),
        action: Action::Deny,
    }];
    let err = ask(request(vec!["secret/key"], vec![]), &deny).unwrap_err();
    assert!(matches!(err, Error::Denied(_)));

    let pending = ask(request(vec!["other/file"], vec!["other/**"]), &Vec::new()).unwrap_err();
    let Error::Pending(req) = pending else {
        panic!("expected pending request");
    };
    assert!(!req.id.is_empty());
    assert_eq!(list().len(), 1);

    reset();
    assert!(list().is_empty());
}

#[test]
fn reply_handles_once_reject_corrected_always_and_unknown() {
    let _guard = STATE_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    reset();

    let pending = ask(request(vec!["other/file"], vec!["other/**"]), &Vec::new()).unwrap_err();
    let Error::Pending(req) = pending else {
        panic!("expected pending request");
    };
    let once = reply(&req.id, Reply::Once, None).unwrap().expect("request should exist");
    assert_eq!(once.patterns, vec!["other/file".to_string()]);
    assert!(reply(&req.id, Reply::Once, None).unwrap().is_none());

    let pending = ask(request(vec!["again/file"], vec!["again/**"]), &Vec::new()).unwrap_err();
    let Error::Pending(req) = pending else {
        panic!("expected pending request");
    };
    let corrected = reply(&req.id, Reply::Reject, Some("use another file".to_string()))
        .expect_err("reject with message should return corrected");
    assert!(matches!(corrected, Error::Corrected(_)));

    let pending = ask(request(vec!["always/file"], vec!["always/**"]), &Vec::new()).unwrap_err();
    let Error::Pending(req) = pending else {
        panic!("expected pending request");
    };
    assert!(reply(&req.id, Reply::Always, None).unwrap().is_some());
    assert!(ask(request(vec!["always/next"], vec![]), &Vec::new()).is_ok());

    reset();
}

#[test]
fn error_display_includes_rejection_denial_and_pending_messages() {
    let denied = Error::Denied(vec![Rule {
        permission: "bash".to_string(),
        pattern: "rm *".to_string(),
        action: Action::Deny,
    }]);
    assert!(denied.to_string().contains("prevents"));
    assert!(Error::Rejected.to_string().contains("rejected"));
    assert!(Error::Corrected("try read".to_string()).to_string().contains("try read"));
    assert!(Error::Pending(request(vec!["x"], vec![])).to_string().contains("approval"));
}
