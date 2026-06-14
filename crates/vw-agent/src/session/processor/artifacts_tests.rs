use crate::session::llm::ToolCall;
use crate::session::processor::Request;
use crate::session::session::{Message, Role, Session};
use crate::session::ui_store;
use crate::session::ui_types as models;
use serde_json::{Value, json};
use uuid::Uuid;

use super::super::llm_runner::LlmStep;
use super::{
    persist_final_ai_call_payload, persist_llm_raw_step_payload, persist_step_ai_call_payload,
};

fn unique_id(label: &str) -> String {
    format!("artifacts-test-{label}-{}", Uuid::new_v4())
}

fn request(session_id: &str, stream: u64, persist: bool) -> Request {
    Request {
        stream,
        session: session_id.to_string(),
        query: "Explain the fixture".to_string(),
        root: Some("/tmp/project".to_string()),
        model: Some("test-model".to_string()),
        options: json!({"mode": "test"}),
        approval: None,
        channel_name: None,
        non_cli_approval_context: None,
        assistant_message_id: None,
        history: Vec::new(),
        persist_app_session_artifacts: persist,
    }
}

fn usage(input: i64, output: i64, cached: i64, reasoning: i64) -> models::TokenUsage {
    models::TokenUsage {
        input_tokens: input,
        output_tokens: output,
        cached_tokens: cached,
        reasoning_tokens: reasoning,
    }
}

fn session_with_all_roles(session_id: &str) -> Session {
    Session {
        id: session_id.to_string(),
        messages: vec![
            Message { role: Role::System, content: "system".to_string() },
            Message { role: Role::User, content: "user".to_string() },
            Message { role: Role::Assistant, content: "assistant".to_string() },
            Message { role: Role::Tool, content: "tool".to_string() },
        ],
    }
}

fn llm_step() -> LlmStep {
    LlmStep {
        usage: usage(10, 20, 30, 40),
        finish_reason: Some("stop".to_string()),
        reasoning_content: "reasoning".to_string(),
        text: "step answer".to_string(),
        tool_calls: vec![ToolCall {
            id: "call-1".to_string(),
            name: "test_tool".to_string(),
            arguments: "{\"ok\":true}".to_string(),
        }],
        full_messages: vec![
            json!({"role": "system", "content": "system one"}),
            json!({"role": "user", "content": "hello"}),
            json!({"role": "system", "content": 42}),
        ],
    }
}

fn read_json(path: &std::path::Path) -> Value {
    let content = std::fs::read_to_string(path).expect("json file should read");
    serde_json::from_str(&content).expect("json file should parse")
}

#[test]
fn step_and_final_ai_call_payloads_persist_to_scoped_session() {
    let session_id = unique_id("ai-call");
    let scope = unique_id("scope");
    let req = request(&session_id, 77, true);
    let session = session_with_all_roles(&session_id);
    let step = llm_step();
    ui_store::delete_session_scoped(&session_id, Some(&scope));

    persist_step_ai_call_payload(&req, &session, &usage(1, 2, 3, 4), &step, 5, Some(&scope));

    let loaded =
        ui_store::load_session_scoped(&session_id, Some(&scope)).expect("AI call should persist");
    assert_eq!(loaded.calls.len(), 1);
    let call = &loaded.calls[0];
    assert_eq!(call["session_id"], session_id);
    assert_eq!(call["stream_id"], 77);
    assert_eq!(call["step_index"], 5);
    assert_eq!(call["model"], "test-model");
    assert_eq!(call["root"], "/tmp/project");
    assert_eq!(call["usage"]["input_tokens"], 11);
    assert_eq!(call["usage"]["output_tokens"], 22);
    assert_eq!(call["usage"]["cached_tokens"], 33);
    assert_eq!(call["usage"]["reasoning_tokens"], 44);
    assert_eq!(call["answer"], "step answer");
    let roles = call["messages"]
        .as_array()
        .expect("messages array")
        .iter()
        .map(|message| message["role"].as_str().expect("role should be a string"))
        .collect::<Vec<_>>();
    assert_eq!(roles, vec!["system", "user", "assistant", "tool"]);

    persist_final_ai_call_payload(
        &req,
        &session,
        &usage(3, 4, 5, 6),
        "final answer",
        Some(&scope),
        true,
    );

    let loaded = ui_store::load_session_scoped(&session_id, Some(&scope))
        .expect("final AI call should replace matching stream");
    assert_eq!(loaded.calls.len(), 1);
    assert_eq!(loaded.calls[0]["answer"], "final answer");
    assert_eq!(loaded.calls[0]["usage"]["input_tokens"], 3);

    ui_store::delete_session_scoped(&session_id, Some(&scope));
}

