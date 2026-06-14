use super::*;
use agent_client_protocol::{
    PermissionOption, PermissionOptionKind, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome,
};
use serde_json::{Value, json};

use crate::types::{NonInteractivePermissionPolicy, PermissionMode};

fn option(option_id: &'static str, kind: PermissionOptionKind) -> PermissionOption {
    PermissionOption::new(option_id, option_id, kind)
}

fn allow_once() -> PermissionOption {
    option("allow-once", PermissionOptionKind::AllowOnce)
}

fn allow_always() -> PermissionOption {
    option("allow-always", PermissionOptionKind::AllowAlways)
}

fn reject_once() -> PermissionOption {
    option("reject-once", PermissionOptionKind::RejectOnce)
}

fn reject_always() -> PermissionOption {
    option("reject-always", PermissionOptionKind::RejectAlways)
}

fn tool_call(title: Option<&str>, kind: Option<&str>) -> Value {
    let mut value = json!({
        "toolCallId": "tool-1",
    });
    if let Some(title) = title {
        value["title"] = json!(title);
    }
    if let Some(kind) = kind {
        value["kind"] = json!(kind);
    }
    value
}

fn request_with_options(
    title: Option<&str>,
    kind: Option<&str>,
    options: Vec<PermissionOption>,
) -> RequestPermissionRequest {
    request_with_tool_call(tool_call(title, kind), options)
}

fn request_with_tool_call(
    tool_call: Value,
    options: Vec<PermissionOption>,
) -> RequestPermissionRequest {
    serde_json::from_value(json!({
        "sessionId": "session-1",
        "toolCall": tool_call,
        "options": options,
    }))
    .expect("permission request fixture should match ACP schema")
}

fn request(title: Option<&str>, kind: Option<&str>) -> RequestPermissionRequest {
    request_with_options(title, kind, vec![allow_once(), reject_once()])
}

fn selected_response(option_id: &str) -> RequestPermissionResponse {
    serde_json::from_value(json!({
        "outcome": {
            "outcome": "selected",
            "optionId": option_id,
        }
    }))
    .expect("selected response fixture should match ACP schema")
}

fn cancelled_response() -> RequestPermissionResponse {
    serde_json::from_value(json!({
        "outcome": {
            "outcome": "cancelled",
        }
    }))
    .expect("cancelled response fixture should match ACP schema")
}

fn selected_option_id(response: &RequestPermissionResponse) -> Option<String> {
    match &response.outcome {
        RequestPermissionOutcome::Selected(SelectedPermissionOutcome { option_id, .. }) => {
            Some(option_id.to_string())
        }
        RequestPermissionOutcome::Cancelled => None,
        _ => None,
    }
}

#[test]
fn permission_mode_satisfies_uses_rank_order() {
    assert!(permission_mode_satisfies(PermissionMode::ApproveAll, PermissionMode::DenyAll));
    assert!(permission_mode_satisfies(PermissionMode::ApproveReads, PermissionMode::ApproveReads));
    assert!(!permission_mode_satisfies(PermissionMode::ApproveReads, PermissionMode::ApproveAll));
    assert!(!permission_mode_satisfies(PermissionMode::DenyAll, PermissionMode::ApproveReads));
}

#[test]
fn infer_tool_kind_from_title_maps_known_command_heads() {
    let cases = [
        ("Read: file", Some("read")),
        ("cat config", Some("read")),
        ("search: files", Some("search")),
        ("find: files", Some("search")),
        ("grep: files", Some("search")),
        ("write: file", Some("edit")),
        ("edit: file", Some("edit")),
        ("patch: file", Some("edit")),
        ("delete: file", Some("delete")),
        ("remove: file", Some("delete")),
        ("move: file", Some("move")),
        ("rename: file", Some("move")),
        ("run: build", Some("execute")),
        ("execute: build", Some("execute")),
        ("bash: build", Some("execute")),
        ("fetch: url", Some("fetch")),
        ("http: get", Some("fetch")),
        ("url: get", Some("fetch")),
        ("think: plan", Some("think")),
        ("unknown: thing", Some("other")),
        ("   ", None),
    ];

    for (title, expected) in cases {
        assert_eq!(infer_tool_kind_from_title(title), expected);
    }
}

