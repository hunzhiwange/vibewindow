//! 网络搜索工具集成测试模块
//!
//! 本模块包含针对网络搜索工具（WebSearch Tool）的集成测试用例。
//! 主要测试内容包括：
//! - 网络搜索工具的基本功能
//! - 搜索结果的处理和解析
//! - 错误处理和边界情况
//!
//! 通过 `use super::super::*` 导入父模块的工具定义和测试辅助函数，
//! 以便在本模块中编写具体的测试用例。

use super::super::*;

use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use serde_json::json;
use std::sync::Arc;

fn new_tool() -> WebSearchTool {
    WebSearchTool::new(
        Arc::new(SecurityPolicy {
            autonomy: AutonomyLevel::Supervised,
            ..SecurityPolicy::default()
        }),
        "exa".to_string(),
        None,
        None,
        5,
        15,
        "test".to_string(),
    )
}

#[test]
fn parameters_schema_exposes_claude_compat_fields() {
    let tool = new_tool();
    let schema = tool.parameters_schema();

    assert!(schema["properties"]["query"].is_object());
    assert!(schema["properties"]["num"].is_object());
    assert!(schema["properties"]["numResults"].is_object());
    assert!(schema["properties"]["lr"].is_object());
}

#[test]
fn args_accept_claude_compat_fields() {
    let args: super::Args = serde_json::from_value(json!({
        "query": "rust",
        "num": 4,
        "lr": "lang_en"
    }))
    .unwrap();

    assert_eq!(args.query.as_deref(), Some("rust"));
    assert_eq!(args.num_results, Some(4));
    assert_eq!(args.lr.as_deref(), Some("lang_en"));
}
