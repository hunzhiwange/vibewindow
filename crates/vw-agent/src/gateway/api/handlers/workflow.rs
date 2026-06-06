//! Workflow 网关路由。

use axum::{
    Json, Router,
    extract::{Path, State},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::{get, post},
};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use vw_api_types::workflow::{
    WorkflowNodeRunDto, WorkflowNodeRunStatus, WorkflowRecord, WorkflowRecordDeleteResponse,
    WorkflowRecordSummary, WorkflowRecordUpsertBody, WorkflowResumeRequest, WorkflowRunRequest,
    WorkflowRunResponse, WorkflowRunStatus,
};

use crate::app::agent::gateway::ApiError;
use crate::app::agent::gateway::state::AppState;
use crate::workflow::{WorkflowRuntime, resume_workflow, run_workflow};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/workflow/resume", post(workflow_resume))
        .route("/workflow/applications/chat-messages", post(workflow_applications_chat_messages))
        .route(
            "/workflow/applications/{uuid}/chat-messages",
            post(workflow_application_chat_messages),
        )
        .merge(applications_router())
}

pub(crate) fn applications_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/workflow/applications",
            get(workflow_applications_list).post(workflow_application_create),
        )
        .route(
            "/workflow/applications/{uuid}",
            get(workflow_application_get)
                .put(workflow_application_update)
                .delete(workflow_application_delete),
        )
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum VibeWindowResponseMode {
    Streaming,
    #[default]
    Blocking,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct VibeWindowChatRequest {
    query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application_uuid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application_workflow: Option<String>,
    #[serde(default)]
    inputs: BTreeMap<String, Value>,
    #[serde(default)]
    response_mode: VibeWindowResponseMode,
    user: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    conversation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    files: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auto_generate_name: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VibeWindowUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VibeWindowResponseMetadata {
    usage: VibeWindowUsage,
    retriever_resources: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VibeWindowChatResponse {
    event: String,
    task_id: String,
    id: String,
    message_id: String,
    conversation_id: String,
    mode: String,
    answer: String,
    metadata: VibeWindowResponseMetadata,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
struct VibeWindowWorkflowStartedData {
    id: String,
    workflow_id: String,
    sequence_number: Option<u32>,
    inputs: Value,
    created_at: u64,
}

#[derive(Debug, Clone, Serialize)]
struct VibeWindowNodeStartedData {
    id: String,
    node_id: String,
    node_type: String,
    title: String,
    index: u32,
    predecessor_node_id: Option<String>,
    inputs: Option<Value>,
    created_at: u64,
    extras: HashMap<String, Value>,
    parallel_id: Option<String>,
    parallel_start_node_id: Option<String>,
    parent_parallel_id: Option<String>,
    parent_parallel_start_node_id: Option<String>,
    iteration_id: Option<String>,
    loop_id: Option<String>,
    parallel_run_id: Option<String>,
    agent_strategy: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct VibeWindowNodeFinishedData {
    id: String,
    node_id: String,
    index: u32,
    predecessor_node_id: Option<String>,
    inputs: Option<Value>,
    process_data: Option<Value>,
    outputs: Option<Value>,
    status: String,
    error: Option<String>,
    elapsed_time: Option<f64>,
    created_at: u64,
}

#[derive(Debug, Clone, Serialize)]
struct VibeWindowWorkflowFinishedData {
    id: String,
    workflow_id: String,
    status: String,
    outputs: Option<Value>,
    error: Option<String>,
    elapsed_time: Option<f64>,
    total_tokens: Option<u32>,
    total_steps: u32,
    created_at: u64,
    finished_at: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
enum VibeWindowSseEvent {
    WorkflowStarted {
        task_id: String,
        workflow_run_id: String,
        data: VibeWindowWorkflowStartedData,
    },
    NodeStarted {
        task_id: String,
        workflow_run_id: String,
        data: VibeWindowNodeStartedData,
    },
    NodeFinished {
        task_id: String,
        workflow_run_id: String,
        data: VibeWindowNodeFinishedData,
    },
    Message {
        id: String,
        task_id: String,
        message_id: String,
        conversation_id: String,
        answer: String,
        created_at: i64,
    },
    WorkflowFinished {
        task_id: String,
        workflow_run_id: String,
        data: VibeWindowWorkflowFinishedData,
    },
    MessageEnd {
        id: String,
        task_id: String,
        message_id: String,
        conversation_id: String,
        metadata: HashMap<String, Value>,
    },
    Error {
        task_id: String,
        status: u32,
        code: String,
        message: String,
    },
}

async fn workflow_applications_chat_messages(
    State(state): State<AppState>,
    Json(body): Json<VibeWindowChatRequest>,
) -> Result<Response, ApiError> {
    chat_messages_inner(state, body).await
}

async fn chat_messages_inner(
    state: AppState,
    body: VibeWindowChatRequest,
) -> Result<Response, ApiError> {
    if body.user.trim().is_empty() {
        return Err(ApiError::bad_request("user is required"));
    }
    if body.query.trim().is_empty() {
        return Err(ApiError::bad_request("query is required"));
    }

    let response_mode = body.response_mode.clone();
    let conversation_id = body.conversation_id.clone();
    let mut request = chat_request_to_workflow_request(body)?;
    hydrate_workflow_yaml_from_uuid(&mut request).await?;
    let response =
        run_workflow(workflow_runtime(&state), request).await.map_err(ApiError::bad_request)?;

    match response_mode {
        VibeWindowResponseMode::Blocking => {
            let chat_response = workflow_response_to_chat_response(&response, conversation_id);
            Ok(Json(chat_response).into_response())
        }
        VibeWindowResponseMode::Streaming => {
            Ok(workflow_response_to_sse_response(response, conversation_id))
        }
    }
}

async fn workflow_application_chat_messages(
    State(state): State<AppState>,
    Path(uuid): Path<String>,
    Json(mut body): Json<VibeWindowChatRequest>,
) -> Result<Response, ApiError> {
    apply_path_chat_workflow_uuid(&mut body, &uuid)?;
    chat_messages_inner(state, body).await
}

async fn workflow_resume(
    State(state): State<AppState>,
    Json(body): Json<WorkflowResumeRequest>,
) -> Result<Json<WorkflowRunResponse>, ApiError> {
    let runtime = workflow_runtime(&state);
    let response = resume_workflow(runtime, body).await.map_err(ApiError::bad_request)?;
    Ok(Json(response))
}

async fn workflow_applications_list() -> Result<Json<Vec<WorkflowRecordSummary>>, ApiError> {
    let records = super::workflow_store::list_records(workflow_db_path()).await?;
    Ok(Json(records))
}

async fn workflow_application_get(
    Path(uuid): Path<String>,
) -> Result<Json<WorkflowRecord>, ApiError> {
    let record = super::workflow_store::get_record(workflow_db_path(), uuid).await?;
    Ok(Json(record))
}

async fn workflow_application_create(
    Json(body): Json<WorkflowRecordUpsertBody>,
) -> Result<Json<WorkflowRecord>, ApiError> {
    let record = super::workflow_store::create_record(workflow_db_path(), body).await?;
    Ok(Json(record))
}

async fn workflow_application_update(
    Path(uuid): Path<String>,
    Json(body): Json<WorkflowRecordUpsertBody>,
) -> Result<Json<WorkflowRecord>, ApiError> {
    let record = super::workflow_store::update_record(workflow_db_path(), uuid, body).await?;
    Ok(Json(record))
}

async fn workflow_application_delete(
    Path(uuid): Path<String>,
) -> Result<Json<WorkflowRecordDeleteResponse>, ApiError> {
    let response = super::workflow_store::delete_record(workflow_db_path(), uuid).await?;
    Ok(Json(response))
}

fn apply_path_chat_workflow_uuid(
    body: &mut VibeWindowChatRequest,
    uuid: &str,
) -> Result<(), ApiError> {
    let path_uuid = validate_path_workflow_uuid(uuid)?;
    if body.application_uuid.is_some() {
        return Err(ApiError::bad_request("application_uuid is not allowed when uuid is in path"));
    }
    if body.inputs.contains_key("application_uuid") {
        return Err(ApiError::bad_request("inputs.application_uuid is not allowed; use path uuid"));
    }
    if body.inputs.contains_key("workflow_uuid") {
        return Err(ApiError::bad_request(
            "inputs.workflow_uuid is no longer supported; use path uuid",
        ));
    }
    if body
        .application_workflow
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some()
    {
        return Err(ApiError::bad_request(
            "application_workflow is not allowed when uuid is in path",
        ));
    }
    if body.inputs.contains_key("application_workflow") {
        return Err(ApiError::bad_request(
            "inputs.application_workflow is not allowed when uuid is in path",
        ));
    }
    if body.inputs.contains_key("workflow_yaml") {
        return Err(ApiError::bad_request(
            "inputs.workflow_yaml is no longer supported; use path uuid",
        ));
    }
    body.application_uuid = Some(path_uuid.to_string());
    Ok(())
}

fn validate_path_workflow_uuid(uuid: &str) -> Result<&str, ApiError> {
    let trimmed = uuid.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request("workflow uuid is required"));
    }
    Ok(trimmed)
}

async fn hydrate_workflow_yaml_from_uuid(body: &mut WorkflowRunRequest) -> Result<(), ApiError> {
    let Some(uuid) = body.workflow_uuid.clone() else {
        return Ok(());
    };
    if body.workflow_yaml.is_some() {
        return Ok(());
    }

    let record = super::workflow_store::get_record(workflow_db_path(), uuid).await?;
    body.workflow_yaml = Some(record.workflow_yaml);
    Ok(())
}

fn workflow_db_path() -> PathBuf {
    crate::global::paths().data.join("workflow").join("workflows.sqlite")
}

fn workflow_runtime(state: &AppState) -> WorkflowRuntime {
    WorkflowRuntime {
        provider: state.provider.clone(),
        knowledge_provider: Some(Arc::new(super::knowledge::knowledge_store(state))),
        document_extractor: None,
        tool_provider: None,
        agent_provider: None,
        pause_store: None,
        model: state.model.clone(),
        temperature: state.temperature,
    }
}

fn chat_request_to_workflow_request(
    body: VibeWindowChatRequest,
) -> Result<WorkflowRunRequest, ApiError> {
    let mut inputs = body.inputs;
    if inputs.contains_key("application_uuid") {
        return Err(ApiError::bad_request("application_uuid must be top-level"));
    }
    if inputs.contains_key("application_workflow") {
        return Err(ApiError::bad_request("application_workflow must be top-level"));
    }
    if inputs.contains_key("workflow_uuid") {
        return Err(ApiError::bad_request(
            "workflow_uuid is no longer supported; use application_uuid",
        ));
    }
    if inputs.contains_key("workflow_yaml") {
        return Err(ApiError::bad_request(
            "workflow_yaml is no longer supported; use application_workflow",
        ));
    }
    let workflow_uuid = normalized_optional_string(body.application_uuid);
    let workflow_yaml = normalized_optional_string(body.application_workflow);
    if workflow_uuid.is_none() && workflow_yaml.is_none() {
        return Err(ApiError::bad_request("application_uuid or application_workflow is required"));
    }
    let max_steps = inputs
        .remove("max_steps")
        .and_then(|value| value.as_u64())
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(200);

    Ok(WorkflowRunRequest {
        workflow_uuid,
        workflow_yaml,
        query: Some(body.query),
        inputs,
        max_steps,
    })
}

fn normalized_optional_string(value: Option<String>) -> Option<String> {
    value.map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn workflow_response_to_chat_response(
    response: &WorkflowRunResponse,
    conversation_id: Option<String>,
) -> VibeWindowChatResponse {
    let created_at = now_unix_i64();
    let answer = workflow_answer(response);
    VibeWindowChatResponse {
        event: "message".to_string(),
        task_id: response.run_id.clone(),
        id: response.run_id.clone(),
        message_id: response.run_id.clone(),
        conversation_id: conversation_id.unwrap_or_default(),
        mode: "advanced-chat".to_string(),
        answer,
        metadata: empty_metadata(),
        created_at,
    }
}

fn workflow_response_to_sse_response(
    response: WorkflowRunResponse,
    conversation_id: Option<String>,
) -> Response {
    let events = workflow_response_to_sse_events(response, conversation_id);
    let sse_stream = stream::iter(events.into_iter().map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
        Ok::<Event, std::convert::Infallible>(Event::default().data(data))
    }));
    Sse::new(sse_stream).into_response()
}

fn workflow_response_to_sse_events(
    response: WorkflowRunResponse,
    conversation_id: Option<String>,
) -> Vec<VibeWindowSseEvent> {
    let created_at = now_unix_u64();
    let task_id = response.run_id.clone();
    let workflow_run_id = response.run_id.clone();
    let message_id = response.run_id.clone();
    let conversation_id = conversation_id.unwrap_or_default();
    let mut events = vec![VibeWindowSseEvent::WorkflowStarted {
        task_id: task_id.clone(),
        workflow_run_id: workflow_run_id.clone(),
        data: VibeWindowWorkflowStartedData {
            id: workflow_run_id.clone(),
            workflow_id: "vibewindow-workflow".to_string(),
            sequence_number: Some(1),
            inputs: Value::Object(Default::default()),
            created_at,
        },
    }];

    for (index, node) in response.nodes.iter().enumerate() {
        let node_index = u32::try_from(index + 1).unwrap_or(u32::MAX);
        events.push(VibeWindowSseEvent::NodeStarted {
            task_id: task_id.clone(),
            workflow_run_id: workflow_run_id.clone(),
            data: node_started_data(node, node_index, created_at),
        });
        events.push(VibeWindowSseEvent::NodeFinished {
            task_id: task_id.clone(),
            workflow_run_id: workflow_run_id.clone(),
            data: node_finished_data(node, node_index, created_at),
        });
    }

    let answer = workflow_answer(&response);
    if !answer.is_empty() {
        events.push(VibeWindowSseEvent::Message {
            id: message_id.clone(),
            task_id: task_id.clone(),
            message_id: message_id.clone(),
            conversation_id: conversation_id.clone(),
            answer,
            created_at: i64::try_from(created_at).unwrap_or(i64::MAX),
        });
    }

    events.push(VibeWindowSseEvent::WorkflowFinished {
        task_id: task_id.clone(),
        workflow_run_id: workflow_run_id.clone(),
        data: workflow_finished_data(&response, created_at),
    });
    events.push(VibeWindowSseEvent::MessageEnd {
        id: message_id.clone(),
        task_id: task_id.clone(),
        message_id,
        conversation_id,
        metadata: HashMap::new(),
    });

    events
}

fn node_started_data(
    node: &WorkflowNodeRunDto,
    index: u32,
    created_at: u64,
) -> VibeWindowNodeStartedData {
    VibeWindowNodeStartedData {
        id: format!("{}-started", node.node_id),
        node_id: node.node_id.clone(),
        node_type: node.node_type.clone(),
        title: node.title.clone(),
        index,
        predecessor_node_id: None,
        inputs: Some(json!(node.inputs)),
        created_at,
        extras: HashMap::new(),
        parallel_id: None,
        parallel_start_node_id: None,
        parent_parallel_id: None,
        parent_parallel_start_node_id: None,
        iteration_id: None,
        loop_id: None,
        parallel_run_id: None,
        agent_strategy: None,
    }
}

fn node_finished_data(
    node: &WorkflowNodeRunDto,
    index: u32,
    created_at: u64,
) -> VibeWindowNodeFinishedData {
    VibeWindowNodeFinishedData {
        id: format!("{}-finished", node.node_id),
        node_id: node.node_id.clone(),
        index,
        predecessor_node_id: None,
        inputs: Some(json!(node.inputs)),
        process_data: None,
        outputs: Some(json!(node.outputs)),
        status: node_status(&node.status).to_string(),
        error: node.error.clone(),
        elapsed_time: Some(node.elapsed_ms as f64 / 1000.0),
        created_at,
    }
}

fn workflow_finished_data(
    response: &WorkflowRunResponse,
    created_at: u64,
) -> VibeWindowWorkflowFinishedData {
    VibeWindowWorkflowFinishedData {
        id: response.run_id.clone(),
        workflow_id: "vibewindow-workflow".to_string(),
        status: workflow_status(&response.status).to_string(),
        outputs: Some(json!(response.outputs)),
        error: response.error.clone(),
        elapsed_time: None,
        total_tokens: None,
        total_steps: u32::try_from(response.nodes.len()).unwrap_or(u32::MAX),
        created_at,
        finished_at: now_unix_u64(),
    }
}

fn workflow_answer(response: &WorkflowRunResponse) -> String {
    response
        .answer
        .clone()
        .or_else(|| string_output(&response.outputs, "answer"))
        .or_else(|| string_output(&response.outputs, "text"))
        .or_else(|| string_output(&response.outputs, "result"))
        .unwrap_or_default()
}

fn string_output(outputs: &BTreeMap<String, Value>, key: &str) -> Option<String> {
    outputs.get(key).and_then(Value::as_str).map(str::to_string)
}

fn empty_metadata() -> VibeWindowResponseMetadata {
    VibeWindowResponseMetadata {
        usage: VibeWindowUsage { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 },
        retriever_resources: None,
    }
}

fn workflow_status(status: &WorkflowRunStatus) -> &'static str {
    match status {
        WorkflowRunStatus::Running => "running",
        WorkflowRunStatus::Paused => "running",
        WorkflowRunStatus::Succeeded => "succeeded",
        WorkflowRunStatus::Failed => "failed",
    }
}

fn node_status(status: &WorkflowNodeRunStatus) -> &'static str {
    match status {
        WorkflowNodeRunStatus::Paused => "running",
        WorkflowNodeRunStatus::Succeeded => "succeeded",
        WorkflowNodeRunStatus::Failed => "failed",
    }
}

fn now_unix_u64() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0)
}

fn now_unix_i64() -> i64 {
    i64::try_from(now_unix_u64()).unwrap_or(i64::MAX)
}

#[cfg(test)]
#[path = "workflow_tests.rs"]
mod workflow_tests;
