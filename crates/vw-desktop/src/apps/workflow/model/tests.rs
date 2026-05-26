//! 工作流模型测试，验证节点、连接和应用元数据的数据结构行为。

use super::*;
use iced::{Point, Size};
use serde_yaml::Value;

fn workflow_node_with_raw(block_type: &str, raw_node: Value) -> WorkflowNode {
    WorkflowNode {
        id: "node_start".to_string(),
        block_type: block_type.to_string(),
        title: "节点".to_string(),
        description: String::new(),
        position: Point::new(0.0, 0.0),
        size: Size::new(180.0, 120.0),
        parent_id: None,
        selected: false,
        source_side: WorkflowHandleSide::Right,
        target_side: WorkflowHandleSide::Left,
        source_handles: Vec::new(),
        target_handles: Vec::new(),
        z_index: 0.0,
        raw_node,
    }
}

#[test]
fn start_node_variables_prefer_variable_name_and_map_value_types() {
    let raw_node = serde_yaml::from_str::<Value>(
        r#"
data:
  variables:
    - variable: user_query
      label: 用户问题
      type: paragraph
    - label: 重试次数
      type: number
    - type: file-list
"#,
    )
    .expect("start node yaml should parse");
    let node = workflow_node_with_raw("start", raw_node.clone());

    let variables = workflow_start_node_variables(&node);

    assert_eq!(variables.len(), 3);
    assert_eq!(variables[0].name, "user_query");
    assert_eq!(variables[0].value_type, "string");
    assert_eq!(variables[1].name, "重试次数");
    assert_eq!(variables[1].value_type, "number");
    assert_eq!(variables[2].name, "变量 3");
    assert_eq!(variables[2].value_type, "array[file]");
    assert_eq!(workflow_start_node_min_height(&raw_node), 190.0);
}

#[test]
fn start_node_min_height_uses_empty_baseline_without_variables() {
    let raw_node = serde_yaml::from_str::<Value>("data: {}")
        .expect("empty start node yaml should parse");

    assert_eq!(workflow_start_node_min_height(&raw_node), 120.0);
}

#[test]
fn default_start_node_data_initializes_query_and_files_variables() {
    let raw_data = default_node_data_value("start");
    let variables = workflow_start_node_variables_from_raw(&yaml_map(vec![("data", raw_data)]));

    assert_eq!(variables.len(), 2);
    assert_eq!(variables[0].name, "query");
    assert_eq!(variables[0].value_type, "string");
    assert_eq!(variables[1].name, "files");
    assert_eq!(variables[1].value_type, "array[file]");
}
