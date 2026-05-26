//! Workflow 图调度与节点执行。

use super::code_runner::run_code_node;
use super::conditions::select_case_handle;
use super::model::{
    WorkflowEdge, WorkflowGraph, WorkflowNode, array_field, parse_workflow_yaml, string_field,
};
use super::template::{render_template, value_to_text};
use super::variables::{VariablePool, selector_from_value};
use crate::providers::{ChatMessage, ChatRequest, Provider};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use vw_api_types::workflow::{
    WorkflowNodeRunDto, WorkflowNodeRunStatus, WorkflowRunRequest, WorkflowRunResponse,
    WorkflowRunStatus,
};

#[derive(Clone)]
pub struct WorkflowRuntime {
    pub provider: Arc<dyn Provider>,
    pub model: String,
    pub temperature: f64,
}

struct NodeExecution {
    status: WorkflowNodeRunStatus,
    inputs: BTreeMap<String, Value>,
    outputs: BTreeMap<String, Value>,
    selected_handle: Option<String>,
    answer: Option<String>,
    error: Option<String>,
    elapsed_ms: u64,
}

pub async fn run_workflow(
    runtime: WorkflowRuntime,
    request: WorkflowRunRequest,
) -> Result<WorkflowRunResponse, String> {
    let run_id = Uuid::new_v4().to_string();
    let max_steps = request.max_steps.clamp(1, 10_000);
    let source = resolve_workflow_source(&request)?;
    let graph = parse_workflow_yaml(&source)?;
    let mut pool = initialize_variable_pool(&graph, &request, &run_id);
    let mut active_nodes = graph.start_node_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut executed_nodes = BTreeSet::new();
    let mut activated_edges = BTreeSet::new();
    let mut selected_handles = BTreeMap::new();
    let mut node_results = Vec::new();
    let mut last_answer = None;
    let mut last_outputs = BTreeMap::new();

    for _ in 0..max_steps {
        let Some(node_id) = next_ready_node(
            &graph,
            &active_nodes,
            &executed_nodes,
            &activated_edges,
            &selected_handles,
        ) else {
            break;
        };
        let Some(node) = graph.nodes.get(&node_id).cloned() else {
            return Err(format!("workflow 节点不存在: {node_id}"));
        };

        let execution = execute_node(&runtime, &node, &mut pool).await;
        for (key, value) in &execution.outputs {
            pool.insert_node_output(&node.id, key, value.clone());
        }
        if let Some(handle) = execution.selected_handle.as_ref() {
            selected_handles.insert(node.id.clone(), handle.clone());
        }
        if let Some(answer) = execution.answer.clone() {
            last_answer = Some(answer);
        }
        last_outputs = execution.outputs.clone();
        let failed = execution.status == WorkflowNodeRunStatus::Failed;

        node_results.push(WorkflowNodeRunDto {
            node_id: node.id.clone(),
            node_type: node.node_type.clone(),
            title: node.title.clone(),
            status: execution.status,
            inputs: redact_map(&execution.inputs),
            outputs: redact_map(&execution.outputs),
            selected_handle: execution.selected_handle,
            error: execution.error.clone(),
            elapsed_ms: execution.elapsed_ms,
        });
        executed_nodes.insert(node.id.clone());

        if failed {
            return Ok(WorkflowRunResponse {
                run_id,
                status: WorkflowRunStatus::Failed,
                answer: last_answer,
                outputs: redact_map(&last_outputs),
                nodes: node_results,
                error: execution.error,
            });
        }

        activate_outgoing_edges(
            &graph,
            &node,
            &selected_handles,
            &mut activated_edges,
            &mut active_nodes,
        );
    }

    let pending = active_nodes.difference(&executed_nodes).next().cloned();
    if let Some(node_id) = pending {
        return Ok(WorkflowRunResponse {
            run_id,
            status: WorkflowRunStatus::Failed,
            answer: last_answer,
            outputs: redact_map(&last_outputs),
            nodes: node_results,
            error: Some(format!("workflow 执行被阻塞，节点未就绪: {node_id}")),
        });
    }

    Ok(WorkflowRunResponse {
        run_id,
        status: WorkflowRunStatus::Succeeded,
        answer: last_answer,
        outputs: redact_map(&last_outputs),
        nodes: node_results,
        error: None,
    })
}

fn resolve_workflow_source(request: &WorkflowRunRequest) -> Result<String, String> {
    if let Some(source) = request.workflow_yaml.as_deref().filter(|value| !value.trim().is_empty())
    {
        return Ok(source.to_string());
    }
    Err("workflow_yaml 必须提供".to_string())
}

