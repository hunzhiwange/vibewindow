use super::*;
use axum::extract::{Path, Query};
use axum::http::HeaderMap;

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[test]
fn pty_connect_query_deserializes_cursor() {
    let query: PtyConnectQuery =
        serde_json::from_value(serde_json::json!({"cursor": 42})).expect("valid query");

    assert_eq!(query.cursor, Some(42));
}

#[tokio::test]
async fn pty_get_update_and_remove_missing_session_return_expected_results() {
    let temp = tempfile::tempdir().expect("tempdir");
    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let headers = HeaderMap::new();

    let missing = pty_get(Path("missing-pty".to_string()), query, headers.clone())
        .await
        .expect_err("missing session should be a not found error");
    assert_eq!(missing.status, axum::http::StatusCode::NOT_FOUND);

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let missing = pty_update(
        Path("missing-pty".to_string()),
        query,
        headers.clone(),
        Json(pty::pty::UpdateInput { title: Some("new title".to_string()), size: None }),
    )
    .await
    .expect_err("missing update should be a not found error");
    assert_eq!(missing.status, axum::http::StatusCode::NOT_FOUND);

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(removed) = pty_remove(Path("missing-pty".to_string()), query, headers)
        .await
        .expect("missing remove is idempotent");
    assert!(removed);
}

#[cfg(all(not(target_arch = "wasm32"), unix))]
#[tokio::test]
async fn pty_create_list_update_get_and_remove_round_trip() {
    let temp = tempfile::tempdir().expect("tempdir");
    let headers = HeaderMap::new();

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(created) = pty_create(
        query,
        headers.clone(),
        Json(pty::pty::CreateInput {
            command: Some("/bin/sh".to_string()),
            args: Some(vec!["-c".to_string(), "printf gateway-pty; sleep 2".to_string()]),
            cwd: Some(temp.path().to_string_lossy().to_string()),
            title: Some("Gateway PTY".to_string()),
            env: None,
        }),
    )
    .await
    .expect("pty should create");
    assert_eq!(created.title, "Gateway PTY");

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(items) = pty_list(query, headers.clone()).await.expect("pty should list");
    assert!(items.iter().any(|item| item.id == created.id));

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(found) =
        pty_get(Path(created.id.clone()), query, headers.clone()).await.expect("pty should get");
    assert_eq!(found.id, created.id);

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(updated) = pty_update(
        Path(created.id.clone()),
        query,
        headers.clone(),
        Json(pty::pty::UpdateInput {
            title: Some("Gateway PTY Renamed".to_string()),
            size: Some(pty::pty::Size { rows: 30, cols: 100 }),
        }),
    )
    .await
    .expect("pty should update");
    assert_eq!(updated.title, "Gateway PTY Renamed");

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(removed) =
        pty_remove(Path(created.id), query, headers).await.expect("pty should remove");
    assert!(removed);
}
