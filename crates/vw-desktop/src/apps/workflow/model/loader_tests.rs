use super::*;

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
