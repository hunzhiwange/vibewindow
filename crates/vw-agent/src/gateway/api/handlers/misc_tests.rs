use super::*;
use axum::extract::Query;
use axum::http::HeaderMap;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[tokio::test]
async fn not_implemented_returns_501() {
    let error = not_implemented().await.expect_err("endpoint is intentionally unavailable");

    assert_eq!(error.status, axum::http::StatusCode::NOT_IMPLEMENTED);
    assert_eq!(error.message, "not implemented");
}

#[tokio::test]
async fn list_handlers_return_json_arrays() {
    let temp = tempfile::tempdir().expect("tempdir");
    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let headers = HeaderMap::new();

    let Json(commands) = command_list(query, headers.clone()).await.expect("commands should list");
    assert!(commands.iter().all(|command| !command.name.trim().is_empty()));

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(agents) = agent_list(query, headers.clone()).await.expect("agents should list");
    assert!(agents.iter().all(|agent| !agent.key.trim().is_empty()));

    let query = Query(InstanceQuery { directory: Some(temp.path().to_string_lossy().to_string()) });
    let Json(skills) = skill_list(query, headers).await.expect("skills should list");
    assert!(
        skills
            .iter()
            .all(|skill| !skill.name.trim().is_empty() && !skill.location.trim().is_empty())
    );
}
