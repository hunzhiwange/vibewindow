//! Workflow 运行相关共享 DTO。
//!
//! 本模块只定义跨网关边界传输的数据结构，不承载执行逻辑。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

fn default_workflow_max_steps() -> u32 {
    200
}

/// Workflow 执行请求。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowRunRequest {
    /// Dify YAML 文本。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_yaml: Option<String>,
    /// 用户查询文本，会写入 `sys.query`，并在存在同名开始变量时复用。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// 开始节点输入变量。
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
    /// 最大执行节点数，防止循环或异常图结构无限运行。
    #[serde(default = "default_workflow_max_steps")]
    pub max_steps: u32,
}

impl Default for WorkflowRunRequest {
    fn default() -> Self {
        Self {
            workflow_yaml: None,
            query: None,
            inputs: BTreeMap::new(),
            max_steps: default_workflow_max_steps(),
        }
    }
}

/// Workflow 执行状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowRunStatus {
    Succeeded,
    Failed,
}

/// 单个节点执行状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowNodeRunStatus {
    Succeeded,
    Failed,
}

/// 单个节点执行结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowNodeRunDto {
    pub node_id: String,
    pub node_type: String,
    pub title: String,
    pub status: WorkflowNodeRunStatus,
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
    #[serde(default)]
    pub outputs: BTreeMap<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_handle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default)]
    pub elapsed_ms: u64,
}

/// Workflow 执行响应。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowRunResponse {
    pub run_id: String,
    pub status: WorkflowRunStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    #[serde(default)]
    pub outputs: BTreeMap<String, Value>,
    #[serde(default)]
    pub nodes: Vec<WorkflowNodeRunDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
