//! 研究阶段模块 —— 在主响应之前的主动信息收集
//!
//! 本模块实现了一个"研究阶段"机制，允许智能体在生成主要响应之前，
//! 先使用可用工具进行聚焦式的信息收集。这创造了一个"思考"阶段，
//! 智能体可以在此阶段探索代码库、搜索记忆或获取外部数据。
//!
//! # 支持的调用模式
//!
//! - **原生工具调用**：适用于 OpenAI、Anthropic、Bedrock 等支持原生工具调用的提供商
//! - **提示引导工具调用**：适用于 Gemini 等不支持原生工具调用的提供商，
//!   通过在系统提示中嵌入工具指令来实现
//!
//! # 工作流程
//!
//! 1. 根据配置判断是否触发研究阶段
//! 2. 构建研究专用的系统提示和工具规格
//! 3. 执行 LLM + 工具的迭代循环，收集信息
//! 4. 当智能体认为信息充足时，返回收集到的上下文
//!
//! # 示例
//!
//! ```ignore
//! use crate::app::agent::agent::research::{run_research_phase, should_trigger};
//!
//! // 判断是否触发
//! if should_trigger(&config, &user_message) {
//!     // 执行研究阶段
//!     let result = run_research_phase(
//!         &config,
//!         provider.as_ref(),
//!         &tools,
//!         &user_message,
//!         model,
//!         temperature,
//!         observer,
//!     ).await?;
//!
//!     // 使用收集到的上下文
//!     println!("收集到的上下文: {}", result.context);
//! }
//! ```

use crate::app::agent::agent::dispatcher::{ToolDispatcher, XmlToolDispatcher};
use crate::app::agent::config::{ResearchPhaseConfig, ResearchTrigger};
use crate::app::agent::observability::Observer;
use crate::app::agent::providers::traits::build_tool_instructions_text;
use crate::app::agent::providers::{ChatMessage, ChatRequest, ChatResponse, Provider, ToolCall};
use crate::app::agent::tools::{Tool, ToolResult, ToolSpec};
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 研究阶段的执行结果
///
/// 包含研究阶段收集到的所有信息，包括格式化的上下文文本、
/// 工具调用统计、执行时长以及每次工具调用的摘要。
///
/// # 字段说明
///
/// - `context`: 收集到的上下文信息，已格式化为可注入主提示的文本
/// - `tool_call_count`: 研究阶段中执行的工具调用总数
/// - `duration`: 研究阶段的总执行时长
/// - `tool_summaries`: 每次工具调用的摘要列表
#[derive(Debug, Clone)]
pub struct ResearchResult {
    /// 收集到的上下文信息（已格式化，可注入主提示）
    pub context: String,
    /// 研究阶段中执行的工具调用次数
    pub tool_call_count: usize,
    /// 研究阶段的总执行时长
    pub duration: Duration,
    /// 工具调用摘要列表，记录每次调用的名称、参数和结果
    pub tool_summaries: Vec<ToolSummary>,
}

/// 单次工具调用的摘要
///
/// 记录研究阶段中某次工具调用的关键信息，包括工具名称、
/// 参数预览、结果预览以及执行状态。
///
/// # 字段说明
///
/// - `tool_name`: 被调用的工具名称
/// - `arguments_preview`: 工具参数的预览文本（截断显示）
/// - `result_preview`: 工具执行结果的预览文本（截断显示）
/// - `success`: 工具调用是否成功
#[derive(Debug, Clone)]
pub struct ToolSummary {
    /// 工具名称
    pub tool_name: String,
    /// 参数预览（前100个字符）
    pub arguments_preview: String,
    /// 结果预览（前200个字符）
    pub result_preview: String,
    /// 是否执行成功
    pub success: bool,
}

