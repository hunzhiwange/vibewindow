#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("defaults_tests"));
}

use super::*;
use iced::{Point, Size};
use serde_yaml::Value;

fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(&Value::String(key.to_string())))
        .expect("expected field")
}

#[test]
fn default_node_sizes_and_handle_sides_cover_known_types() {
    assert_eq!(default_node_size("start"), Size::new(240.0, 100.0));
    assert_eq!(default_node_size("answer"), Size::new(240.0, 96.0));
    assert_eq!(default_node_size("if-else"), Size::new(242.0, 180.0));
    assert_eq!(default_node_size("iteration"), Size::new(320.0, 220.0));
    assert_eq!(default_node_size("loop"), Size::new(320.0, 220.0));
    assert_eq!(default_node_size("custom"), Size::new(242.0, 82.0));

    assert_eq!(default_source_side("iteration"), "right");
    assert_eq!(default_source_side("loop"), "right");
    assert_eq!(default_source_side("answer"), "right");
    assert_eq!(default_target_side("start"), "left");
    assert_eq!(default_target_side("answer"), "left");
}

#[test]
fn blank_node_value_writes_default_data_geometry_and_positions() {
    let value = blank_node_value("loop", "loop-1".to_string(), Point::new(-3.0, 9.5), 12.0);

    assert_eq!(field(&value, "id").as_str(), Some("loop-1"));
    assert_eq!(field(&value, "type").as_str(), Some("custom"));
    assert_eq!(field(&value, "sourcePosition").as_str(), Some("right"));
    assert_eq!(field(&value, "targetPosition").as_str(), Some("left"));
    assert_eq!(field(&value, "width").as_f64(), Some(320.0));
    assert_eq!(field(&value, "height").as_f64(), Some(220.0));
    assert_eq!(field(&value, "zIndex").as_f64(), Some(12.0));
    assert_eq!(field(field(&value, "position"), "x").as_f64(), Some(-3.0));
    assert_eq!(field(field(&value, "positionAbsolute"), "y").as_f64(), Some(9.5));
    assert_eq!(field(field(&value, "data"), "type").as_str(), Some("loop"));
}

#[test]
fn default_node_data_value_covers_specialized_node_types() {
    let start = default_node_data_value("start");
    let start_variables =
        field(&start, "variables").as_sequence().expect("start variables should be a list");
    assert_eq!(start_variables.len(), 2);
    assert_eq!(field(&start_variables[0], "variable").as_str(), Some("query"));
    assert_eq!(field(&start_variables[1], "variable").as_str(), Some("files"));

    let answer = default_node_data_value("answer");
    assert_eq!(field(&answer, "answer").as_str(), Some("你好，这是一条新的回复节点。"));
    assert!(field(&answer, "variables").as_sequence().expect("answer vars").is_empty());

    let code = default_node_data_value("code");
    assert_eq!(field(&code, "code_language").as_str(), Some("python3"));
    assert!(field(&code, "outputs").as_mapping().expect("outputs map").is_empty());
    assert!(field(&code, "code").as_str().expect("code text").contains("def main"));

    let llm = default_node_data_value("llm");
    assert_eq!(field(field(&llm, "context"), "enabled").as_bool(), Some(false));
    assert_eq!(field(field(&llm, "vision"), "enabled").as_bool(), Some(false));
    assert_eq!(
        field(&field(&llm, "prompt_template").as_sequence().expect("prompt list")[0], "role")
            .as_str(),
        Some("system")
    );

    let if_else = default_node_data_value("if-else");
    let cases = field(&if_else, "cases").as_sequence().expect("cases should exist");
    assert_eq!(cases.len(), 2);
    assert_eq!(field(&cases[0], "case_id").as_str(), Some("true"));
    assert_eq!(field(&cases[1], "case_id").as_str(), Some("false"));

    let knowledge = default_node_data_value("knowledge-retrieval");
    assert_eq!(field(&knowledge, "retrieval_mode").as_str(), Some("multiple"));
    assert_eq!(field(field(&knowledge, "multiple_retrieval_config"), "top_k").as_i64(), Some(5));
    assert_eq!(
        field(field(field(&knowledge, "single_retrieval_config"), "model"), "mode").as_str(),
        Some("chat")
    );

    let tool = default_node_data_value("tool");
    assert_eq!(field(&tool, "tool_node_version").as_str(), Some("2"));
    assert!(field(&tool, "tool_parameters").as_mapping().expect("tool params").is_empty());

    let agent = default_node_data_value("agent");
    assert_eq!(field(&agent, "tool_node_version").as_str(), Some("2"));
    assert_eq!(field(field(field(&agent, "memory"), "window"), "size").as_i64(), Some(3));

    let iteration = default_node_data_value("iteration");
    assert_eq!(field(&iteration, "type").as_str(), Some("iteration"));
    assert_eq!(field(&iteration, "title").as_str(), Some("迭代"));

    let loop_node = default_node_data_value("loop");
    assert_eq!(field(&loop_node, "type").as_str(), Some("loop"));
    assert_eq!(field(&loop_node, "title").as_str(), Some("循环"));

    let custom = default_node_data_value("http-request");
    assert_eq!(field(&custom, "type").as_str(), Some("http-request"));
    assert_eq!(field(&custom, "title").as_str(), Some("HTTP 请求"));
}
