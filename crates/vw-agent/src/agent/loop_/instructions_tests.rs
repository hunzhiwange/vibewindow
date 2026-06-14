use super::*;
use crate::app::agent::tools::ToolSpec;
use async_trait::async_trait;

struct Testcov0095Tool;

#[async_trait]
impl Tool for Testcov0095Tool {
    fn name(&self) -> &str {
        "testcov_0095"
    }

    fn description(&self) -> &str {
        "tool registry coverage"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            }
        })
    }

    async fn execute(
        &self,
        _args: serde_json::Value,
    ) -> anyhow::Result<crate::app::agent::tools::ToolResult> {
        Ok(crate::app::agent::tools::ToolResult::default())
    }
}

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
fn tool_instructions_can_be_built_from_registry_and_empty_specs() {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(Testcov0095Tool)];
    let from_registry = build_tool_instructions(&tools);
    let empty = build_tool_instructions_from_specs(&[]);

    assert!(from_registry.contains("**testcov_0095**"));
    assert!(from_registry.contains("tool registry coverage"));
    assert!(from_registry.contains(r#""input""#));
    assert!(empty.contains("## Tool Use Protocol"));
    assert!(empty.contains("### Available Tools"));
    assert!(!empty.contains("**testcov_0095**"));
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

#[test]
fn shell_policy_read_only_returns_disabled_message() {
    let autonomy = crate::app::agent::config::AutonomyConfig {
        level: crate::app::agent::security::AutonomyLevel::ReadOnly,
        allowed_commands: vec!["git".to_string()],
        ..Default::default()
    };

    let instructions = build_shell_policy_instructions(&autonomy);

    assert!(instructions.contains("- Autonomy level: `read_only`"));
    assert!(instructions.contains("Bash execution is disabled"));
    assert!(!instructions.contains("Allowed commands:"));
}

#[test]
fn shell_policy_handles_wildcard_empty_and_risk_flags() {
    let wildcard = crate::app::agent::config::AutonomyConfig {
        level: crate::app::agent::security::AutonomyLevel::Full,
        allowed_commands: vec!["*".to_string(), "git".to_string()],
        require_approval_for_medium_risk: false,
        block_high_risk_commands: false,
        ..Default::default()
    };
    let empty = crate::app::agent::config::AutonomyConfig {
        level: crate::app::agent::security::AutonomyLevel::Full,
        allowed_commands: vec![" ".to_string()],
        require_approval_for_medium_risk: false,
        block_high_risk_commands: true,
        ..Default::default()
    };

    let wildcard_instructions = build_shell_policy_instructions(&wildcard);
    let empty_instructions = build_shell_policy_instructions(&empty);

    assert!(wildcard_instructions.contains("wildcard `*`"));
    assert!(!wildcard_instructions.contains("Medium-risk bash commands require"));
    assert!(!wildcard_instructions.contains("High-risk bash commands are blocked"));
    assert!(empty_instructions.contains("Allowed commands: none configured"));
    assert!(empty_instructions.contains("High-risk bash commands are blocked"));
}

#[test]
fn shell_policy_limits_displayed_allowed_commands() {
    let autonomy = crate::app::agent::config::AutonomyConfig {
        level: crate::app::agent::security::AutonomyLevel::Supervised,
        allowed_commands: (0..66).map(|index| format!("cmd-{index:02}")).collect(),
        require_approval_for_medium_risk: true,
        block_high_risk_commands: false,
        ..Default::default()
    };

    let instructions = build_shell_policy_instructions(&autonomy);

    assert!(instructions.contains("`cmd-00`"));
    assert!(instructions.contains("`cmd-63`"));
    assert!(instructions.contains("(+2 more)"));
    assert!(!instructions.contains("`cmd-64`"));
    assert!(instructions.contains("Medium-risk bash commands require explicit approval"));
}
