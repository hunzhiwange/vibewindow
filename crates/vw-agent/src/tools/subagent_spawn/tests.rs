//! 子代理生成工具的单元测试模块
//!
//! 本模块包含 `SubAgentSpawnTool` 的全面测试套件，验证子代理生成功能的各种场景：
//!
//! # 测试覆盖范围
//!
//! - **基本功能**：工具名称、schema、描述的正确性
//! - **参数验证**：必需参数缺失、空白值、无效值的处理
//! - **安全策略**：只读模式、速率限制的执行
//! - **并发控制**：最大并发数限制的遵守
//! - **代理管理**：未知代理、未配置代理的处理
//!
//! # 测试工具
//!
//! 模块提供辅助函数用于创建测试所需的：
//! - `test_security()` - 默认安全策略
//! - `sample_agents()` - 示例代理配置
//! - `make_tool()` - 配置好的工具实例

use super::super::*;
use super::*;
use crate::app::agent::security::{AutonomyLevel, SecurityPolicy};
use serde_json::json;

/// 创建用于测试的默认安全策略
///
/// 返回一个具有默认配置的安全策略实例，用于大多数测试场景。
/// 特定测试（如只读模式、速率限制）会创建自定义策略。
///
/// # 返回值
///
/// 返回包装在 `Arc` 中的默认 `SecurityPolicy` 实例
fn test_security() -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy::default())
}

/// 创建用于测试的示例代理配置映射
///
/// 构建一个包含 "researcher" 代理的配置映射，该代理配置包括：
/// - Ollama 提供者
/// - Llama3 模型
/// - 研究助手系统提示词
/// - 较低的温度参数（0.3）
/// - 适当的深度和迭代限制
///
/// # 返回值
///
/// 返回包含示例代理配置的 `HashMap`，键为代理名称
///
/// # 示例
///
/// ```ignore
/// let agents = sample_agents();
/// assert!(agents.contains_key("researcher"));
/// ```
fn sample_agents() -> HashMap<String, DelegateAgentConfig> {
    let mut agents = HashMap::new();
    agents.insert(
        "researcher".to_string(),
        DelegateAgentConfig {
            label: None,
            description: None,
            builtin: false,
            mode: "all".to_string(),
            enabled: true,
            provider: "ollama".to_string(),
            model: "llama3".to_string(),
            system_prompt: Some("You are a research assistant.".to_string()),
            api_key: None,
            temperature: Some(0.3),
            top_p: None,
            identity_format: None,
            hidden: false,
            max_depth: 3,
            agentic: false,
            allowed_tools: Vec::new(),
            allowed_skills: Vec::new(),
            options: HashMap::new(),
            permission: Value::Null,
            max_iterations: 10,
            steps: None,
        },
    );
    agents
}

/// 创建配置好的 SubAgentSpawnTool 实例
///
/// 使用提供的代理配置和安全策略创建工具实例，
/// 其他参数使用默认值。
///
/// # 参数
///
/// - `agents`: 代理名称到配置的映射
/// - `security`: 安全策略（包装在 Arc 中）
///
/// # 返回值
///
/// 返回配置完整的 `SubAgentSpawnTool` 实例
///
/// # 示例
///
/// ```ignore
/// let tool = make_tool(sample_agents(), test_security());
/// assert_eq!(tool.name(), "subagent_spawn");
/// ```
fn make_tool(
    agents: HashMap<String, DelegateAgentConfig>,
    security: Arc<SecurityPolicy>,
) -> SubAgentSpawnTool {
    SubAgentSpawnTool::new(
        agents,
        None,
        security,
        crate::app::agent::providers::ProviderRuntimeOptions::default(),
        Arc::new(SubAgentRegistry::new()),
        Arc::new(Vec::new()),
        crate::app::agent::config::MultimodalConfig::default(),
    )
}

