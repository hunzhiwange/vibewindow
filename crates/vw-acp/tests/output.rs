//! 验证不同输出格式 formatter 对 ACP 消息的渲染。
//!
//! 测试覆盖普通文本、quiet 模式、工具调用摘要和读取内容脱敏，确保 CLI 输出既
//! 易读又不会在默认路径中泄露被读取文件的完整内容。

use serde_json::{Value, json};
use vw_acp::{
    AcpJsonRpcMessage, OutputFormat, OutputFormatter, OutputFormatterOptions,
    create_output_formatter,
};

/// 将 JSON 值转换为 ACP JSON-RPC 消息，简化各测试中构造协议事件的样板代码。
fn acp_message(value: Value) -> AcpJsonRpcMessage {
    serde_json::from_value(value).expect("message should deserialize into ACP JSON-RPC type")
}

/// 验证文本 formatter 会渲染 agent 文本、工具摘要和完成原因，并隐藏读取输出。
#[test]
fn text_output_formatter_renders_prompt_and_tool_updates() {
    let mut formatter = create_output_formatter(
        OutputFormat::Text,
        Vec::new(),
        OutputFormatterOptions {
            suppress_reads: true,
            is_tty: false,
            ..OutputFormatterOptions::default()
        },
    );

    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "content": {
                    "type": "text",
                    "text": "hello"
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
                "sessionUpdate": "tool_call",
                "toolCallId": "tool-1",
                "title": "Read File",
                "status": "completed",
                "rawInput": {
                    "path": "/tmp/demo.txt"
                },
                "rawOutput": {
                    "content": "secret"
                }
            }
        }
    })));
    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "id": 7,
        "result": {
            "stopReason": "end_turn"
        }
    })));
    formatter.flush();

    let output =
        String::from_utf8(formatter.into_inner()).expect("formatter output should be utf8");
    assert!(output.contains("hello"));
    assert!(output.contains("[tool] Read File (completed)"));
    assert!(output.contains("input: path=/tmp/demo.txt"));
    assert!(output.contains("[read output suppressed]"));
    assert!(output.contains("[done] end_turn"));
}

/// 验证工具输出截断按 UTF-8 边界处理，避免多字节字符导致 panic 或乱码。
#[test]
fn text_output_formatter_truncates_multibyte_tool_output_without_panicking() {
    let mut formatter = create_output_formatter(
        OutputFormat::Text,
        Vec::new(),
        OutputFormatterOptions { is_tty: false, ..OutputFormatterOptions::default() },
    );

    // 1998 个 ASCII 字节加一个三字节字符，专门覆盖截断点落在多字节字符前的边界。
    let multibyte_output = format!("{}向", "a".repeat(1_998));

    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "tool-2",
                "title": "Read AGENTS.md",
                "status": "completed",
                "rawOutput": {
                    "content": multibyte_output
                }
            }
        }
    })));
    formatter.flush();

    let output =
        String::from_utf8(formatter.into_inner()).expect("formatter output should be utf8");
    assert!(output.contains("[tool] Read AGENTS.md (completed)"));
    assert!(output.contains(&format!("{}...", "a".repeat(1_998))));
    assert!(!output.contains("向"));
}

/// 验证 quiet formatter 只输出最终 agent 文本，适合脚本消费。
#[test]
fn quiet_output_formatter_only_emits_final_prompt_text() {
    let mut formatter =
        create_output_formatter(OutputFormat::Quiet, Vec::new(), OutputFormatterOptions::default());

    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "session-1",
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "content": {
                    "type": "text",
                    "text": "hello quiet mode"
                }
            }
        }
    })));
    formatter.on_acp_message(acp_message(json!({
        "jsonrpc": "2.0",
        "id": 8,
        "result": {
            "stopReason": "end_turn"
        }
    })));
    formatter.flush();

    let output =
        String::from_utf8(formatter.into_inner()).expect("formatter output should be utf8");
    assert_eq!(output, "hello quiet mode\n");
}
