use std::path::PathBuf;

use vw_gateway_client::{GatewayClient, GatewayEndpoint};
use vw_shared::session::ui_types::ChatSession;

use super::gateway::{GatewaySessionSeed, GatewayUiRuntime};
use super::session_store::UiSessionCreateInfo;

fn runtime(session_id: Option<&str>) -> GatewayUiRuntime {
    let client = GatewayClient::new(GatewayEndpoint::new("127.0.0.1", 42617)).unwrap();
    let mut seed = GatewaySessionSeed::new(PathBuf::from("/tmp/session-store-tests"));
    if let Some(session_id) = session_id {
        seed = seed.with_id(Some(session_id.to_string()));
    }
    GatewayUiRuntime::new(client, seed)
}

fn empty_session(id: &str) -> ChatSession {
    ChatSession {
        id: id.to_string(),
        title: "Title".to_string(),
        messages: Vec::new(),
        message_ids: Vec::new(),
        calls: Vec::new(),
        steps: Vec::new(),
        created_ms: 1,
        updated_ms: 2,
    }
}

#[test]
fn session_create_info_deserializes_gateway_response_shape() {
    let info: UiSessionCreateInfo =
        serde_json::from_value(serde_json::json!({"id": "s1", "title": "New"})).unwrap();

    assert_eq!(info.id, "s1");
    assert_eq!(info.title, "New");
}

#[tokio::test]
async fn load_preview_and_path_require_explicit_or_bound_session_id() {
    let runtime = runtime(None);

    assert_eq!(
        runtime.session_ui_load(None).await.unwrap_err(),
        "gateway runtime session id is required"
    );
    assert_eq!(
        runtime.session_ui_load_any(Some("  ")).await.unwrap_err(),
        "gateway runtime session id is required"
    );
    assert_eq!(
        runtime.session_preview_meta(None).await.unwrap_err(),
        "gateway runtime session id is required"
    );
    assert_eq!(
        runtime.session_path(None).await.unwrap_err(),
        "gateway runtime session id is required"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn blocking_load_wrappers_return_session_id_errors_before_network() {
    let runtime = runtime(None);

    assert_eq!(
        runtime.session_ui_load_blocking(None).unwrap_err(),
        "gateway runtime session id is required"
    );
    assert_eq!(
        runtime.session_ui_load_any_blocking(Some(" ")).unwrap_err(),
        "gateway runtime session id is required"
    );
    assert_eq!(
        runtime.session_preview_meta_blocking(None).unwrap_err(),
        "gateway runtime session id is required"
    );
    assert_eq!(
        runtime.session_path_blocking(None).unwrap_err(),
        "gateway runtime session id is required"
    );
}

#[tokio::test]
async fn save_rejects_blank_session_id_before_network() {
    let runtime = runtime(Some("bound"));

    assert_eq!(
        runtime.session_ui_save(&empty_session("  ")).await.unwrap_err(),
        "session_ui save requires a non-empty session id"
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn blocking_save_rejects_blank_session_id_before_network() {
    let runtime = runtime(Some("bound"));

    assert_eq!(
        runtime.session_ui_save_blocking(&empty_session("")).unwrap_err(),
        "session_ui save requires a non-empty session id"
    );
}