#[test]
fn infer_tool_kind_from_title_only_uses_text_before_colon() {
    assert_eq!(infer_tool_kind_from_title("unknown: thing"), Some("other"));
    assert_eq!(infer_tool_kind_from_title("unknown: read file"), Some("other"));
}

#[test]
fn read_and_search_kinds_are_auto_approved() {
    assert!(is_auto_approved_read_kind(Some("read")));
    assert!(is_auto_approved_read_kind(Some("search")));
    assert!(!is_auto_approved_read_kind(Some("edit")));
    assert!(!is_auto_approved_read_kind(None));
}

#[test]
fn pick_option_uses_kind_priority_and_reports_missing_matches() {
    let options = vec![allow_always(), reject_always()];

    let selected = pick_option(
        &options,
        &[PermissionOptionKind::AllowOnce, PermissionOptionKind::AllowAlways],
    )
    .expect("allow_always should be the first available allowed kind");
    assert_eq!(selected.option_id.to_string(), "allow-always");

    assert!(pick_option(&options, &[PermissionOptionKind::RejectOnce]).is_none());
}

#[test]
fn infer_tool_kind_prefers_explicit_kind_and_falls_back_to_title() {
    let explicit = request(Some("read: file"), Some("execute"));
    assert_eq!(tool_call_string_field(&explicit, "kind").as_deref(), Some("execute"));
    assert_eq!(infer_tool_kind(&explicit).as_deref(), Some("execute"));

    let from_title = request(Some("find: file"), None);
    assert_eq!(tool_call_string_field(&from_title, "missing"), None);
    assert_eq!(infer_tool_kind(&from_title).as_deref(), Some("search"));

    let missing = request(None, None);
    assert_eq!(infer_tool_kind(&missing), None);
}

#[test]
fn infer_tool_kind_handles_missing_tool_fields_and_empty_title_heads() {
    let missing = request(None, None);
    assert_eq!(tool_call_string_field(&missing, "title"), None);
    assert_eq!(tool_call_string_field(&missing, "kind"), None);
    assert_eq!(infer_tool_kind(&missing), None);

    let empty_head = request(Some("  : read file"), None);
    assert_eq!(infer_tool_kind(&empty_head), None);
}

#[test]
fn prompt_for_tool_permission_returns_false_when_prompt_is_unavailable() {
    if can_prompt_for_permission() {
        return;
    }

    assert!(!prompt_for_tool_permission(&request(Some("edit: file"), Some("edit"))).unwrap());
    assert!(!prompt_for_tool_permission(&request(None, None)).unwrap());
}

#[test]
fn response_for_prompt_decision_selects_matching_prompt_options() {
    let allow = allow_once();
    let reject = reject_once();

    assert_eq!(
        selected_option_id(&response_for_prompt_decision(true, Some(&allow), Some(&reject)))
            .as_deref(),
        Some("allow-once")
    );
    assert_eq!(
        selected_option_id(&response_for_prompt_decision(false, Some(&allow), Some(&reject)))
            .as_deref(),
        Some("reject-once")
    );
    assert!(selected_option_id(&response_for_prompt_decision(true, None, Some(&reject))).is_none());
    assert!(selected_option_id(&response_for_prompt_decision(false, Some(&allow), None)).is_none());
}

#[test]
fn resolve_permission_request_cancels_when_no_options_are_available() {
    let response = resolve_permission_request(
        &request_with_options(Some("read: file"), Some("read"), Vec::new()),
        PermissionMode::ApproveAll,
        None,
    )
    .unwrap();

    assert!(selected_option_id(&response).is_none());
}

#[test]
fn resolve_permission_request_approve_all_prefers_allow_or_first_option() {
    let preferred = resolve_permission_request(
        &request_with_options(
            Some("edit: file"),
            Some("edit"),
            vec![reject_once(), allow_always()],
        ),
        PermissionMode::ApproveAll,
        None,
    )
    .unwrap();
    assert_eq!(selected_option_id(&preferred).as_deref(), Some("allow-always"));

    let fallback = resolve_permission_request(
        &request_with_options(Some("edit: file"), Some("edit"), vec![reject_once()]),
        PermissionMode::ApproveAll,
        None,
    )
    .unwrap();
    assert_eq!(selected_option_id(&fallback).as_deref(), Some("reject-once"));
}

