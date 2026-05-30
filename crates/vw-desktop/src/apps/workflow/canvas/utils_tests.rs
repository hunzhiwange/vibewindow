//! 工作流画布辅助函数测试，验证坐标、尺寸和布局计算的稳定性。

use super::*;
use serde_yaml::Value;

fn code_node(error_strategy: Option<&str>) -> WorkflowNode {
    let raw_node = match error_strategy {
        Some(strategy) => {
            serde_yaml::from_str::<Value>(&format!("data:\n  error_strategy: {strategy}\n"))
                .expect("node yaml should parse")
        }
        None => serde_yaml::from_str::<Value>("data: {}\n").expect("node yaml should parse"),
    };

    WorkflowNode {
        id: "code_1".to_string(),
        block_type: "code".to_string(),
        title: "初始化变量".to_string(),
        description: String::new(),
        position: Point::ORIGIN,
        size: Size::new(180.0, 84.0),
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
fn code_node_description_hides_when_error_strategy_is_none() {
    let document = WorkflowDocument::default();

    assert!(node_description_text(&document, &code_node(None)).is_empty());
    assert!(node_description_text(&document, &code_node(Some("none"))).is_empty());
}

#[test]
fn code_node_description_shows_error_strategy_summary() {
    let document = WorkflowDocument::default();

    assert_eq!(
        node_description_text(&document, &code_node(Some("default-value"))),
        "异常时 输出默认值"
    );
    assert_eq!(
        node_description_text(&document, &code_node(Some("fail-branch"))),
        "异常时 异常分支"
    );
}