#[test]
fn subagent_spawn_merges_workspace_identity_context_with_agent_prompt() {
    let tool = make_tool(sample_agents(), test_security())
        .with_workspace_identity_context("## Project Context\n\nAGENTS".to_string());

    let merged =
        tool.merged_system_prompt("researcher", Some("You are a research assistant.")).unwrap();

    assert!(merged.contains("## Project Context"));
    assert!(merged.contains("AGENTS"));
    assert!(merged.contains("You are a research assistant."));
}

#[test]
fn subagent_spawn_uses_workspace_identity_context_without_agent_prompt() {
    let tool = make_tool(sample_agents(), test_security())
        .with_workspace_identity_context("## Project Context\n\nIDENTITY".to_string());

    let merged = tool.merged_system_prompt("researcher", None).unwrap();

    assert_eq!(merged, "## Project Context\n\nIDENTITY");
}

/// 测试工具名称和参数 schema 的正确性
///
/// 验证：
/// - 工具名称为 "subagent_spawn"
/// - Schema 包含 agent、task、context 属性
/// - agent 和 task 为必需参数
/// - 禁止额外属性
#[test]
fn name_and_schema() {
    let tool = make_tool(sample_agents(), test_security());
    assert_eq!(tool.name(), "subagent_spawn");
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["agent"].is_object());
    assert!(schema["properties"]["task"].is_object());
    assert!(schema["properties"]["context"].is_object());
    let required = schema["required"].as_array().unwrap();
    assert!(required.contains(&json!("agent")));
    assert!(required.contains(&json!("task")));
    assert_eq!(schema["additionalProperties"], json!(false));
}

/// 测试工具描述不为空
///
/// 确保工具返回有效的描述文本，用于在工具列表中展示
#[test]
fn description_not_empty() {
    let tool = make_tool(sample_agents(), test_security());
    assert!(!tool.description().is_empty());
}

/// 测试缺少 agent 参数时的错误处理
///
/// 当只提供 task 参数而不提供 agent 参数时，
/// 工具应返回错误而不是 panic
#[tokio::test]
async fn missing_agent_param() {
    let tool = make_tool(sample_agents(), test_security());
    let result = tool.execute(json!({"task": "test"})).await;
    assert!(result.is_err());
}

/// 测试缺少 task 参数时的错误处理
///
/// 当只提供 agent 参数而不提供 task 参数时，
/// 工具应返回错误而不是 panic
#[tokio::test]
async fn missing_task_param() {
    let tool = make_tool(sample_agents(), test_security());
    let result = tool.execute(json!({"agent": "researcher"})).await;
    assert!(result.is_err());
}

