use super::*;
use crate::app::agent::tools::ToolSpec;

fn tool_spec(id: &str) -> ToolSpec {
    ToolSpec {
        id: id.to_string(),
        display_name: id.to_string(),
        description: "desc".to_string(),
        input_schema: serde_json::json!({"type": "object"}),
        name: id.to_string(),
        parameters: serde_json::json!({"type": "object"}),
        aliases: Vec::new(),
        read_only: true,
        destructive: false,
        concurrency_safe: true,
        requires_user_interaction: false,
        strict: true,
    }
}

#[test]
fn tool_instructions_include_each_tool_spec() {
    let instructions = build_tool_instructions_from_specs(&[tool_spec("shell"), tool_spec("grep")]);

    assert!(instructions.contains("## Tool Use Protocol"));
    assert!(instructions.contains("**shell**"));
    assert!(instructions.contains("**grep**"));
}

#[test]
fn shell_policy_deduplicates_and_sorts_allowed_commands() {
    let autonomy = crate::app::agent::config::AutonomyConfig {
        allowed_commands: vec!["git".to_string(), "  ls  ".to_string(), "git".to_string()],
        ..Default::default()
    };

    let instructions = build_shell_policy_instructions(&autonomy);

    assert!(instructions.contains("`git`"));
    assert!(instructions.contains("`ls`"));
}
