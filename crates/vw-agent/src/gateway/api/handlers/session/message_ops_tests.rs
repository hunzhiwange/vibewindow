use super::*;
use crate::app::agent::gateway::instance::InstanceQuery;
use axum::Json;
use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};

#[test]
fn message_handlers_are_available() {
    let _ = session_message_list;
    let _ = session_message_get;
    let _ = session_message_part_delete;
    let _ = session_message_part_patch;
}

#[tokio::test]
async fn session_message_part_patch_rejects_body_path_mismatch_before_storage_access() {
    let part = agent_session::message::Part::Text(agent_session::message::TextPart {
        base: agent_session::message::PartBase {
            id: "prt-body".to_string(),
            session_id: "ses-body".to_string(),
            message_id: "msg-body".to_string(),
        },
        text: "updated".to_string(),
        synthetic: None,
        ignored: None,
        time: None,
        metadata: None,
    });

    let error = session_message_part_patch(
        Path(("ses-path".to_string(), "msg-path".to_string(), "prt-path".to_string())),
        Query(InstanceQuery { directory: None }),
        HeaderMap::new(),
        Json(part),
    )
    .await
    .expect_err("mismatch should be rejected");

    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert!(error.to_string().contains("part mismatch"));
    assert!(error.to_string().contains("ses-path"));
    assert!(error.to_string().contains("msg-path"));
    assert!(error.to_string().contains("prt-path"));
}
