//! Dify YAML 图结构解析。

use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct WorkflowGraph {
    pub(crate) nodes: BTreeMap<String, WorkflowNode>,
    pub(crate) edges: Vec<WorkflowEdge>,
    pub(crate) start_node_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkflowNode {
    pub(crate) id: String,
    pub(crate) node_type: String,
    pub(crate) title: String,
    pub(crate) data: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkflowEdge {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) source_handle: Option<String>,
}

pub(crate) fn parse_workflow_yaml(source: &str) -> Result<WorkflowGraph, String> {
    let root: Value = serde_yaml::from_str(source)
        .map_err(|error| format!("解析 Dify workflow YAML 失败: {error}"))?;
    let graph = root
        .get("workflow")
        .and_then(|workflow| workflow.get("graph"))
        .or_else(|| root.get("graph"))
        .ok_or_else(|| "未找到 workflow.graph".to_string())?;

    let nodes = graph
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or_else(|| "workflow.graph.nodes 必须是数组".to_string())?;
    let edges = graph.get("edges").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);

    let mut parsed_nodes = BTreeMap::new();
    let mut start_node_ids = Vec::new();
    for node in nodes {
        let id = string_field(node, "id")
            .ok_or_else(|| "workflow node 缺少 id".to_string())?
            .to_string();
        let data = node.get("data").cloned().unwrap_or_else(|| Value::Object(Default::default()));
        let node_type = string_field(&data, "type")
            .or_else(|| string_field(node, "type"))
            .unwrap_or("custom")
            .trim()
            .to_string();
        let title = string_field(&data, "title")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(node_type.as_str())
            .trim()
            .to_string();
        if node_type == "start" {
            start_node_ids.push(id.clone());
        }
        parsed_nodes.insert(id.clone(), WorkflowNode { id, node_type, title, data });
    }

    let parsed_edges = edges
        .iter()
        .filter_map(|edge| {
            let source = string_field(edge, "source")?.to_string();
            let target = string_field(edge, "target")?.to_string();
            Some(WorkflowEdge {
                source,
                target,
                source_handle: string_field(edge, "sourceHandle").map(ToOwned::to_owned),
            })
        })
        .collect::<Vec<_>>();

    if start_node_ids.is_empty() {
        return Err("workflow 缺少 start 节点".to_string());
    }

    Ok(WorkflowGraph { nodes: parsed_nodes, edges: parsed_edges, start_node_ids })
}

pub(crate) fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

pub(crate) fn array_field<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value.get(key).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}
