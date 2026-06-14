//! ToolSearch 工具实现。
//!
//! 该工具只搜索当前上下文真实可见的工具 spec，并返回匹配原因与分数。它不执行工具、
//! 不扩大可见工具集合，只作为模型在复杂工具面前的发现入口。

use super::context::current_tool_use_context;
use super::registry;
use super::traits::{
    Tool, ToolCallResult, ToolCallTelemetry, ToolRenderHint, ToolResult, ToolSpec,
};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use vw_api_types::tools::ToolResultContentDto;

#[derive(Debug, Clone, Deserialize)]
struct Args {
    /// 搜索词，会同时匹配工具 id、显示名、别名和描述。
    query: String,
    /// 返回结果上限。
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Clone, Default)]
pub struct ToolSearchTool;

impl ToolSearchTool {
    /// 创建 ToolSearch 工具实例。
    ///
    /// 返回值：无状态工具实例。
    /// 错误处理：该函数不返回错误。
    pub fn new() -> Self {
        Self
    }
}

/// 单个工具匹配结果。
#[derive(Debug)]
struct MatchRow {
    /// 排序分数，越高越靠前。
    score: usize,
    /// 命中原因。
    reason: String,
    /// 被命中的工具 spec。
    spec: super::traits::ToolSpec,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ToolSearchTool {
    fn name(&self) -> &str {
        crate::app::agent::tools::TOOL_SEARCH_TOOL_ID
    }

    fn description(&self) -> &str {
        "搜索当前上下文中真实可见的工具，并解释每个结果为什么命中。"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "query": {
                    "type": "string",
                    "description": "要搜索的工具名称、描述或关键词。"
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 50,
                    "description": "返回结果上限，默认 10。"
                }
            },
            "required": ["query"]
        })
    }

    fn spec(&self) -> ToolSpec {
        ToolSpec::new(self.name(), self.description(), self.parameters_schema())
            .with_display_name("ToolSearch")
            .with_aliases(
                crate::app::agent::tools::TOOL_SEARCH_TOOL_ALIASES
                    .iter()
                    .map(|alias| alias.to_string())
                    .collect::<Vec<_>>(),
            )
            .with_read_only(true)
            .with_destructive(false)
            .with_concurrency_safe(true)
            .with_requires_user_interaction(false)
            .with_strict(true)
    }

    async fn call(&self, input: Value) -> anyhow::Result<ToolCallResult> {
        let args: Args = serde_json::from_value(input)
            .map_err(|error| anyhow::anyhow!("invalid tool search arguments: {error}"))?;
        let context = current_tool_use_context()
            .ok_or_else(|| anyhow::anyhow!("missing active tool context"))?;
        let query = args.query.trim().to_lowercase();
        if query.is_empty() {
            return Err(anyhow::anyhow!("query must not be empty"));
        }

        // registry::specs 会按当前模型上下文返回可见工具；搜索层不自己拼装能力列表，
        // 避免展示模型实际不能调用的工具。
        let model_ref = match (context.provider(), context.model()) {
            (Some(provider), Some(model)) => Some(format!("{provider}/{model}")),
            (_, Some(model)) => Some(model.to_string()),
            _ => None,
        };
        let mut matches = registry::specs(model_ref.as_deref())
            .into_iter()
            .filter_map(|spec| score_spec(&spec, &query))
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| {
            // 分数相同按 id 稳定排序，确保模型和测试看到确定性结果。
            right.score.cmp(&left.score).then_with(|| left.spec.id.cmp(&right.spec.id))
        });

        let items = matches
            .into_iter()
            .take(args.limit.clamp(1, 50))
            .map(|row| {
                json!({
                    "id": row.spec.id,
                    "display_name": row.spec.display_name,
                    "description": row.spec.description,
                    "aliases": row.spec.aliases,
                    "read_only": row.spec.read_only,
                    "reason": row.reason,
                    "score": row.score
                })
            })
            .collect::<Vec<_>>();
        let data = json!({
            "query": args.query,
            "count": items.len(),
            "items": items
        });

        Ok(ToolCallResult {
            data: data.clone(),
            model_result: Value::String(format!("Found {} matching tool(s)", data["count"])),
            content_blocks: vec![ToolResultContentDto::Json { value: data.clone() }],
            render_hint: Some(ToolRenderHint {
                title: Some("ToolSearch".to_string()),
                kind: Some("tool_search".to_string()),
                summary: Some(format!("Found {} matching tool(s)", data["count"])),
                metadata: json!({ "query": query }),
            }),
            telemetry: Some(ToolCallTelemetry { success: true, ..ToolCallTelemetry::default() }),
            ..ToolCallResult::default()
        })
    }

    async fn execute(&self, _args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { success: true, output: "Searched tools".to_string(), error: None })
    }
}

fn score_spec(spec: &ToolSpec, query: &str) -> Option<MatchRow> {
    let id = spec.id.to_lowercase();
    let display_name = spec.display_name.to_lowercase();
    let description = spec.description.to_lowercase();
    let aliases = spec.aliases.iter().map(|alias| alias.to_lowercase()).collect::<Vec<_>>();

    if id == query || display_name == query || aliases.iter().any(|alias| alias == query) {
        return Some(MatchRow {
            score: 100,
            reason: "exact id or alias match".to_string(),
            spec: spec.clone(),
        });
    }
    if id.contains(query) || display_name.contains(query) {
        return Some(MatchRow { score: 80, reason: "name match".to_string(), spec: spec.clone() });
    }
    if aliases.iter().any(|alias| alias.contains(query)) {
        return Some(MatchRow { score: 70, reason: "alias match".to_string(), spec: spec.clone() });
    }
    if description.contains(query) {
        return Some(MatchRow {
            score: 50,
            reason: "description keyword match".to_string(),
            spec: spec.clone(),
        });
    }
    None
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
