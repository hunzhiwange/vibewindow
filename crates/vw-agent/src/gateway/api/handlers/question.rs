//! 问题管理路由模块

use axum::Json;
use axum::Router;
use axum::extract::Path;
use axum::routing::{get, post};
use serde::Deserialize;

use crate::app::agent::gateway::ApiError;
use crate::app::agent::question;

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/question", get(question_list))
        .route("/question/{request_id}/reply", post(question_reply))
        .route("/question/{request_id}/reject", post(question_reject))
}

async fn question_list() -> Json<Vec<question::Request>> {
    Json(question::list())
}

#[derive(Debug, Deserialize)]
struct QuestionReplyRequest {
    answers: Vec<Vec<String>>,
}

async fn question_reply(
    Path(request_id): Path<String>,
    Json(body): Json<QuestionReplyRequest>,
) -> Result<Json<bool>, ApiError> {
    question::reply(question::ReplyInput { request_id, answers: body.answers });
    Ok(Json(true))
}

async fn question_reject(Path(request_id): Path<String>) -> Result<Json<bool>, ApiError> {
    question::reject(request_id);
    Ok(Json(true))
}

#[cfg(test)]
#[path = "question_tests.rs"]
mod question_tests;