/// 判断是否应该为当前消息触发研究阶段
///
/// 根据配置中的触发策略，判断是否需要执行研究阶段。
/// 支持多种触发模式：从不触发、总是触发、关键词触发、长度触发、问句触发。
///
/// # 参数
///
/// - `config`: 研究阶段的配置对象
/// - `message`: 用户输入的消息文本
///
/// # 返回值
///
/// 返回 `true` 表示应该触发研究阶段，`false` 表示不需要
///
/// # 触发策略说明
///
/// - `Never`: 从不触发研究阶段
/// - `Always`: 总是触发研究阶段（如果启用）
/// - `Keywords`: 当消息包含配置的关键词时触发
/// - `Length`: 当消息长度达到最小阈值时触发
/// - `Question`: 当消息包含问号时触发
///
/// # 示例
///
/// ```ignore
/// let config = ResearchPhaseConfig {
///     enabled: true,
///     trigger: ResearchTrigger::Question,
///     keywords: vec!["search".to_string()],
///     min_message_length: 50,
///     // ...
/// };
///
/// assert!(should_trigger(&config, "这是什么？"));  // 包含问号
/// assert!(!should_trigger(&config, "搜索文件"));    // 不包含问号
/// ```
pub fn should_trigger(config: &ResearchPhaseConfig, message: &str) -> bool {
    // 如果研究阶段功能未启用，直接返回 false
    if !config.enabled {
        return false;
    }

    // 根据配置的触发策略进行判断
    match config.trigger {
        // 从不触发
        ResearchTrigger::Never => false,
        // 总是触发
        ResearchTrigger::Always => true,
        // 关键词触发：检查消息是否包含任一配置的关键词（不区分大小写）
        ResearchTrigger::Keywords => {
            let message_lower = message.to_lowercase();
            config.keywords.iter().any(|kw| message_lower.contains(&kw.to_lowercase()))
        }
        // 长度触发：检查消息长度是否达到最小阈值
        ResearchTrigger::Length => message.len() >= config.min_message_length,
        // 问句触发：检查消息是否包含问号
        ResearchTrigger::Question => message.contains('?'),
    }
}

/// 研究阶段的默认系统提示
///
/// 此提示定义了研究模式的规则和期望行为。智能体在此模式下
/// 应专注于收集事实信息，而不是直接回答用户问题。
///
/// # 关键规则
///
/// 1. 使用工具进行搜索、读取文件、检查状态或获取数据
/// 2. 专注于收集事实，而非直接回答
/// 3. 高效执行，只收集必要信息
/// 4. 收集完毕后以 "[RESEARCH COMPLETE]" 开头总结
///
/// # 禁止行为
///
/// - 直接回答用户问题
/// - 修改文件
/// - 执行破坏性命令
const RESEARCH_SYSTEM_PROMPT: &str = r#"You are in RESEARCH MODE. Your task is to gather information that will help answer the user's question.

RULES:
1. Use tools to search, read files, check status, or fetch data
2. Focus on gathering FACTS, not answering yet
3. Be efficient — only gather what's needed
4. After gathering enough info, respond with a summary starting with "[RESEARCH COMPLETE]"

DO NOT:
- Answer the user's question directly
- Make changes to files
- Execute destructive commands

When you have enough information, summarize what you found in this format:
[RESEARCH COMPLETE]
- Finding 1: ...
- Finding 2: ...
- Finding 3: ...
"#;