#[test]
fn resolve_permission_request_deny_all_rejects_or_cancels() {
    let rejected = resolve_permission_request(
        &request_with_options(
            Some("read: file"),
            Some("read"),
            vec![allow_once(), reject_always()],
        ),
        PermissionMode::DenyAll,
        None,
    )
    .unwrap();
    assert_eq!(selected_option_id(&rejected).as_deref(), Some("reject-always"));

    let cancelled = resolve_permission_request(
        &request_with_options(Some("read: file"), Some("read"), vec![allow_once()]),
        PermissionMode::DenyAll,
        None,
    )
    .unwrap();
    assert!(selected_option_id(&cancelled).is_none());
}

#[test]
fn resolve_permission_request_auto_approves_read_like_requests() {
    let explicit_read = resolve_permission_request(
        &request(Some("edit: file"), Some("read")),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Deny),
    )
    .unwrap();
    assert_eq!(selected_option_id(&explicit_read).as_deref(), Some("allow-once"));

    let inferred_search = resolve_permission_request(
        &request(Some("grep: files"), None),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Deny),
    )
    .unwrap();
    assert_eq!(selected_option_id(&inferred_search).as_deref(), Some("allow-once"));

    let no_allow_option = resolve_permission_request(
        &request_with_options(Some("read: file"), Some("read"), vec![reject_once()]),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Deny),
    )
    .unwrap();
    assert_eq!(selected_option_id(&no_allow_option).as_deref(), Some("reject-once"));
}

#[test]
fn resolve_permission_request_non_interactive_policy_denies_or_fails() {
    let denied = resolve_permission_request(
        &request(Some("edit: file"), Some("edit")),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Deny),
    )
    .unwrap();
    assert_eq!(selected_option_id(&denied).as_deref(), Some("reject-once"));

    let cancelled = resolve_permission_request(
        &request_with_options(Some("edit: file"), Some("edit"), vec![allow_once()]),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Deny),
    )
    .unwrap();
    assert!(selected_option_id(&cancelled).is_none());

    let error = resolve_permission_request(
        &request(Some("edit: file"), Some("edit")),
        PermissionMode::ApproveReads,
        Some(NonInteractivePermissionPolicy::Fail),
    )
    .expect_err("fail policy should report unavailable permission prompt");
    assert!(!error.to_string().is_empty());
}

#[test]
fn resolve_permission_request_without_prompt_rejects_or_cancels() {
    if can_prompt_for_permission() {
        return;
    }

    let rejected = resolve_permission_request(
        &request(Some("edit: file"), Some("edit")),
        PermissionMode::ApproveReads,
        None,
    )
    .unwrap();
    assert_eq!(selected_option_id(&rejected).as_deref(), Some("reject-once"));

    let cancelled = resolve_permission_request(
        &request_with_options(Some("edit: file"), Some("edit"), vec![allow_once()]),
        PermissionMode::ApproveReads,
        None,
    )
    .unwrap();
    assert!(selected_option_id(&cancelled).is_none());
}

#[test]
fn classify_permission_decision_maps_selected_options_and_cancellations() {
    let params = request_with_options(
        Some("edit: file"),
        Some("edit"),
        vec![allow_once(), allow_always(), reject_once(), reject_always()],
    );

    assert_eq!(
        classify_permission_decision(&params, &selected_response("allow-once")),
        PermissionDecision::Approved
    );
    assert_eq!(
        classify_permission_decision(&params, &selected_response("allow-always")),
        PermissionDecision::Approved
    );
    assert_eq!(
        classify_permission_decision(&params, &selected_response("reject-once")),
        PermissionDecision::Denied
    );
    assert_eq!(
        classify_permission_decision(&params, &selected_response("reject-always")),
        PermissionDecision::Denied
    );
    assert_eq!(
        classify_permission_decision(&params, &selected_response("missing")),
        PermissionDecision::Cancelled
    );
    assert_eq!(
        classify_permission_decision(&params, &cancelled_response()),
        PermissionDecision::Cancelled
    );
}
