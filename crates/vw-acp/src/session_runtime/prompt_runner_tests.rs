//! Prompt 执行器纯函数逻辑的单元测试。

use std::collections::HashMap;

use crate::session_runtime::prompt_runner::{
    clone_non_empty_tools, non_empty_trimmed, parse_agent_command, session_options_from_record,
    split_command_line,
};
use crate::types::{
    SessionAcpxState, SessionEventLog, SessionRecord, SessionStateOptions, SessionTokenUsage,
};

fn test_record() -> SessionRecord {
    SessionRecord {
        schema: crate::SESSION_RECORD_SCHEMA.to_string(),
        vwacp_record_id: "record-1".to_string(),
        acp_session_id: "acp-1".to_string(),
        agent_session_id: None,
        agent_command: "agent".to_string(),
        agent_config: None,
        cwd: "/tmp/project".to_string(),
        name: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        last_used_at: "2026-01-01T00:00:00Z".to_string(),
        last_seq: 0,
        last_request_id: None,
        event_log: SessionEventLog {
            active_path: "/tmp/active.ndjson".to_string(),
            segment_count: 1,
            max_segment_bytes: 1024,
            max_segments: 3,
            last_write_at: None,
            last_write_error: None,
        },
        closed: Some(false),
        closed_at: None,
        pid: None,
        agent_started_at: None,
        last_prompt_at: None,
        last_agent_exit_code: None,
        last_agent_exit_signal: None,
        last_agent_exit_at: None,
        last_agent_disconnect_reason: None,
        protocol_version: None,
        agent_capabilities: None,
        title: None,
        messages: Vec::new(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
        cumulative_token_usage: SessionTokenUsage::default(),
        request_token_usage: HashMap::new(),
        vwacp: None,
    }
}

fn test_vwacp_state(session_options: Option<SessionStateOptions>) -> SessionAcpxState {
    SessionAcpxState {
        current_mode_id: None,
        desired_mode_id: None,
        current_model_id: None,
        available_models: None,
        available_commands: None,
        config_options: None,
        session_options,
    }
}

#[test]
fn split_command_line_preserves_quoted_segments() {
    let parts = split_command_line(r#"npx "@scope/agent cli" --flag "two words""#);

    assert_eq!(
        parts,
        vec![
            "npx".to_string(),
            "@scope/agent cli".to_string(),
            "--flag".to_string(),
            "two words".to_string(),
        ]
    );
}

#[test]
fn split_command_line_handles_single_quotes_and_escaped_spaces() {
    let parts = split_command_line(r#"tool 'literal "value"' escaped\ space plain"#);

    assert_eq!(
        parts,
        vec![
            "tool".to_string(),
            "literal \"value\"".to_string(),
            "escaped space".to_string(),
            "plain".to_string(),
        ]
    );
}

#[test]
fn split_command_line_keeps_escaped_double_quote_content() {
    let parts = split_command_line(r#"agent "say \"hello\"" trailing\\"#);

    assert_eq!(
        parts,
        vec!["agent".to_string(), "say \"hello\"".to_string(), "trailing\\".to_string()]
    );
}

#[test]
fn parse_agent_command_extracts_command_and_args() {
    let config = parse_agent_command(r#"node ./bin/agent.js --mode fast"#);

    assert_eq!(config.command, "node");
    assert_eq!(
        config.args,
        vec!["./bin/agent.js".to_string(), "--mode".to_string(), "fast".to_string(),]
    );
    assert!(config.env.is_empty());
}

#[test]
fn parse_agent_command_handles_empty_input_without_args() {
    let config = parse_agent_command("   ");

    assert_eq!(config.command, "");
    assert!(config.args.is_empty());
    assert!(config.env.is_empty());
}

#[test]
fn non_empty_trimmed_filters_missing_and_blank_values() {
    assert_eq!(non_empty_trimmed(None), None);
    assert_eq!(non_empty_trimmed(Some(" \t\n ")), None);
    assert_eq!(non_empty_trimmed(Some("  model-a  ")), Some("model-a".to_string()));
}

#[test]
fn clone_non_empty_tools_trims_and_drops_blank_entries() {
    let stored = SessionStateOptions {
        model: None,
        allowed_tools: Some(vec![
            " shell ".to_string(),
            "".to_string(),
            "  ".to_string(),
            "read".to_string(),
        ]),
        max_turns: None,
    };

    assert_eq!(clone_non_empty_tools(&stored), Some(vec!["shell".to_string(), "read".to_string()]));
}

#[test]
fn clone_non_empty_tools_omits_missing_or_blank_tool_lists() {
    let missing = SessionStateOptions { model: None, allowed_tools: None, max_turns: None };
    let blank = SessionStateOptions {
        model: None,
        allowed_tools: Some(vec![" ".to_string(), "\t".to_string()]),
        max_turns: None,
    };

    assert_eq!(clone_non_empty_tools(&missing), None);
    assert_eq!(clone_non_empty_tools(&blank), None);
}

#[test]
fn session_options_from_record_omits_missing_vwacp_state() {
    assert_eq!(session_options_from_record(&test_record()), None);
}

#[test]
fn session_options_from_record_filters_blank_options() {
    let mut record = test_record();
    record.vwacp = Some(test_vwacp_state(Some(SessionStateOptions {
        model: Some("  ".to_string()),
        allowed_tools: Some(vec!["".to_string(), "  ".to_string()]),
        max_turns: None,
    })));

    assert_eq!(session_options_from_record(&record), None);
}

#[test]
fn session_options_from_record_preserves_valid_options() {
    let mut record = test_record();
    record.vwacp = Some(test_vwacp_state(Some(SessionStateOptions {
        model: Some("  gpt-5  ".to_string()),
        allowed_tools: Some(vec![" shell ".to_string(), " ".to_string(), "read".to_string()]),
        max_turns: Some(4),
    })));

    let options = session_options_from_record(&record).expect("session options");

    assert_eq!(options.model.as_deref(), Some("gpt-5"));
    assert_eq!(options.allowed_tools, Some(vec!["shell".to_string(), "read".to_string()]));
    assert_eq!(options.max_turns, Some(4));
}