/// 执行研究阶段
///
/// 运行一个聚焦的 LLM + 工具循环来收集信息，然后将收集到的
/// 上下文返回，以便注入到主对话中。支持原生工具调用和提示引导
/// 工具调用两种模式。
///
/// # 参数
///
/// - `config`: 研究阶段的配置对象，控制触发条件、迭代次数等
/// - `provider`: LLM 提供商的 trait 对象，用于调用模型
/// - `tools`: 可用工具列表，智能体在研究阶段可以调用这些工具
/// - `user_message`: 用户的原始消息，研究将围绕此消息进行
/// - `model`: 要使用的模型标识符
/// - `temperature`: 生成温度参数，控制输出的随机性
/// - `_observer`: 观察者对象（当前未使用，保留用于未来扩展）
///
/// # 返回值
///
/// 返回 `Result<ResearchResult>`，包含：
/// - 成功时：收集到的上下文、工具调用统计、执行时长等信息
/// - 失败时：错误信息
///
/// # 工作流程
///
/// 1. 初始化计时器和状态变量
/// 2. 检测提供商是否支持原生工具调用
/// 3. 构建工具规格列表
/// 4. 构建系统提示（根据工具调用模式调整）
/// 5. 进入研究循环：
///    a. 构建完整的消息列表
///    b. 调用 LLM
///    c. 检查是否完成（响应包含 "[RESEARCH COMPLETE]"）
///    d. 解析工具调用（原生或 XML 格式）
///    e. 执行工具调用并将结果添加到对话历史
///    f. 检查迭代限制
/// 6. 返回收集到的结果
///
/// # 错误处理
///
/// - LLM 调用失败会返回错误
/// - 工具调用失败会记录在结果中，但不会中断研究阶段
///
/// # 示例
///
/// ```ignore
/// let result = run_research_phase(
///     &config,
///     provider.as_ref(),
///     &tools,
///     "请帮我分析这个项目的架构",
///     "gpt-4",
///     0.7,
///     observer,
/// ).await?;
///
/// println!("研究耗时: {:?}", result.duration);
/// println!("工具调用次数: {}", result.tool_call_count);
/// ```
pub async fn run_research_phase(
    config: &ResearchPhaseConfig,
    provider: &dyn Provider,
    tools: &[Box<dyn Tool>],
    user_message: &str,
    model: &str,
    temperature: f64,
    _observer: Arc<dyn Observer>,
) -> Result<ResearchResult> {
    // 记录开始时间，用于计算总执行时长
    let start = Instant::now();
    // 存储所有工具调用的摘要
    let mut tool_summaries = Vec::new();
    // 存储收集到的上下文信息
    let mut collected_context = String::new();
    // 当前迭代次数
    let mut iteration = 0;

    // 检测提供商是否支持原生工具调用
    let uses_native_tools = provider.supports_native_tools();

    // 构建工具规格列表（用于原生工具调用或生成工具指令）
    let tool_specs: Vec<ToolSpec> = tools.iter().map(|tool| tool.spec()).collect();

    // 构建系统提示
    // 如果配置了自定义前缀，则组合使用；否则使用默认提示
    // 对于提示引导模式的提供商，还需要附加工具指令
    let base_prompt = if config.system_prompt_prefix.is_empty() {
        RESEARCH_SYSTEM_PROMPT.to_string()
    } else {
        format!("{}\n\n{}", config.system_prompt_prefix, RESEARCH_SYSTEM_PROMPT)
    };

    let system_prompt = if uses_native_tools {
        // 原生工具调用：直接使用基础提示
        base_prompt
    } else {
        // 提示引导：在系统提示中附加工具指令文本
        format!("{}\n\n{}", base_prompt, build_tool_instructions_text(&tool_specs))
    };

    // 初始化对话历史，以研究任务的形式构建第一条用户消息
    let mut messages = vec![ChatMessage::user(format!(
        "Research the following question to gather relevant information:\n\n{}",
        user_message
    ))];

    // 研究循环：最多执行配置的最大迭代次数
    while iteration < config.max_iterations {
        iteration += 1;

        // 如果配置了显示进度，记录当前迭代
        if config.show_progress {
            tracing::info!(iteration, "Research phase iteration");
        }

        // 构建完整的消息列表：系统提示 + 对话历史
        let mut full_messages = vec![ChatMessage::system(&system_prompt)];
        full_messages.extend(messages.iter().cloned());

        // 构建 LLM 请求
        let request = ChatRequest {
            messages: &full_messages,
            // 原生工具调用时传递工具规格；提示引导模式不传递（工具指令已在系统提示中）
            tools: if uses_native_tools { Some(&tool_specs) } else { None },
        };

        // 调用 LLM
        let response: ChatResponse = provider.chat(request, model, temperature).await?;

        // 检查研究是否完成：响应中是否包含 "[RESEARCH COMPLETE]" 标记
        if let Some(ref text) = response.text {
            if text.contains("[RESEARCH COMPLETE]") {
                // 提取总结部分（从标记开始到结尾）
                if let Some(idx) = text.find("[RESEARCH COMPLETE]") {
                    collected_context = text[idx..].to_string();
                }
                break;
            }
        }

        // 解析工具调用
        // 原生工具调用：直接从响应中获取
        // 提示引导：从响应文本中解析 XML <invoke> 标签
        let tool_calls: Vec<ToolCall> = if uses_native_tools {
            response.tool_calls.clone()
        } else {
            // 使用 XmlToolDispatcher 解析响应中的 XML 工具调用标签
            let dispatcher = XmlToolDispatcher;
            let (_, parsed) = dispatcher.parse_response(&response);
            parsed
                .into_iter()
                .enumerate()
                .map(|(i, p)| ToolCall {
                    // 如果解析结果中没有 ID，生成一个默认的
                    id: p.tool_call_id.unwrap_or_else(|| format!("tc_{}_{}", iteration, i)),
                    name: p.name,
                    arguments: serde_json::to_string(&p.arguments).unwrap_or_default(),
                })
                .collect()
        };

        // 如果没有工具调用，说明智能体认为已经完成（但没有显式标记）
        if tool_calls.is_empty() {
            if let Some(text) = response.text {
                collected_context = text;
            }
            break;
        }

        // 执行所有工具调用
        for tool_call in &tool_calls {
            // 调用工具并获取结果
            let tool_result = execute_tool_call(tools, tool_call).await;

            // 创建工具调用摘要
            let summary = ToolSummary {
                tool_name: tool_call.name.clone(),
                arguments_preview: truncate(&tool_call.arguments, 100),
                result_preview: truncate(&tool_result.output, 200),
                success: tool_result.success,
            };

            // 如果配置了显示进度，记录工具调用信息
            if config.show_progress {
                tracing::info!(
                    tool = %summary.tool_name,
                    success = summary.success,
                    "Research tool call"
                );
            }

            tool_summaries.push(summary);

            // 将工具调用和结果添加到对话历史，以便下一轮 LLM 可以看到
            messages.push(ChatMessage::assistant(format!(
                "Called tool `{}` with arguments: {}",
                tool_call.name, tool_call.arguments
            )));
            messages.push(ChatMessage::user(format!("Tool result:\n{}", tool_result.output)));
        }
    }

    // 计算总执行时长
    let duration = start.elapsed();

    // 返回研究结果
    Ok(ResearchResult {
        context: collected_context,
        tool_call_count: tool_summaries.len(),
        duration,
        tool_summaries,
    })
}

