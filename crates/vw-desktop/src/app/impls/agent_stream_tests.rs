// Tests for plan6 task 846.
const SOURCE: &str = include_str!("agent_stream.rs");

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn agent_stream_tests_keeps_planned_coverage_targets() {
    for name in [
        "agent_stream",
        "gateway_agent_stream",
        "parse_usage",
        "temporary_workflow_yaml",
        "stream_workflow_chat_messages",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

#[test]
fn temporary_workflow_yaml_contains_required_chat_nodes() {
    let yaml = super::temporary_workflow_yaml(
        "请帮我实现 chat 面板里的工作流执行进度展示",
        Some("demo-model"),
    )
    .expect("workflow yaml");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("valid yaml");
    let nodes = value
        .get("workflow")
        .and_then(|workflow| workflow.get("graph"))
        .and_then(|graph| graph.get("nodes"))
        .and_then(serde_yaml::Value::as_sequence)
        .expect("nodes");

    let ids = nodes
        .iter()
        .filter_map(|node| node.get("id").and_then(serde_yaml::Value::as_str))
        .collect::<Vec<_>>();
    assert!(ids.contains(&"start"));
    assert!(ids.contains(&"task_1"));
    assert!(ids.contains(&"answer"));
    assert!(!ids.contains(&"parallel_context"));

    let edges = value
        .get("workflow")
        .and_then(|workflow| workflow.get("graph"))
        .and_then(|graph| graph.get("edges"))
        .and_then(serde_yaml::Value::as_sequence)
        .expect("edges");
    assert!(edges.iter().any(|edge| {
        edge.get("source").and_then(serde_yaml::Value::as_str) == Some("start")
            && edge.get("target").and_then(serde_yaml::Value::as_str) == Some("task_1")
    }));
    assert!(edges.iter().any(|edge| {
        edge.get("source").and_then(serde_yaml::Value::as_str) == Some("task_1")
            && edge.get("target").and_then(serde_yaml::Value::as_str) == Some("answer")
    }));

    let task_title = nodes
        .iter()
        .find(|node| node.get("id").and_then(serde_yaml::Value::as_str) == Some("task_1"))
        .and_then(|node| node.get("data"))
        .and_then(|data| data.get("title"))
        .and_then(serde_yaml::Value::as_str)
        .expect("task title");
    assert_eq!(task_title, "实现 chat 面板里的工作流执行进度展示");
}

#[test]
fn temporary_workflow_yaml_generates_different_graphs_by_requirement() {
    let coding =
        super::temporary_workflow_yaml("先梳理需求，然后实现 Rust API，最后验证接口", None)
            .expect("workflow yaml");
    let writing =
        super::temporary_workflow_yaml("写一封产品发布邮件", None).expect("workflow yaml");

    assert!(coding.contains("梳理需求"));
    assert!(coding.contains("实现 Rust API"));
    assert!(coding.contains("验证接口"));
    assert!(!coding.contains("Requirement Analysis"));
    assert!(writing.contains("写一封产品发布邮件"));
    assert_ne!(coding, writing);
}

#[test]
fn temporary_workflow_yaml_sets_dify_node_layout() {
    let yaml = super::temporary_workflow_yaml("为什么 workflow 会超时并卡死", None)
        .expect("workflow yaml");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("valid yaml");
    let nodes = value
        .get("workflow")
        .and_then(|workflow| workflow.get("graph"))
        .and_then(|graph| graph.get("nodes"))
        .and_then(serde_yaml::Value::as_sequence)
        .expect("nodes");

    let mut positions = Vec::new();
    for node in nodes {
        let position =
            node.get("position").and_then(serde_yaml::Value::as_mapping).expect("node position");
        let x = position
            .get(&serde_yaml::Value::String("x".to_string()))
            .and_then(serde_yaml::Value::as_i64)
            .expect("x");
        let y = position
            .get(&serde_yaml::Value::String("y".to_string()))
            .and_then(serde_yaml::Value::as_i64)
            .expect("y");
        assert_eq!(node.get("type").and_then(serde_yaml::Value::as_str), Some("custom"));
        assert!(node.get("positionAbsolute").is_some());
        assert!(node.get("sourcePosition").is_some());
        assert!(node.get("targetPosition").is_some());
        positions.push((x, y));
    }

    positions.sort_unstable();
    positions.dedup();
    assert_eq!(positions.len(), nodes.len());
}

#[test]
fn parse_workflow_chat_messages_event_maps_dify_payloads() {
    let delta = super::parse_workflow_chat_messages_event(serde_json::json!({
        "event": "message",
        "answer": "hello"
    }));
    assert!(matches!(delta, super::WorkflowChatMessagesEvent::Delta(text) if text == "hello"));

    let done = super::parse_workflow_chat_messages_event(serde_json::json!({
        "event": "message_end",
        "message_id": "msg-workflow",
        "metadata": {
            "usage": {
                "prompt_tokens": 1,
                "completion_tokens": 2,
                "total_tokens": 3
            }
        }
    }));
    assert!(matches!(
        done,
        super::WorkflowChatMessagesEvent::Done {
            message_id: Some(message_id),
            ..
        } if message_id == "msg-workflow"
    ));

    let node_started = super::parse_workflow_chat_messages_event(serde_json::json!({
        "event": "node_started",
        "data": {
            "node_id": "llm",
            "node_type": "llm",
            "title": "Run Task",
            "index": 2
        }
    }));
    assert!(matches!(
        node_started,
        super::WorkflowChatMessagesEvent::NodeStarted(event)
            if event.node_id == "llm" && event.title == "Run Task" && event.index == 2
    ));

    let node_finished = super::parse_workflow_chat_messages_event(serde_json::json!({
        "event": "node_finished",
        "data": {
            "node_id": "llm",
            "node_type": "llm",
            "title": "Run Task",
            "index": 2,
            "status": "succeeded",
            "outputs": {"answer": "ok"}
        }
    }));
    assert!(matches!(
        node_finished,
        super::WorkflowChatMessagesEvent::NodeFinished(event)
            if event.node_id == "llm" && event.status.as_deref() == Some("succeeded")
    ));

    let text_chunk = super::parse_workflow_chat_messages_event(serde_json::json!({
        "event": "text_chunk",
        "data": {
            "text": "{\"rows\":1}",
            "from_variable_selector": ["answer", "text"]
        }
    }));
    assert!(
        matches!(text_chunk, super::WorkflowChatMessagesEvent::Delta(text) if text == "{\"rows\":1}")
    );
}

#[test]
fn workflow_node_tool_block_marks_running_and_finished_states() {
    let event = super::WorkflowNodeStreamEvent {
        node_id: "llm".to_string(),
        node_type: "llm".to_string(),
        title: "Run Task".to_string(),
        index: 2,
        status: Some("succeeded".to_string()),
        elapsed_time: Some(1.25),
        error: None,
        outputs: Some(serde_json::json!({"answer": "ok"})),
    };

    let workflow_yaml = "kind: app\nworkflow:\n  graph:\n    nodes: []\n    edges: []\n";
    let running = super::workflow_node_tool_block(&event, true, workflow_yaml);
    assert!(running.contains("\"status\":\"running\""));
    assert!(running.contains("\"canonical_tool_id\":\"workflow_node\""));
    assert!(running.contains("workflow_yaml"));
    assert!(running.contains("\\\"answer\\\":\\\"ok\\\""));

    let finished = super::workflow_node_tool_block(&event, false, workflow_yaml);
    assert!(finished.contains("\"status\":\"completed\""));
    assert!(finished.contains("\\\"answer\\\":\\\"ok\\\""));
}

#[test]
fn workflow_node_tool_block_exposes_usage_metadata() {
    let event = super::WorkflowNodeStreamEvent {
        node_id: "llm".to_string(),
        node_type: "llm".to_string(),
        title: "Run Task".to_string(),
        index: 2,
        status: Some("succeeded".to_string()),
        elapsed_time: Some(1.25),
        error: None,
        outputs: Some(serde_json::json!({
            "answer": "ok",
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })),
    };

    let block = super::workflow_node_tool_block(&event, false, "kind: app\n");
    assert!(block.contains("\"usage\""));
    assert!(block.contains("\"total_tokens\":15"));
}

#[test]
fn workflow_history_context_filters_empty_messages_and_keeps_order() {
    let history = vec![
        crate::app::models::ChatMessage {
            role: crate::app::models::ChatRole::System,
            content: " system rules ".to_string(),
            think_timing: Vec::new(),
        },
        crate::app::models::ChatMessage {
            role: crate::app::models::ChatRole::User,
            content: "   ".to_string(),
            think_timing: Vec::new(),
        },
        crate::app::models::ChatMessage {
            role: crate::app::models::ChatRole::Assistant,
            content: " answer ".to_string(),
            think_timing: Vec::new(),
        },
        crate::app::models::ChatMessage {
            role: crate::app::models::ChatRole::Tool,
            content: " tool output ".to_string(),
            think_timing: Vec::new(),
        },
    ];

    assert_eq!(
        super::workflow_history_context(&history),
        "System: system rules\n\nAssistant: answer\n\nTool: tool output"
    );
}

#[test]
fn workflow_history_context_truncates_large_context_from_tail() {
    let history = vec![crate::app::models::ChatMessage {
        role: crate::app::models::ChatRole::User,
        content: "x".repeat(super::WORKFLOW_HISTORY_MAX_CHARS + 64),
        think_timing: Vec::new(),
    }];

    let context = super::workflow_history_context(&history);

    assert!(context.starts_with("...[truncated]\n"));
    assert!(context.len() <= super::WORKFLOW_HISTORY_MAX_CHARS + "...[truncated]\n".len());
    assert!(context.ends_with("xxxxxxxx"));
}

#[test]
fn temporary_workflow_task_titles_follow_user_requirement() {
    assert_eq!(
        super::temporary_workflow_task_titles("先读取 CSV，然后统计金额，最后生成报表"),
        vec!["读取 CSV", "统计金额", "生成报表"]
    );
    assert_eq!(
        super::temporary_workflow_task_titles("1. 设计接口\n2. 实现 API\n3. 补充测试"),
        vec!["设计接口", "实现 API", "补充测试"]
    );
}

#[test]
fn temporary_workflow_task_titles_strip_prefixes_dedup_and_limit_long_lists() {
    let titles = super::temporary_workflow_task_titles(
        "一、请帮我分析需求\n二、请帮我分析需求\n3）实现功能\n4: 补测试\n5. 写文档\n6. 发版\n7. 监控\n8. 复盘\n9. 归档\n10. 清理",
    );

    assert_eq!(titles.len(), super::TEMPORARY_WORKFLOW_MAX_TASKS);
    assert_eq!(titles[0], "分析需求");
    assert_eq!(titles[1], "实现功能");
    assert_eq!(titles[7], "完成剩余任务：归档；清理");
}

#[test]
fn temporary_workflow_task_titles_falls_back_for_empty_requirement() {
    assert_eq!(super::temporary_workflow_task_titles("   "), vec!["完成用户需求"]);
}

#[test]
fn temporary_workflow_topology_uses_requirement_language() {
    assert_eq!(
        super::temporary_workflow_topology("同时生成标题", 1),
        super::TemporaryWorkflowTopology::Serial
    );
    assert_eq!(
        super::temporary_workflow_topology("同时生成标题并且生成摘要", 2),
        super::TemporaryWorkflowTopology::Parallel
    );
    assert_eq!(
        super::temporary_workflow_topology("先生成标题，然后写摘要", 2),
        super::TemporaryWorkflowTopology::Serial
    );
}

#[test]
fn temporary_workflow_yaml_trims_empty_model_and_sets_non_empty_model() {
    let without_model = super::temporary_workflow_yaml("普通任务", Some("   ")).expect("yaml");
    assert!(!without_model.contains("provider: vibewindow"));

    let with_model = super::temporary_workflow_yaml("普通任务", Some(" gpt-local ")).expect("yaml");
    assert!(with_model.contains("name: gpt-local"));
    assert!(with_model.contains("provider: vibewindow"));
}

#[test]
fn temporary_workflow_plan_serial_edges_chain_to_final_node() {
    let plan = super::temporary_workflow_plan("先读取文件，然后总结内容");

    assert_eq!(plan.topology, super::TemporaryWorkflowTopology::Serial);
    assert_eq!(plan.final_node_id, "task_2");
    assert_eq!(plan.edges[0].source, "start");
    assert_eq!(plan.edges[0].target, "task_1");
    assert_eq!(plan.edges[1].source, "task_1");
    assert_eq!(plan.edges[1].target, "task_2");
    assert!(plan.steps[1].system_prompt.contains("final task node"));
}

#[test]
fn temporary_workflow_yaml_builds_dynamic_parallel_tasks_when_requested() {
    let yaml = super::temporary_workflow_yaml("同时生成标题并且生成摘要", None).expect("yaml");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("valid yaml");
    let nodes = value
        .get("workflow")
        .and_then(|workflow| workflow.get("graph"))
        .and_then(|graph| graph.get("nodes"))
        .and_then(serde_yaml::Value::as_sequence)
        .expect("nodes");
    let titles = nodes
        .iter()
        .filter_map(|node| node.get("data"))
        .filter_map(|data| data.get("title"))
        .filter_map(serde_yaml::Value::as_str)
        .collect::<Vec<_>>();

    assert!(titles.contains(&"生成标题"));
    assert!(titles.contains(&"生成摘要"));
    assert!(titles.contains(&"汇总并交付最终结果"));
    assert!(!yaml.contains("Parallel Context"));
}

#[test]
fn workflow_node_helpers_build_stable_keys_and_delta_preview_event() {
    let delta = super::WorkflowNodeDeltaStreamEvent {
        node_id: "node-a".to_string(),
        node_type: "llm".to_string(),
        title: "Draft".to_string(),
        index: 7,
        text: "partial".to_string(),
        replace: false,
    };
    let mut output = super::WorkflowNodeOutputBuffer::new();
    output.push("partial", false);

    let event = super::workflow_node_stream_event_from_delta(&delta, &output);

    assert_eq!(super::workflow_node_delta_event_key(&delta), "7-node-a");
    assert_eq!(super::workflow_node_stream_event_key(&event), "7-node-a");
    assert_eq!(super::workflow_node_tool_call_id(&event), "workflow-node-7-node-a");
    assert_eq!(event.status.as_deref(), Some("running"));
    assert_eq!(
        event
            .outputs
            .as_ref()
            .and_then(|outputs| outputs.get("answer"))
            .and_then(serde_json::Value::as_str),
        Some("partial")
    );
}

#[test]
fn workflow_node_parsers_accept_top_level_payloads_and_defaults() {
    let node = super::parse_workflow_node_event(&serde_json::json!({
        "node_id": "top",
        "node_type": "tool",
        "title": "Top Level",
        "index": 4,
        "elapsed_time": 0.5,
        "error": "failed",
        "outputs": null
    }));
    assert_eq!(node.node_id, "top");
    assert_eq!(node.node_type, "tool");
    assert_eq!(node.title, "Top Level");
    assert_eq!(node.index, 4);
    assert_eq!(node.elapsed_time, Some(0.5));
    assert_eq!(node.error.as_deref(), Some("failed"));
    assert_eq!(node.outputs, Some(serde_json::Value::Null));

    let delta = super::parse_workflow_node_delta_event(&serde_json::json!({
        "node_id": "delta",
        "answer": "fallback answer"
    }));
    assert_eq!(delta.node_id, "delta");
    assert_eq!(delta.text, "fallback answer");
    assert!(!delta.replace);
}

#[test]
fn workflow_node_output_buffer_flushes_by_force_replace_and_size() {
    let mut output = super::WorkflowNodeOutputBuffer::new();
    output.push("abc", false);
    assert!(!output.should_flush(false));
    assert!(output.should_flush(true));

    output.push(&"x".repeat(super::WORKFLOW_NODE_STREAM_FLUSH_CHARS), false);
    assert!(output.should_flush(false));

    output.push("replacement", true);
    assert_eq!(output.text, "replacement");
    assert!(output.should_flush(true));
}

#[test]
fn compact_workflow_text_preserves_short_text_and_truncates_long_text() {
    let (short, short_truncated) = super::compact_workflow_text("hello");
    assert_eq!(short, "hello");
    assert!(!short_truncated);

    let (long, long_truncated) =
        super::compact_workflow_text(&"字".repeat(super::WORKFLOW_NODE_PREVIEW_MAX_CHARS + 24));
    assert!(long_truncated);
    assert!(long.contains("已截断 24 字符"));

    let mut in_place = "y".repeat(super::WORKFLOW_NODE_PREVIEW_MAX_CHARS + 1);
    assert!(super::compact_workflow_text_in_place(&mut in_place));
    assert!(in_place.contains("已截断 1 字符"));
}

#[test]
fn workflow_output_preview_text_uses_preferred_keys_or_json_fallback() {
    assert_eq!(
        super::workflow_output_preview_text(&serde_json::json!({"answer": "  final answer  "})),
        "final answer"
    );
    assert_eq!(
        super::workflow_output_preview_text(&serde_json::json!({"text": " text answer "})),
        "text answer"
    );
    assert!(
        super::workflow_output_preview_text(&serde_json::json!({"items": [1, 2]}))
            .contains("\"items\"")
    );
}

#[test]
fn workflow_node_preview_outputs_skips_null_and_uses_completion_fallback() {
    let null_event = super::WorkflowNodeStreamEvent {
        node_id: "n".to_string(),
        node_type: "llm".to_string(),
        title: "Null".to_string(),
        index: 1,
        status: None,
        elapsed_time: None,
        error: None,
        outputs: Some(serde_json::Value::Null),
    };
    assert!(super::workflow_node_preview_outputs(&null_event).is_none());

    let block = super::workflow_node_tool_block(&null_event, false, "");
    assert!(block.contains("节点执行完成"));
}

#[test]
fn workflow_preview_outputs_marks_truncation_and_preserves_usage() {
    let outputs = super::workflow_preview_outputs(
        &"x".repeat(super::WORKFLOW_NODE_PREVIEW_MAX_CHARS + 8),
        Some(serde_json::json!({"total_tokens": 9})),
        false,
    );

    assert_eq!(outputs.get("truncated").and_then(serde_json::Value::as_bool), Some(true));
    assert_eq!(
        outputs
            .get("usage")
            .and_then(|usage| usage.get("total_tokens"))
            .and_then(serde_json::Value::as_i64),
        Some(9)
    );
}

#[test]
fn workflow_node_tool_block_marks_errors_and_omits_large_yaml() {
    let event = super::WorkflowNodeStreamEvent {
        node_id: "node-b".to_string(),
        node_type: String::new(),
        title: "Failed Node".to_string(),
        index: 3,
        status: Some("failed".to_string()),
        elapsed_time: None,
        error: Some("boom".to_string()),
        outputs: None,
    };

    let block = super::workflow_node_tool_block(
        &event,
        false,
        &"x".repeat(super::WORKFLOW_NODE_INLINE_YAML_MAX_CHARS + 1),
    );

    assert!(block.contains("\"status\":\"error\""));
    assert!(block.contains("\"input\":\"Failed Node\""));
    assert!(block.contains("\"workflow_yaml\":null"));
}

#[test]
fn take_next_workflow_stream_payload_handles_lf_crlf_and_multiline_data() {
    let mut buffer = "event: message\ndata: {\"a\":1}\ndata: {\"b\":2}\n\nrest".to_string();
    assert_eq!(
        super::take_next_workflow_stream_payload(&mut buffer),
        Some("{\"a\":1}\n{\"b\":2}".to_string())
    );
    assert_eq!(buffer, "rest");

    let mut crlf = "data: done\r\n\r\n".to_string();
    assert_eq!(super::take_next_workflow_stream_payload(&mut crlf), Some("done".to_string()));
    assert!(crlf.is_empty());

    let mut no_payload = ": keepalive\n\n".to_string();
    assert!(super::take_next_workflow_stream_payload(&mut no_payload).is_none());
    assert!(no_payload.is_empty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_workflow_auth_sends_skey_as_bearer() {
    let client = reqwest::Client::new();
    let endpoint = vw_gateway_client::GatewayEndpoint::new("127.0.0.1", 1).with_auth(
        vw_gateway_client::GatewayAuth {
            skey: Some(" key ".to_string()),
        },
    );

    let request = super::apply_workflow_auth(client.post("http://127.0.0.1/"), &endpoint)
        .build()
        .expect("request should build");

    assert_eq!(
        request.headers().get(reqwest::header::AUTHORIZATION).and_then(|v| v.to_str().ok()),
        Some("Bearer  key ")
    );
    assert!(!request.headers().contains_key("x-skey"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn apply_workflow_auth_leaves_request_untouched_without_auth() {
    let client = reqwest::Client::new();
    let endpoint = vw_gateway_client::GatewayEndpoint::new("127.0.0.1", 1);

    let request = super::apply_workflow_auth(client.post("http://127.0.0.1/"), &endpoint)
        .build()
        .expect("request should build");

    assert!(!request.headers().contains_key(reqwest::header::AUTHORIZATION));
    assert!(!request.headers().contains_key("x-skey"));
}

#[test]
fn parse_workflow_chat_messages_event_covers_error_other_and_delta_variants() {
    assert!(matches!(
        super::parse_workflow_chat_messages_event(serde_json::json!({
            "event": "workflow_finished",
            "data": {"status": "failed", "error": "bad workflow"}
        })),
        super::WorkflowChatMessagesEvent::Error(error) if error == "bad workflow"
    ));
    assert!(matches!(
        super::parse_workflow_chat_messages_event(serde_json::json!({
            "event": "workflow_finished",
            "data": {"status": "succeeded"}
        })),
        super::WorkflowChatMessagesEvent::Other
    ));
    assert!(matches!(
        super::parse_workflow_chat_messages_event(serde_json::json!({
            "event": "error",
            "message": "bad request"
        })),
        super::WorkflowChatMessagesEvent::Error(error) if error == "bad request"
    ));
    assert!(matches!(
        super::parse_workflow_chat_messages_event(serde_json::json!({
            "event": "node_delta",
            "data": {"node_id": "n", "answer": "chunk", "replace": true}
        })),
        super::WorkflowChatMessagesEvent::NodeDelta(event)
            if event.node_id == "n" && event.text == "chunk" && event.replace
    ));
}

#[test]
fn parse_usage_supports_gateway_and_desktop_shapes() {
    let gateway = super::parse_usage(Some(&serde_json::json!({
        "prompt_tokens": 11,
        "completion_tokens": 12,
        "total_tokens": 23
    })));
    assert_eq!(gateway.input_tokens, 11);
    assert_eq!(gateway.output_tokens, 12);
    assert_eq!(gateway.cached_tokens, 0);

    let desktop = super::parse_usage(Some(&serde_json::json!({
        "input_tokens": 3,
        "output_tokens": 4,
        "cached_tokens": 5,
        "reasoning_tokens": 6
    })));
    assert_eq!(desktop.input_tokens, 3);
    assert_eq!(desktop.output_tokens, 4);
    assert_eq!(desktop.cached_tokens, 5);
    assert_eq!(desktop.reasoning_tokens, 6);

    assert_eq!(super::parse_usage(None), crate::app::models::TokenUsage::default());
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_workflow_stream_response(
    status: &'static str,
    body: String,
) -> (vw_gateway_client::GatewayEndpoint, std::thread::JoinHandle<String>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let addr = listener.local_addr().expect("test server address");
    let handle = std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return String::new();
        };
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        while let Ok(read) = stream.read(&mut chunk) {
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        String::from_utf8_lossy(&request).to_string()
    });

    (vw_gateway_client::GatewayEndpoint::new("127.0.0.1", addr.port()), handle)
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn stream_workflow_chat_messages_sends_auth_query_and_parses_events() {
    let stream_body = concat!(
        "data: {\"event\":\"message\",\"answer\":\"hello\"}\n\n",
        "data: {\"event\":\"node_delta\",\"data\":{\"node_id\":\"n1\",\"node_type\":\"llm\",\"title\":\"Run\",\"index\":1,\"text\":\"partial\"}}\n\n",
        "data: {\"event\":\"message_end\",\"message_id\":\"done\",\"metadata\":{\"usage\":{\"prompt_tokens\":1,\"completion_tokens\":2}}}\n\n"
    )
    .to_string();
    let (endpoint, handle) = spawn_workflow_stream_response("200 OK", stream_body);
    let endpoint = endpoint.with_auth(vw_gateway_client::GatewayAuth {
        skey: Some("skey-value".to_string()),
    });
    let mut events = Vec::new();

    let result = super::stream_workflow_chat_messages(
        &endpoint,
        Some("/tmp/project"),
        &serde_json::json!({"query": "hello"}),
        |event| {
            events.push(event);
            true
        },
    )
    .await;
    let request = handle.join().expect("server thread should finish");

    assert!(result.is_ok());
    assert!(
        request.starts_with(
            "POST /v1/workflow/applications/chat-messages?directory=%2Ftmp%2Fproject "
        )
    );
    assert!(request.contains("authorization: Bearer skey-value"));
    assert!(!request.contains("x-skey:"));
    assert!(
        matches!(&events[0], super::WorkflowChatMessagesEvent::Delta(delta) if delta == "hello")
    );
    assert!(
        matches!(&events[1], super::WorkflowChatMessagesEvent::NodeDelta(event) if event.node_id == "n1")
    );
    assert!(matches!(
        &events[2],
        super::WorkflowChatMessagesEvent::Done {
            message_id: Some(message_id),
            ..
        } if message_id == "done"
    ));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn stream_workflow_chat_messages_returns_status_errors() {
    let (endpoint, handle) =
        spawn_workflow_stream_response("500 Internal Server Error", "broken".to_string());

    let result = super::stream_workflow_chat_messages(
        &endpoint,
        None,
        &serde_json::json!({"query": "hello"}),
        |_| true,
    )
    .await;
    let _ = handle.join().expect("server thread should finish");

    let error = result.expect_err("status error should be returned");
    assert!(error.contains("workflow chat messages failed: 500 Internal Server Error broken"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn agent_stream_wrapper_returns_stream_without_polling() {
    let req = super::AgentRequest {
        id: 42,
        session: "session".to_string(),
        query: "hello".to_string(),
        root: None,
        model: None,
        acp_test: false,
        acp_agent: None,
        acp_allowed_tools: None,
        agent: None,
        allowed_tools: None,
        acp_force_new_session: false,
        acp_history_mode: crate::app::state::AcpHistoryReplayMode::Discard,
        acp_recent_count: 0,
        full_access_enabled: false,
        resume_history_only: false,
        workflow_mode_enabled: false,
        history: Vec::new(),
    };

    let stream = super::agent_stream(&req);
    drop(stream);
}