#[test]
fn ai_call_payloads_respect_persistence_guards() {
    let session_id = unique_id("skip");
    let scope = unique_id("scope");
    let no_persist = request(&session_id, 88, false);
    let session = session_with_all_roles(&session_id);
    let step = llm_step();
    ui_store::delete_session_scoped(&session_id, Some(&scope));

    persist_step_ai_call_payload(&no_persist, &session, &usage(0, 0, 0, 0), &step, 0, Some(&scope));
    assert!(ui_store::load_session_scoped(&session_id, Some(&scope)).is_none());

    let empty_session = request("", 89, true);
    persist_step_ai_call_payload(
        &empty_session,
        &session,
        &usage(0, 0, 0, 0),
        &step,
        0,
        Some(&scope),
    );
    assert!(ui_store::load_session_scoped("", Some(&scope)).is_none());

    let req = request(&session_id, 90, true);
    persist_final_ai_call_payload(
        &req,
        &session,
        &usage(1, 1, 1, 1),
        "ignored",
        Some(&scope),
        false,
    );
    assert!(ui_store::load_session_scoped(&session_id, Some(&scope)).is_none());
}

#[test]
fn raw_llm_step_payload_persists_system_messages_output_and_tool_calls() {
    let session_id = unique_id("raw");
    let scope = unique_id("scope");
    let req = request(&session_id, 91, true);
    let step = llm_step();
    let llm_messages = vec![json!({"role": "user", "content": "hello"})];

    persist_llm_raw_step_payload(&req, &session_id, 3, &llm_messages, &step, Some(&scope));

    let path = ui_store::session_step_llm_raw_file_path_scoped(&session_id, 3, Some(&scope))
        .expect("raw step path should resolve");
    assert!(path.is_file());
    let payload = read_json(&path);
    assert_eq!(payload["session_id"], session_id);
    assert_eq!(payload["step_index"], 3);
    assert_eq!(payload["model"], "test-model");
    assert_eq!(payload["system"], json!(["system one"]));
    assert_eq!(payload["messages"], json!([{"role": "user", "content": "hello"}]));
    assert_eq!(payload["output"]["text"], "step answer");
    assert_eq!(payload["output"]["reasoning_content"], "reasoning");
    assert_eq!(payload["output"]["finish_reason"], "stop");
    assert_eq!(payload["output"]["tool_calls"][0]["id"], "call-1");
    assert_eq!(payload["output"]["tool_calls"][0]["type"], "function");
    assert_eq!(payload["output"]["tool_calls"][0]["function"]["name"], "test_tool");
    assert_eq!(payload["output"]["tool_calls"][0]["function"]["arguments"], "{\"ok\":true}");
    assert_eq!(payload["output"]["usage"]["reasoning_tokens"], 40);

    let _ = std::fs::remove_file(path);
}

#[test]
fn raw_llm_step_payload_respects_persistence_guards() {
    let session_id = unique_id("raw-skip");
    let scope = unique_id("scope");
    let step = llm_step();
    let no_persist = request(&session_id, 92, false);

    persist_llm_raw_step_payload(&no_persist, &session_id, 4, &[], &step, Some(&scope));
    let skipped_path =
        ui_store::session_step_llm_raw_file_path_scoped(&session_id, 4, Some(&scope))
            .expect("skipped raw step path should resolve");
    assert!(!skipped_path.exists());

    let empty_session = request("", 93, true);
    persist_llm_raw_step_payload(&empty_session, "", 5, &[], &step, Some(&scope));
}
