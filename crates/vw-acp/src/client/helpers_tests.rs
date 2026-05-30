use super::helpers::*;
use crate::types::AcpSessionOptions;

#[test]
fn session_options_meta_filters_empty_values() {
    let meta = build_session_options_meta(Some(&AcpSessionOptions {
        model: Some(" claude-sonnet ".to_string()),
        allowed_tools: Some(vec!["Read".to_string(), " ".to_string(), "Write".to_string()]),
        max_turns: Some(3),
    }))
    .expect("meta");

    let options = &meta["claudeCode"]["options"];
    assert_eq!(options["model"], " claude-sonnet ");
    assert_eq!(options["allowedTools"], serde_json::json!(["Read", "Write"]));
    assert_eq!(options["maxTurns"], 3);
}

#[test]
fn session_options_meta_omits_empty_payload() {
    assert!(build_session_options_meta(None).is_none());
    assert!(
        build_session_options_meta(Some(&AcpSessionOptions {
            model: Some(" ".to_string()),
            allowed_tools: Some(vec![" ".to_string()]),
            max_turns: None,
        }))
        .is_none()
    );
}

#[test]
fn child_exit_summary_handles_missing_status() {
    let summary = child_exit_summary(None);

    assert_eq!(summary.exit_code, None);
    assert_eq!(summary.signal, None);
}