fn initialize_variable_pool(
    graph: &WorkflowGraph,
    request: &WorkflowRunRequest,
    run_id: &str,
) -> VariablePool {
    let mut pool = VariablePool::default();
    let query = request
        .query
        .clone()
        .or_else(|| request.inputs.get("query").and_then(Value::as_str).map(ToOwned::to_owned))
        .unwrap_or_default();
    pool.insert_selector(&["sys".to_string(), "query".to_string()], Value::String(query.clone()));
    pool.insert_selector(
        &["sys".to_string(), "workflow_run_id".to_string()],
        Value::String(run_id.to_string()),
    );

    for start_id in &graph.start_node_ids {
        if let Some(node) = graph.nodes.get(start_id) {
            for variable in array_field(&node.data, "variables") {
                if let Some(name) = string_field(variable, "variable") {
                    let value = request
                        .inputs
                        .get(name)
                        .cloned()
                        .or_else(|| (name == "query").then(|| Value::String(query.clone())))
                        .unwrap_or(Value::Null);
                    pool.insert_node_output(start_id, name, value);
                }
            }
        }
    }
    pool
}

fn next_ready_node(
    graph: &WorkflowGraph,
    active_nodes: &BTreeSet<String>,
    executed_nodes: &BTreeSet<String>,
    activated_edges: &BTreeSet<usize>,
    selected_handles: &BTreeMap<String, String>,
) -> Option<String> {
    active_nodes
        .iter()
        .filter(|node_id| !executed_nodes.contains(*node_id))
        .find(|node_id| {
            node_ready(
                graph,
                node_id,
                active_nodes,
                executed_nodes,
                activated_edges,
                selected_handles,
            )
        })
        .cloned()
}

fn node_ready(
    graph: &WorkflowGraph,
    node_id: &str,
    active_nodes: &BTreeSet<String>,
    executed_nodes: &BTreeSet<String>,
    activated_edges: &BTreeSet<usize>,
    selected_handles: &BTreeMap<String, String>,
) -> bool {
    let incoming = graph
        .edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| edge.target == node_id)
        .collect::<Vec<_>>();
    if incoming.is_empty() {
        return true;
    }

    incoming
        .into_iter()
        .filter(|(_, edge)| {
            edge_required(graph, edge, active_nodes, executed_nodes, selected_handles)
        })
        .all(|(index, edge)| {
            executed_nodes.contains(&edge.source) && activated_edges.contains(&index)
        })
}

fn edge_required(
    graph: &WorkflowGraph,
    edge: &WorkflowEdge,
    active_nodes: &BTreeSet<String>,
    executed_nodes: &BTreeSet<String>,
    selected_handles: &BTreeMap<String, String>,
) -> bool {
    if !active_nodes.contains(&edge.source) {
        return false;
    }
    let Some(source) = graph.nodes.get(&edge.source) else {
        return false;
    };
    if !executed_nodes.contains(&edge.source) {
        return source.node_type != "if-else";
    }
    if source.node_type == "if-else" {
        return selected_handles
            .get(&edge.source)
            .is_some_and(|handle| edge.source_handle.as_deref() == Some(handle.as_str()));
    }
    true
}

fn activate_outgoing_edges(
    graph: &WorkflowGraph,
    node: &WorkflowNode,
    selected_handles: &BTreeMap<String, String>,
    activated_edges: &mut BTreeSet<usize>,
    active_nodes: &mut BTreeSet<String>,
) {
    let selected_handle = selected_handles.get(&node.id);
    for (index, edge) in graph.edges.iter().enumerate().filter(|(_, edge)| edge.source == node.id) {
        let activate = if node.node_type == "if-else" {
            selected_handle
                .is_some_and(|handle| edge.source_handle.as_deref() == Some(handle.as_str()))
        } else {
            true
        };
        if activate {
            activated_edges.insert(index);
            active_nodes.insert(edge.target.clone());
        }
    }
}

async fn execute_node(
    runtime: &WorkflowRuntime,
    node: &WorkflowNode,
    pool: &mut VariablePool,
) -> NodeExecution {
    let started = Instant::now();
    let result = match node.node_type.as_str() {
        "start" => Ok(execute_start_node(node, pool)),
        "llm" => execute_llm_node(runtime, node, pool).await,
        "if-else" => execute_if_else_node(node, pool),
        "code" => execute_code_node(node, pool).await,
        "answer" => Ok(execute_answer_node(node, pool)),
        other => Err(format!("不支持的 workflow 节点类型: {other}")),
    };

    match result {
        Ok(mut execution) => {
            execution.elapsed_ms = elapsed_ms(started);
            execution
        }
        Err(error) => NodeExecution {
            status: WorkflowNodeRunStatus::Failed,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
            selected_handle: None,
            answer: None,
            error: Some(error),
            elapsed_ms: elapsed_ms(started),
        },
    }
}

