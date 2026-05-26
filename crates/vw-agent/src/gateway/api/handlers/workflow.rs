//! Workflow 网关路由。

use axum::{Json, Router, extract::State, routing::post};
use vw_api_types::workflow::{WorkflowRunRequest, WorkflowRunResponse};

use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::state::AppState;
use crate::workflow::{WorkflowRuntime, run_workflow};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/workflow/run", post(workflow_run))
}

async fn workflow_run(
    State(state): State<AppState>,
    Json(body): Json<WorkflowRunRequest>,
) -> Result<Json<WorkflowRunResponse>, ApiError> {
    let runtime = WorkflowRuntime {
        provider: state.provider.clone(),
        model: state.model.clone(),
        temperature: state.temperature,
    };
    let response = run_workflow(runtime, body).await.map_err(ApiError::bad_request)?;
    Ok(Json(response))
}

#[cfg(test)]
#[path = "workflow_tests.rs"]
mod workflow_tests;
