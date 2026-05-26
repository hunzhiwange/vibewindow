//! Question 工具测试模块
//!
//! 本模块包含针对 `QuestionTool` 及其相关辅助函数的单元测试。
//! 主要测试以下内容：
//! - 工具 schema 的结构完整性（必需字段验证）
//! - 问题头部生成函数的长度限制行为
//!
//! # 测试覆盖范围
//!
//! - `question_schema_has_required_fields`: 验证 JSON schema 符合规范
//! - `header_from_question_limits_length`: 验证字符串截断逻辑

use super::super::*;
use crate::app::agent::tools::question::header_from_question;
use crate::app::agent::tools::traits::Tool;
use serde_json::json;

/// 测试 QuestionTool 的 schema 是否包含所有必需字段
///
/// # 测试目标
///
/// 验证 `QuestionTool::schema()` 返回的 JSON schema 满足以下条件：
/// 1. 类型为 "object"（表示这是一个对象结构）
/// 2. 包含 "questions" 属性定义
/// 3. "questions" 被标记为必需字段
///
/// # 重要性
///
/// Schema 的正确性对于工具的自动验证和文档生成至关重要。
/// 如果缺少必需字段标记，可能导致客户端提交无效的问题数据。
#[test]
fn question_schema_has_required_fields() {
    // 获取 QuestionTool 的 JSON schema
    let schema = QuestionTool::schema();

    // 验证 schema 类型为对象
    assert_eq!(schema["type"], "object");

    // 验证 questions 属性存在且为对象类型
    assert!(schema["properties"]["questions"].is_object());
    assert_eq!(schema["properties"]["questions"]["minItems"], 1);
    assert_eq!(schema["properties"]["questions"]["maxItems"], 4);
    assert!(schema["properties"]["answers"].is_object());
    assert!(schema["properties"]["annotations"].is_object());
    assert!(schema["properties"]["metadata"].is_object());

    // 验证 questions 字段被标记为必需
    assert_eq!(schema["required"], json!(["questions"]));
}

/// 测试 header_from_question 函数的长度限制功能
///
/// # 测试目标
///
/// 验证 `header_from_question` 函数能够正确截断过长的字符串，
/// 确保生成的头部字符串不超过预期的最大长度。
///
/// # 测试数据
///
/// 输入：36 个字符的字母数字字符串 "abcdefghijklmnopqrstuvwxyz1234567890"
/// 预期输出：前 30 个字符 "abcdefghijklmnopqrstuvwxyz1234"
///
/// # 重要性
///
/// 长度限制可以防止：
/// - 日志显示时的格式破坏
/// - UI 布局溢出
/// - 存储时的字段超限
#[test]
fn header_from_question_limits_length() {
    // 测试用例：36 字符的输入字符串
    let header = header_from_question("abcdefghijklmnopqrstuvwxyz1234567890");

    // 验证输出被截断为 12 个字符
    assert_eq!(header, "abcdefghijkl");
}

#[test]
fn question_tool_spec_uses_claude_surface() {
    let tool = QuestionTool::new("session-123".to_string());
    let spec = tool.spec();

    assert_eq!(spec.id, ASK_USER_QUESTION_TOOL_ID);
    assert!(spec.aliases.iter().any(|alias| alias == QUESTION_TOOL_ALIAS));
    assert!(spec.requires_user_interaction);
    assert!(spec.concurrency_safe);
}

#[test]
fn question_validate_input_normalizes_claude_fields() {
    let tool = QuestionTool::new("session-123".to_string());

    let normalized = tool
        .validate_input(json!({
            "questions": [
                {
                    "question": "Pick a library?",
                    "options": [
                        {
                            "label": "Zod",
                            "description": "Stay close to Claude Code",
                            "preview": "<pre>import { z } from 'zod'</pre>"
                        },
                        {
                            "label": "Valibot",
                            "description": "Smaller runtime"
                        }
                    ],
                    "multiSelect": false
                }
            ]
        }))
        .expect("input should normalize");

    assert_eq!(normalized["questions"][0]["header"], "Pick a libra");
    assert_eq!(normalized["questions"][0]["multiSelect"], false);
    assert_eq!(normalized["questions"][0]["custom"], true);
    assert_eq!(normalized["questions"][0]["options"][0]["preview"], "<pre>import { z } from 'zod'</pre>");
}

#[test]
fn question_validate_input_rejects_duplicate_question_text() {
    let tool = QuestionTool::new("session-123".to_string());

    let err = tool
        .validate_input(json!({
            "questions": [
                {
                    "question": "Same question?",
                    "options": [
                        { "label": "A", "description": "first" },
                        { "label": "B", "description": "second" }
                    ]
                },
                {
                    "question": "Same question?",
                    "options": [
                        { "label": "C", "description": "third" },
                        { "label": "D", "description": "fourth" }
                    ]
                }
            ]
        }))
        .expect_err("duplicate questions should fail");

    assert!(err.to_string().contains("Question texts must be unique"));
}

#[tokio::test]
async fn question_call_with_prefilled_answers_returns_claude_shape() {
    let tool = QuestionTool::new("session-123".to_string());
    let input = tool
        .validate_input(json!({
            "questions": [
                {
                    "question": "Which library should we use?",
                    "header": "Library",
                    "options": [
                        {
                            "label": "Zod",
                            "description": "Stay close to Claude Code",
                            "preview": "<pre>import { z } from 'zod'</pre>"
                        },
                        {
                            "label": "Valibot",
                            "description": "Smaller runtime"
                        }
                    ]
                }
            ],
            "answers": {
                "Which library should we use?": "Zod"
            },
            "annotations": {
                "Which library should we use?": {
                    "preview": "<pre>import { z } from 'zod'</pre>",
                    "notes": "Keep parity with Claude"
                }
            }
        }))
        .expect("input should normalize");

    let result = tool.call(input).await.expect("call should succeed");

    assert_eq!(result.data["questions"][0]["header"], "Library");
    assert_eq!(result.data["questions"][0]["multiSelect"], false);
    assert_eq!(result.data["answers"]["Which library should we use?"], "Zod");
    assert_eq!(
        result.data["annotations"]["Which library should we use?"]["notes"],
        "Keep parity with Claude"
    );
    assert!(result.model_result.as_str().is_some_and(|text| text.contains("User has answered your questions")));
}
