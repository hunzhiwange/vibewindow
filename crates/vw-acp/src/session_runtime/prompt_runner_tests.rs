//! Prompt 执行器命令解析逻辑的单元测试。

use crate::session_runtime::prompt_runner::{parse_agent_command, split_command_line};

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
fn parse_agent_command_extracts_command_and_args() {
    let config = parse_agent_command(r#"node ./bin/agent.js --mode fast"#);

    assert_eq!(config.command, "node");
    assert_eq!(
        config.args,
        vec!["./bin/agent.js".to_string(), "--mode".to_string(), "fast".to_string(),]
    );
    assert!(config.env.is_empty());
}