/// 测试空白 agent 名称被拒绝
///
/// 当提供空白字符串作为 agent 名称时，
/// 工具应返回失败结果，包含适当的错误信息
#[tokio::test]
async fn blank_agent_rejected() {
    let tool = make_tool(sample_agents(), test_security());
    let result = tool.execute(json!({"agent": "  ", "task": "test"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("must not be empty"));
}

/// 测试空白 task 内容被拒绝
///
/// 当提供空白字符串作为 task 内容时，
/// 工具应返回失败结果，包含适当的错误信息
#[tokio::test]
async fn blank_task_rejected() {
    let tool = make_tool(sample_agents(), test_security());
    let result = tool.execute(json!({"agent": "researcher", "task": "  "})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("must not be empty"));
}

/// 测试未知代理返回错误
///
/// 当请求的代理名称在配置中不存在时，
/// 工具应返回失败结果，包含 "Unknown agent" 错误信息
#[tokio::test]
async fn unknown_agent_returns_error() {
    let tool = make_tool(sample_agents(), test_security());
    let result = tool.execute(json!({"agent": "nonexistent", "task": "test"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("Unknown agent"));
}

/// 测试只读模式下 spawn 被阻止
///
/// 当安全策略的自主级别设置为 ReadOnly 时，
/// 工具应拒绝生成子代理，返回包含 "read-only mode" 的错误
#[tokio::test]
async fn spawn_blocked_in_readonly_mode() {
    // 创建只读模式的安全策略
    let readonly =
        Arc::new(SecurityPolicy { autonomy: AutonomyLevel::ReadOnly, ..SecurityPolicy::default() });
    let tool = make_tool(sample_agents(), readonly);
    let result = tool.execute(json!({"agent": "researcher", "task": "test"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.as_deref().unwrap_or("").contains("read-only mode"));
}

/// 测试速率限制时 spawn 被阻止
///
/// 当安全策略的每小时最大操作数设置为 0 时，
/// 工具应拒绝生成子代理，返回包含 "Rate limit exceeded" 的错误
#[tokio::test]
async fn spawn_blocked_when_rate_limited() {
    // 创建速率限制的安全策略（每小时最大操作数为 0）
    let limited = Arc::new(SecurityPolicy { max_actions_per_hour: 0, ..SecurityPolicy::default() });
    let tool = make_tool(sample_agents(), limited);
    let result = tool.execute(json!({"agent": "researcher", "task": "test"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.as_deref().unwrap_or("").contains("Rate limit exceeded"));
}

/// 测试 spawn 返回会话 ID
///
/// 验证成功生成子代理时返回正确的会话信息：
/// - session_id 应为字符串
/// - status 应为 "running"
///
/// 注意：即使后台任务因无效提供者失败，
/// spawn 本身也应立即返回 session_id。
/// 对于 ollama，即使没有运行中的服务器也应成功创建提供者。
/// 结果可能成功（spawn）或失败（无效提供者），取决于环境，
/// 但不应发生 panic。
#[tokio::test]
async fn spawn_returns_session_id() {
    let tool = make_tool(sample_agents(), test_security());
    let result = tool.execute(json!({"agent": "researcher", "task": "test task"})).await.unwrap();
    // 如果 spawn 成功，验证输出格式
    if result.success {
        let output: serde_json::Value = serde_json::from_str(&result.output).unwrap();
        assert!(output["session_id"].is_string());
        assert_eq!(output["status"], "running");
    }
    // 无论成功或失败，都不应 panic
}

/// 测试未配置代理时的 spawn 行为
///
/// 当没有配置任何代理时，尝试生成子代理应返回失败，
/// 错误信息包含 "none configured"
#[tokio::test]
async fn spawn_no_agents_configured() {
    let tool = make_tool(HashMap::new(), test_security());
    let result = tool.execute(json!({"agent": "any", "task": "test"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("none configured"));
}

/// 测试 spawn 遵守并发限制
///
/// 验证当已达到最大并发子代理数时，新的 spawn 请求被拒绝：
/// 1. 创建一个注册表
/// 2. 填充至最大并发数
/// 3. 尝试生成新的子代理
/// 4. 验证请求被拒绝，错误信息包含 "Maximum concurrent"
#[tokio::test]
async fn spawn_respects_concurrent_limit() {
    let registry = Arc::new(SubAgentRegistry::new());

    // 用运行中的会话填满注册表，达到最大并发数
    for i in 0..MAX_CONCURRENT_SUBAGENTS {
        registry.insert(SubAgentSession {
            id: format!("s{i}"),
            agent_name: "agent".to_string(),
            title: None,
            task: "task".to_string(),
            metadata: serde_json::Value::Object(Default::default()),
            status: SubAgentStatus::Running,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            result: None,
            handle: None,
        });
    }

    // 使用已满的注册表创建工具
    let tool = SubAgentSpawnTool::new(
        sample_agents(),
        None,
        test_security(),
        crate::app::agent::providers::ProviderRuntimeOptions::default(),
        registry,
        Arc::new(Vec::new()),
        crate::app::agent::config::MultimodalConfig::default(),
    );

    // 尝试生成新的子代理，应被拒绝
    let result = tool.execute(json!({"agent": "researcher", "task": "test"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("Maximum concurrent"));
}

/// 测试 schema 中列出可用代理名称
///
/// 验证工具的参数 schema 中，agent 参数的描述包含
/// 所有已配置的代理名称（如 "researcher"）
#[tokio::test]
async fn schema_lists_agent_names() {
    let tool = make_tool(sample_agents(), test_security());
    let schema = tool.parameters_schema();
    let desc = schema["properties"]["agent"]["description"].as_str().unwrap();
    assert!(desc.contains("researcher"));
}
