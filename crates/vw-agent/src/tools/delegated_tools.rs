use super::traits::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::sync::Arc;

const BLOCKED_DELEGATE_TOOL_IDS: [&str; 6] =
    ["AgentTool", "batch", "delegate", "subagent_manage", "subagent_spawn", "agent_tool"];

pub(crate) fn build_agentic_tools(
    parent_tools: &[Arc<dyn Tool>],
    allowed_tools: &[String],
    allowed_skills: &[String],
) -> Vec<Box<dyn Tool>> {
    let allowed_tools = allowed_tools
        .iter()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .collect::<BTreeSet<_>>();

    let allowed_skills = allowed_skills
        .iter()
        .map(|name| name.trim())
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();

    parent_tools
        .iter()
        .filter_map(|tool| {
            let spec = tool.spec();
            let tool_id = spec.id.as_str();
            if !tool_allowed(&allowed_tools, &spec) || tool_blocked_for_delegation(tool_id) {
                return None;
            }
            if tool_id == "skill" {
                if allowed_skills.is_empty() {
                    return None;
                }
                return Some(Box::new(AllowedSkillTool::new(tool.clone(), allowed_skills.clone()))
                    as Box<dyn Tool>);
            }
            Some(Box::new(DelegatedToolRef::new(tool.clone())) as Box<dyn Tool>)
        })
        .collect()
}

fn tool_allowed(
    allowed_tools: &BTreeSet<&str>,
    spec: &crate::app::agent::tools::traits::ToolSpec,
) -> bool {
    allowed_tools.contains(spec.id.as_str())
        || spec.aliases.iter().any(|alias| allowed_tools.contains(alias.as_str()))
}

fn tool_blocked_for_delegation(tool_id: &str) -> bool {
    BLOCKED_DELEGATE_TOOL_IDS.contains(&tool_id)
}

struct DelegatedToolRef {
    inner: Arc<dyn Tool>,
}

impl DelegatedToolRef {
    fn new(inner: Arc<dyn Tool>) -> Self {
        Self { inner }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for DelegatedToolRef {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters_schema(&self) -> Value {
        self.inner.parameters_schema()
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        self.inner.execute(args).await
    }
}

struct AllowedSkillTool {
    inner: Arc<dyn Tool>,
    allowed_skills: BTreeSet<String>,
    description: String,
}

impl AllowedSkillTool {
    fn new(inner: Arc<dyn Tool>, allowed_skills: BTreeSet<String>) -> Self {
        let allowed = allowed_skills.iter().cloned().collect::<Vec<_>>().join(", ");
        Self {
            inner,
            allowed_skills,
            description: format!("加载委托代理允许的技能。可用技能：{allowed}"),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for AllowedSkillTool {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> Value {
        let names = self.allowed_skills.iter().cloned().map(Value::String).collect::<Vec<_>>();
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "enum": names
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        let requested =
            args.get("name").and_then(Value::as_str).map(str::trim).filter(|name| !name.is_empty());

        let Some(name) = requested else {
            return self.inner.execute(args).await;
        };

        if !self.allowed_skills.contains(name) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Skill '{name}' is not allowed for this delegate agent")),
            });
        }

        self.inner.execute(args).await
    }
}

#[cfg(test)]
#[path = "delegated_tools_tests.rs"]
mod tests;