/// 执行单个工具调用
///
/// 根据工具调用请求找到对应的工具并执行，返回执行结果。
/// 如果工具不存在或执行失败，返回包含错误信息的结果对象。
///
/// # 参数
///
/// - `tools`: 可用工具列表
/// - `tool_call`: 工具调用请求，包含工具名称和参数
///
/// # 返回值
///
/// 返回 `ToolResult`，包含：
/// - `success`: 执行是否成功
/// - `output`: 工具的输出文本
/// - `error`: 如果失败，包含错误信息
///
/// # 错误处理
///
/// - 工具不存在：返回包含 "Unknown tool" 错误的结果
/// - 参数解析失败：使用空参数继续执行
/// - 执行失败：返回包含错误信息的结果
async fn execute_tool_call(tools: &[Box<dyn Tool>], tool_call: &ToolCall) -> ToolResult {
    // 在工具列表中查找匹配的工具
    let tool = tools.iter().find(|t| t.name() == tool_call.name);

    match tool {
        Some(t) => {
            // 解析参数 JSON，失败时使用空对象
            let args: serde_json::Value = serde_json::from_str(&tool_call.arguments)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            // 执行工具并处理结果
            match t.execute(args).await {
                Ok(result) => result,
                Err(e) => ToolResult {
                    success: false,
                    output: format!("Error: {}", e),
                    error: Some(e.to_string()),
                },
            }
        }
        // 工具不存在，返回错误结果
        None => ToolResult {
            success: false,
            output: format!("Unknown tool: {}", tool_call.name),
            error: Some(format!("Unknown tool: {}", tool_call.name)),
        },
    }
}

/// 截断字符串并添加省略号
///
/// 如果字符串长度超过指定的最大长度，则截断并添加 "..." 后缀。
/// 否则返回原字符串。
///
/// # 参数
///
/// - `s`: 要截断的字符串
/// - `max_len`: 最大长度（包括省略号）
///
/// # 返回值
///
/// 返回截断后的字符串。如果原字符串长度不超过 `max_len`，则返回原字符串。
///
/// # 示例
///
/// ```ignore
/// assert_eq!(truncate("hello", 10), "hello");
/// assert_eq!(truncate("hello world", 8), "hello...");
/// ```
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // 减去 3 个字符用于省略号
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
