//! 杂项路由模块

use axum::Json;
use axum::Router;
use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use serde::Serialize;
use serde_json::Value;

use crate::app::agent::agent;
use crate::app::agent::command;
use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::instance::{InstanceQuery, resolve_directory, with_instance};
use crate::app::agent::skill;

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/doc", get(not_implemented))
        .route("/command", get(command_list))
        .route("/agent", get(agent_list))
        .route("/skill", get(skill_list))
}

pub(crate) async fn not_implemented() -> Result<Json<Value>, ApiError> {
    Err(ApiError { status: StatusCode::NOT_IMPLEMENTED, message: "not implemented".to_string() })
}

async fn command_list(
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<command::command::Info>>, ApiError> {
    let dir = resolve_directory(&query, &headers);
    let result =
        with_instance(dir, move || Box::pin(async move { Ok(command::command::list().await) }))
            .await?;
    Ok(Json(result))
}

#[axum::debug_handler]
async fn agent_list(
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<agent::Info>>, ApiError> {
    let dir = resolve_directory(&query, &headers);
    let result =
        with_instance(dir, move || Box::pin(async move { Ok(agent::list().await) })).await?;
    Ok(Json(result))
}

#[derive(Debug, Serialize)]
struct SkillInfo {
    name: String,
    description: String,
    location: String,
    content: String,
}

async fn skill_list(
    Query(query): Query<InstanceQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<SkillInfo>>, ApiError> {
    let dir = resolve_directory(&query, &headers);
    let result = with_instance(dir, move || {
        Box::pin(async move {
            let items = skill::all().await;
            Ok(items
                .into_iter()
                .map(|s| SkillInfo {
                    name: s.name,
                    description: s.description,
                    location: s.location,
                    content: s.content,
                })
                .collect::<Vec<_>>())
        })
    })
    .await?;
    Ok(Json(result))
}

#[cfg(test)]
#[path = "misc_tests.rs"]
mod misc_tests;
