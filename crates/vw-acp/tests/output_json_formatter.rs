//! 验证 JSON 输出 formatter 的协议保持与敏感内容脱敏。
//!
//! JSON 输出通常被机器读取，因此测试既检查 JSON-RPC 结构稳定，也检查 read 类
//! 响应和工具更新不会把文件内容直接透出到日志流。

use serde_json::{Value, json};
use vw_acp::{
    AcpJsonRpcMessage, OutputErrorCode, OutputErrorOrigin, OutputErrorParams, OutputFormatter,
    OutputFormatterContext, create_json_output_formatter,
};

/// 将 JSON 值转换成 ACP JSON-RPC 消息，避免每个用例重复反序列化样板代码。
fn acp_message(value: Value) -> AcpJsonRpcMessage {
    serde_json::from_value(value).expect("message should deserialize into ACP JSON-RPC type")
}

/// 验证开启脱敏时，文件读取响应中的 content 被替换但其它字段保持原样。
#[test]
fn output_json_formatter_sanitizes_read_responses() {
    let mut formatter = create_json_output_formatter(
        Vec::new(),
        true,
        Some(OutputFormatterContext { session_id: "session-123".to_string() }),
    );

    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "fs/read_text_file",
        "params": {
            "path": "/tmp/demo.txt"
        }
    })));
    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "id": 7,
        "result": {
            "content": "secret",
            "path": "/tmp/demo.txt"
        }
    })));

    let output =
        String::from_utf8(formatter.into_inner()).expect("formatter output should be utf8");
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);

    let response: Value =
        serde_json::from_str(lines[1]).expect("response line should be valid json");
    assert_eq!(
        response.get("result").and_then(|value| value.get("content")).and_then(Value::as_str),
        Some("[read output suppressed]")
    );
    assert_eq!(
        response.get("result").and_then(|value| value.get("path")).and_then(Value::as_str),
        Some("/tmp/demo.txt")
    );
}

/// 验证 read 类工具的初始更新和后续更新都会同时脱敏结构化 content 与 rawOutput。
#[test]
fn output_json_formatter_sanitizes_read_like_tool_updates() {
    let mut formatter = create_json_output_formatter(Vec::new(), true, None);

    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "tool-1",
                "title": "Read File",
                "content": [
                    {
                        "type": "content",
                        "content": {
                            "type": "text",
                            "text": "secret"
                        }
                    }
                ],
                "rawOutput": {
                    "content": "secret"
                }
            }
        }
    })));
    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "tool-1",
                "content": [
                    {
                        "type": "content",
                        "content": {
                            "type": "text",
                            "text": "still secret"
                        }
                    }
                ],
                "rawOutput": {
                    "content": "still secret"
                }
            }
        }
    })));

    let output =
        String::from_utf8(formatter.into_inner()).expect("formatter output should be utf8");
    let lines = output.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);

    let first: Value = serde_json::from_str(lines[0]).expect("first line should be valid json");
    let second: Value = serde_json::from_str(lines[1]).expect("second line should be valid json");
    // 同一工具调用可能分多次更新，脱敏必须覆盖每一帧，不能只处理首条消息。
    for entry in [first, second] {
        assert_eq!(
            entry
                .get("params")
                .and_then(|value| value.get("update"))
                .and_then(|value| value.get("rawOutput"))
                .and_then(|value| value.get("content"))
                .and_then(Value::as_str),
            Some("[read output suppressed]")
        );
        assert_eq!(
            entry
                .get("params")
                .and_then(|value| value.get("update"))
                .and_then(|value| value.get("content"))
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(|value| value.get("content"))
                .and_then(|value| value.get("text"))
                .and_then(Value::as_str),
            Some("[read output suppressed]")
        );
    }
}

/// 验证 formatter 能把内部输出错误转换为带 session 上下文的 JSON-RPC error 行。
#[test]
fn output_json_formatter_emits_jsonrpc_error_response() {
    let mut formatter = create_json_output_formatter(
        Vec::new(),
        false,
        Some(OutputFormatterContext { session_id: "session-error".to_string() }),
    );

    formatter.on_error(OutputErrorParams {
        code: OutputErrorCode::Runtime,
        detail_code: Some("DETAIL".to_string()),
        origin: Some(OutputErrorOrigin::Queue),
        message: "boom".to_string(),
        retryable: Some(true),
        acp: None,
        timestamp: Some("2026-04-04T00:00:00Z".to_string()),
    });

    let output =
        String::from_utf8(formatter.into_inner()).expect("formatter output should be utf8");
    let line = output.lines().next().expect("one error line should be written");
    let payload: Value = serde_json::from_str(line).expect("error line should be valid json");

    assert_eq!(payload.get("jsonrpc").and_then(Value::as_str), Some("2.0"));
    assert_eq!(payload.get("id"), Some(&Value::Null));
    assert_eq!(
        payload
            .get("error")
            .and_then(|value| value.get("data"))
            .and_then(|value| value.get("sessionId"))
            .and_then(Value::as_str),
        Some("session-error")
    );
}
