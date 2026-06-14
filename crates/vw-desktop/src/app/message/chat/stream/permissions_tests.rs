#![allow(unused_must_use)]
#[test]
fn permissions_tests_module_is_wired() {
    assert!(module_path!().ends_with("permissions_tests"));
}

fn request(id: &str, session_id: &str) -> vw_gateway_client::PendingPermissionRequestDto {
    vw_gateway_client::PendingPermissionRequestDto {
        id: id.to_string(),
        session_id: session_id.to_string(),
        permission: "shell".to_string(),
        patterns: Vec::new(),
        metadata: serde_json::Map::new(),
        always: Vec::new(),
        tool: None,
    }
}

#[test]
fn split_auto_approved_permission_requests_keeps_other_sessions() {
    let requests = vec![request("b", "other"), request("a", "s1"), request("c", "s1")];

    let (remaining, auto_ids) =
        super::split_auto_approved_permission_requests(Some("s1"), requests);

    assert_eq!(remaining, vec![request("b", "other")]);
    assert_eq!(auto_ids, vec!["a".to_string(), "c".to_string()]);
}

#[test]
fn split_auto_approved_permission_requests_without_session_keeps_all() {
    let requests = vec![request("a", "s1")];

    let (remaining, auto_ids) = super::split_auto_approved_permission_requests(None, requests);

    assert_eq!(remaining, vec![request("a", "s1")]);
    assert!(auto_ids.is_empty());
}

#[test]
fn sync_permission_requests_sorts_and_preserves_existing_selection() {
    let (requests, selected) = super::sync_permission_requests(
        Some("b"),
        vec![request("c", "s"), request("a", "s"), request("b", "s")],
    );

    assert_eq!(
        requests.iter().map(|request| request.id.as_str()).collect::<Vec<_>>(),
        vec!["a", "b", "c"]
    );
    assert_eq!(selected.as_deref(), Some("b"));
}

#[test]
fn sync_permission_requests_selects_first_when_current_missing_or_empty() {
    let (_requests, selected) =
        super::sync_permission_requests(Some("z"), vec![request("b", "s"), request("a", "s")]);
    assert_eq!(selected.as_deref(), Some("a"));

    let (requests, selected) = super::sync_permission_requests(Some("a"), Vec::new());
    assert!(requests.is_empty());
    assert_eq!(selected, None);
}

#[test]
fn advance_permission_requests_after_reply_removes_selected_and_selects_next() {
    let (remaining, selected) = super::advance_permission_requests_after_reply(
        Some("b"),
        vec![request("a", "s"), request("b", "s"), request("c", "s")],
    );

    assert_eq!(
        remaining.iter().map(|request| request.id.as_str()).collect::<Vec<_>>(),
        vec!["a", "c"]
    );
    assert_eq!(selected.as_deref(), Some("c"));
}

#[test]
fn advance_permission_requests_after_reply_selects_previous_slot_for_last_item() {
    let (remaining, selected) = super::advance_permission_requests_after_reply(
        Some("c"),
        vec![request("a", "s"), request("b", "s"), request("c", "s")],
    );

    assert_eq!(
        remaining.iter().map(|request| request.id.as_str()).collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    assert_eq!(selected.as_deref(), Some("b"));
}

#[test]
fn handle_permission_list_loaded_applies_sorted_request_and_clears_on_error() {
    let (mut app, _task) = crate::app::App::new();

    super::handle_permission_list_loaded(&mut app, Ok(vec![request("b", "s"), request("a", "s")]));

    assert_eq!(app.permission_modal_request_id.as_deref(), Some("a"));
    assert_eq!(
        app.permission_modal_requests.iter().map(|request| request.id.as_str()).collect::<Vec<_>>(),
        vec!["a", "b"]
    );

    super::handle_permission_list_loaded(&mut app, Err("offline".to_string()));

    assert_eq!(app.permission_modal_request_id, None);
    assert!(app.permission_modal_request.is_none());
    assert!(app.permission_modal_requests.is_empty());
}

#[test]
fn handle_permission_select_request_updates_current_when_found_and_clears_when_missing() {
    let (mut app, _task) = crate::app::App::new();
    app.permission_modal_requests = vec![request("a", "s"), request("b", "s")];

    super::handle_permission_select_request(&mut app, "b".to_string());

    assert_eq!(app.permission_modal_request_id.as_deref(), Some("b"));
    assert_eq!(app.permission_modal_request.as_ref().map(|request| request.id.as_str()), Some("b"));

    super::handle_permission_select_request(&mut app, "missing".to_string());

    assert_eq!(app.permission_modal_request_id, None);
    assert!(app.permission_modal_request.is_none());
}

#[test]
fn handle_permission_approve_all_always_clears_modal_state() {
    let (mut app, _task) = crate::app::App::new();
    app.permission_modal_request_id = Some("a".to_string());
    app.permission_modal_request = Some(request("a", "s"));
    app.permission_modal_requests = vec![request("a", "s"), request("b", "s")];

    super::handle_permission_approve_all_always(&mut app);

    assert_eq!(app.permission_modal_request_id, None);
    assert!(app.permission_modal_request.is_none());
    assert!(app.permission_modal_requests.is_empty());
}
