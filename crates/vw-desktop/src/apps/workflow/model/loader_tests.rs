use super::*;
use iced::{Point, Size};
use serde_yaml::Value;

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("loader_tests"));
}

#[test]
fn load_document_accepts_edges_without_ids() {
    let loaded = load_document_from_text(
        None,
        r#"
workflow:
  graph:
    nodes:
      - id: start
        data:
          title: Start
          type: start
      - id: answer
        data:
          title: Answer
          type: answer
    edges:
      - source: start
        sourceHandle: source
        target: answer
"#
        .to_string(),
    )
    .expect("workflow should load when generated DSL omits edge ids");

    assert_eq!(loaded.document.edges.len(), 1);
    assert_eq!(loaded.document.edges[0].source, "start");
    assert_eq!(loaded.document.edges[0].target, "answer");
    assert_eq!(loaded.document.edges[0].id, "start-source-answer-target-1");
}

fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(&Value::String(key.to_string())))
        .expect("expected field")
}

#[test]
fn load_document_reads_app_graph_variables_viewport_and_handles() {
    let loaded = load_document_from_text(
        Some("/tmp/original.yml".to_string()),
        r##"
app:
  name: "  "
  description: " desc "
  icon: ""
  icon_background: ""
  mode: workflow
  use_icon_as_answer_icon: true
  max_active_requests: -8
workflow:
  environment_variables:
    - id: ""
      name: "  "
      value_type: ""
      value: hello
      description: "  env desc  "
  conversation_variables:
    - id: ""
      name: "  "
      value_type: ""
      value:
        count: 1
      description: "  conv desc  "
  graph:
    viewport:
      x: 7
      y: 8
      zoom: 0
    nodes:
      - id: start
        type: custom
        selected: true
        position:
          x: 1
          y: 2
        width: 20
        height: 10
        sourcePosition: top
        targetPosition: bottom
        zIndex: 2
        data:
          title: "  "
          desc: "  trimmed  "
          type: start
          selected: false
          variables:
            - variable: q
              type: paragraph
            - label: file
              type: file
      - id: branch
        type: custom
        positionAbsolute:
          x: 20
          y: 40
        parentId: group
        width: 300
        height: 180
        sourcePosition: left
        targetPosition: right
        data:
          title: Branch
          type: if-else
          error_strategy: fail-branch
          cases:
            - case_id: "true"
            - id: custom-case
            - case_id: "long-branch-name-with-dash"
      - id: end
        type: end
        data:
          title: ""
          type: ""
    edges:
      - id: "  "
        source: start
        sourceHandle: source
        target: branch
        targetHandle: target
        selected: true
        zIndex: 5
        data:
          sourceType: ""
          targetType: ""
"##
        .to_string(),
    )
    .expect("workflow should load");

    assert_eq!(loaded.source_name, "original");
    assert_eq!(loaded.app_meta.name, "original");
    assert_eq!(loaded.app_meta.description, " desc ");
    assert_eq!(loaded.app_meta.icon, "");
    assert_eq!(loaded.app_meta.icon_background, "");
    assert_eq!(loaded.app_meta.mode, "workflow");
    assert!(loaded.app_meta.use_icon_as_answer_icon);
    assert_eq!(loaded.app_meta.max_active_requests, 0);
    assert!(loaded.had_viewport);
    assert_eq!(loaded.document.viewport, WorkflowViewport { x: 7.0, y: 8.0, zoom: 0.1 });
    assert_eq!(loaded.document.nodes.len(), 3);

    let start = loaded.document.node("start").expect("start node");
    assert_eq!(start.title, "开始");
    assert_eq!(start.description, "trimmed");
    assert_eq!(start.position, Point::new(1.0, 2.0));
    assert_eq!(start.size, Size::new(120.0, 148.0));
    assert!(start.selected);
    assert_eq!(start.source_side, WorkflowHandleSide::Top);
    assert_eq!(start.target_side, WorkflowHandleSide::Bottom);
    assert_eq!(start.source_handles.len(), 1);
    assert!(start.target_handles.is_empty());

    let branch = loaded.document.node("branch").expect("branch node");
    assert_eq!(branch.parent_id.as_deref(), Some("group"));
    assert_eq!(branch.position, Point::new(20.0, 40.0));
    assert_eq!(branch.source_side, WorkflowHandleSide::Left);
    assert_eq!(branch.target_side, WorkflowHandleSide::Right);
    assert_eq!(
        branch
            .source_handles
            .iter()
            .map(|handle| (handle.id.as_str(), handle.label.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("true", "是"),
            ("custom-case", "分支 2"),
            ("long-branch-name-with-dash", "分支 3"),
            ("fail-branch", "异常")
        ]
    );
    assert_eq!(branch.target_handles.len(), 1);

    let end = loaded.document.node("end").expect("end node");
    assert_eq!(end.block_type, "end");
    assert_eq!(end.title, "结束");
    assert!(end.source_handles.is_empty());
    assert_eq!(end.target_handles.len(), 1);

    let edge = &loaded.document.edges[0];
    assert_eq!(edge.id, "start-source-branch-target-1");
    assert_eq!(edge.source_type, "start");
    assert_eq!(edge.target_type, "if-else");
    assert!(edge.selected);
    assert_eq!(edge.z_index, 5.0);

    assert_eq!(loaded.environment_variables[0].id, "env-var");
    assert_eq!(loaded.environment_variables[0].name, "environment_var");
    assert_eq!(loaded.environment_variables[0].value_type, "string");
    assert_eq!(loaded.environment_variables[0].description, "env desc");
    assert_eq!(loaded.conversation_variables[0].id, "conversation-var");
    assert_eq!(loaded.conversation_variables[0].name, "conversation_var");
    assert_eq!(loaded.conversation_variables[0].value_type, "string");
    assert_eq!(loaded.conversation_variables[0].description, "conv desc");
}

#[test]
fn load_document_supports_root_graph_and_default_app_meta() {
    let loaded = load_document_from_text(
        None,
        r#"
graph:
  nodes:
    - id: trigger
      type: trigger-webhook
      data:
        type: ""
        title: ""
    - id: answer
      data:
        type: answer
        title: Answer
  edges:
    - id: edge-1
      source: trigger
      target: answer
      data:
        sourceType: trigger-webhook
        targetType: answer
"#
        .to_string(),
    )
    .expect("root graph should load");

    assert_eq!(loaded.source_name, "未命名应用");
    assert_eq!(loaded.app_meta, WorkflowAppMeta::default());
    assert!(!loaded.had_viewport);
    assert_eq!(loaded.document.viewport, WorkflowViewport::default());

    let trigger = loaded.document.node("trigger").expect("trigger node");
    assert_eq!(trigger.block_type, "trigger-webhook");
    assert!(trigger.target_handles.is_empty());
    assert_eq!(trigger.source_handles.len(), 1);
}

#[test]
fn load_document_reports_parse_and_shape_errors() {
    let parse_error = load_document_from_text(None, "workflow: [".to_string())
        .expect_err("invalid yaml should fail");
    assert!(parse_error.starts_with("解析 Dify DSL 失败:"));

    let shape_error = load_document_from_value(None, yaml_map(vec![("app", yaml_map(vec![]))]))
        .expect_err("missing graph should fail");
    assert_eq!(shape_error, "未找到 workflow.graph 节点，无法构建画布");
}

#[test]
fn suggested_workflow_file_name_sanitizes_titles() {
    assert_eq!(suggested_workflow_file_name(" Sales Bot "), "Sales_Bot.yml");
    assert_eq!(suggested_workflow_file_name("客户/审批:*"), "客户_审批.yml");
    assert_eq!(suggested_workflow_file_name(" !!! "), "workflow_app.yml");
}

#[test]
fn serialize_workflow_yaml_patches_root_and_removes_legacy_graph() {
    let loaded = create_blank_workflow(WorkflowAppMeta {
        name: "保存测试".to_string(),
        description: "旧描述".to_string(),
        ..WorkflowAppMeta::default()
    })
    .expect("blank workflow should load");
    let mut raw_root = loaded.raw_root.clone();
    raw_root
        .as_mapping_mut()
        .expect("root map")
        .insert(Value::String("graph".to_string()), yaml_map(vec![]));
    let environment_variables = vec![WorkflowEnvironmentVariable {
        id: "env-id".to_string(),
        name: "token".to_string(),
        value_type: "secret".to_string(),
        value: Value::String("abc".to_string()),
        description: "密钥".to_string(),
        raw_variable: Value::Null,
    }];
    let conversation_variables = vec![WorkflowConversationVariable {
        id: "conv-id".to_string(),
        name: "count".to_string(),
        value_type: "number".to_string(),
        value: serde_yaml::to_value(3).expect("number value"),
        description: "计数".to_string(),
        raw_variable: Value::Null,
    }];

    let yaml = serialize_workflow_yaml(
        &WorkflowAppMeta {
            name: "新名称".to_string(),
            description: "新描述".to_string(),
            icon: "N".to_string(),
            icon_background: "#FFFFFF".to_string(),
            mode: "advanced-chat".to_string(),
            use_icon_as_answer_icon: true,
            max_active_requests: 9,
        },
        &loaded.document,
        &environment_variables,
        &conversation_variables,
        &raw_root,
        WorkflowViewport { x: 1.0, y: 2.0, zoom: 1.5 },
    )
    .expect("workflow yaml should serialize");
    let value = serde_yaml::from_str::<Value>(&yaml).expect("serialized yaml should parse");

    assert!(
        value.as_mapping().expect("root map").get(&Value::String("graph".to_string())).is_none()
    );
    assert_eq!(field(field(&value, "app"), "name").as_str(), Some("新名称"));
    assert_eq!(
        field(field(field(&value, "workflow"), "graph"), "nodes")
            .as_sequence()
            .expect("nodes")
            .len(),
        loaded.document.nodes.len()
    );
    assert_eq!(
        field(field(field(field(&value, "workflow"), "graph"), "viewport"), "zoom").as_f64(),
        Some(1.5)
    );
    assert_eq!(
        field(
            &field(field(&value, "workflow"), "environment_variables")
                .as_sequence()
                .expect("env vars")[0],
            "value_type"
        )
        .as_str(),
        Some("secret")
    );
    assert_eq!(
        field(
            &field(field(&value, "workflow"), "conversation_variables")
                .as_sequence()
                .expect("conv vars")[0],
            "name"
        )
        .as_str(),
        Some("count")
    );
}