fn execute_start_node(node: &WorkflowNode, pool: &VariablePool) -> NodeExecution {
    let outputs = pool.node_outputs(&node.id);
    NodeExecution {
        status: WorkflowNodeRunStatus::Succeeded,
        inputs: BTreeMap::new(),
        outputs,
        selected_handle: None,
        answer: None,
        error: None,
        elapsed_ms: 0,
    }
}

async fn execute_llm_node(
    runtime: &WorkflowRuntime,
    node: &WorkflowNode,
    pool: &VariablePool,
) -> Result<NodeExecution, String> {
    let messages = build_llm_messages(node, pool);
    let model = node
        .data
        .get("model")
        .and_then(|model| string_field(model, "name"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(runtime.model.as_str());
    let response = runtime
        .provider
        .chat(ChatRequest { messages: &messages, tools: None }, model, runtime.temperature)
        .await
        .map_err(|error| format!("LLM 节点调用失败: {error}"))?;
    let text = response.text.unwrap_or_default();
    Ok(NodeExecution {
        status: WorkflowNodeRunStatus::Succeeded,
        inputs: BTreeMap::from([(
            "messages".to_string(),
            serde_json::to_value(&messages).unwrap_or(Value::Null),
        )]),
        outputs: BTreeMap::from([("text".to_string(), Value::String(text))]),
        selected_handle: None,
        answer: None,
        error: None,
        elapsed_ms: 0,
    })
}

fn build_llm_messages(node: &WorkflowNode, pool: &VariablePool) -> Vec<ChatMessage> {
    let mut messages = Vec::new();
    for item in array_field(&node.data, "prompt_template") {
        let role = string_field(item, "role").unwrap_or("user");
        let text = render_template(string_field(item, "text").unwrap_or_default(), pool);
        match role {
            "system" => messages.push(ChatMessage::system(text)),
            "assistant" => messages.push(ChatMessage::assistant(text)),
            _ => messages.push(ChatMessage::user(text)),
        }
    }
    if messages.is_empty() {
        let query = pool
            .get_selector(&["sys".to_string(), "query".to_string()])
            .map(value_to_text)
            .unwrap_or_default();
        messages.push(ChatMessage::user(query));
    }
    messages
}

fn execute_if_else_node(node: &WorkflowNode, pool: &VariablePool) -> Result<NodeExecution, String> {
    let selected = select_case_handle(array_field(&node.data, "cases"), pool)?;
    Ok(NodeExecution {
        status: WorkflowNodeRunStatus::Succeeded,
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("selected_handle".to_string(), Value::String(selected.clone()))]),
        selected_handle: Some(selected),
        answer: None,
        error: None,
        elapsed_ms: 0,
    })
}

async fn execute_code_node(
    node: &WorkflowNode,
    pool: &VariablePool,
) -> Result<NodeExecution, String> {
    let inputs = code_inputs(node, pool);
    let language = string_field(&node.data, "code_language").unwrap_or("python3");
    let code = string_field(&node.data, "code").ok_or_else(|| "code 节点缺少 code".to_string())?;
    let outputs = run_code_node(language, code, &inputs).await?;
    Ok(NodeExecution {
        status: WorkflowNodeRunStatus::Succeeded,
        inputs,
        outputs,
        selected_handle: None,
        answer: None,
        error: None,
        elapsed_ms: 0,
    })
}

fn code_inputs(node: &WorkflowNode, pool: &VariablePool) -> BTreeMap<String, Value> {
    array_field(&node.data, "variables")
        .iter()
        .filter_map(|variable| {
            let name = string_field(variable, "variable")?.to_string();
            let selector =
                variable.get("value_selector").map(selector_from_value).unwrap_or_default();
            let value = pool.get_selector(&selector).cloned().unwrap_or(Value::Null);
            Some((name, value))
        })
        .collect()
}

fn execute_answer_node(node: &WorkflowNode, pool: &VariablePool) -> NodeExecution {
    let answer = render_template(string_field(&node.data, "answer").unwrap_or_default(), pool);
    NodeExecution {
        status: WorkflowNodeRunStatus::Succeeded,
        inputs: BTreeMap::new(),
        outputs: BTreeMap::from([("answer".to_string(), Value::String(answer.clone()))]),
        selected_handle: None,
        answer: Some(answer),
        error: None,
        elapsed_ms: 0,
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn redact_map(values: &BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    values.iter().map(|(key, value)| (key.clone(), redact_value(key, value))).collect()
}

fn redact_value(key: &str, value: &Value) -> Value {
    let lower = key.to_ascii_lowercase();
    if ["token", "secret", "password", "api_key", "authorization", "auth", "skey"]
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Value::String("[REDACTED]".to_string());
    }
    match value {
        Value::Object(object) => Value::Object(
            object.iter().map(|(key, value)| (key.clone(), redact_value(key, value))).collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.iter().map(|item| redact_value(key, item)).collect())
        }
        other => other.clone(),
    }
}
