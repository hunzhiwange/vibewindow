use super::model::parse_workflow_yaml;
use super::template::render_template;
use super::variables::VariablePool;
use serde_json::Value;

#[test]
fn parse_workflow_yaml_reads_dify_graph() {
    let graph = parse_workflow_yaml(
        r#"
workflow:
  graph:
    nodes:
      - id: start
        data:
          title: 开始
          type: start
      - id: answer
        data:
          title: 回复
          type: answer
    edges:
      - source: start
        target: answer
        sourceHandle: source
"#,
    )
    .expect("graph");

    assert_eq!(graph.start_node_ids, vec!["start"]);
    assert_eq!(graph.nodes["answer"].node_type, "answer");
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn render_template_replaces_dify_selectors() {
    let mut pool = VariablePool::default();
    pool.insert_selector(
        &["sys".to_string(), "query".to_string()],
        Value::String("查订单".to_string()),
    );

    assert_eq!(render_template("用户: {{#sys.query#}}", &pool), "用户: 查订单");
}
