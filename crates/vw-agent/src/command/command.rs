//!
//! # 命令核心实现
//!
//! 本模块实现了 VibeWindow 代理命令系统的核心功能，包括命令的定义、加载、查询和执行事件发布。
//!
//! ## 核心组件
//!
//! ### 数据结构
//! - [`Info`]：命令的完整信息，包含名称、描述、模板、提示等元数据
//! - [`State`]：命令系统状态，维护命令名称到命令信息的映射表
//! - [`Source`]：命令来源类型枚举，区分内置命令、MCP 命令和技能命令
//! - [`ExecutedEvent`]：命令执行事件，记录命令执行的上下文信息
//!
//! ### 公共接口
//! - [`get`]：按名称查询单个命令
//! - [`list`]：获取所有可用命令列表
//! - [`hints`]：从命令模板中提取占位符提示
//! - [`publish_executed`]：发布命令执行事件
//!
//! ## 命令生命周期
//!
//! 1. **加载阶段**：通过 `load_state` 从配置、技能等来源加载所有命令
//! 2. **注册阶段**：命令被注册到全局状态中的哈希表
//! 3. **查询阶段**：通过 `get` 或 `list` 查询命令信息
//! 4. **执行阶段**：代理执行命令并发布 `ExecutedEvent` 事件
//!
//! ## 默认命令
//!
//! 系统内置两个核心命令：
//! - `init`：初始化或更新 AGENTS.md 文件
//! - `review`：审查代码变更（支持 commit/branch/pr 模式）
//!
//! ## 示例
//!
//! ```rust,ignore
//! // 查询命令
//! if let Some(cmd) = get("init").await {
//!     println!("命令: {} - {}", cmd.name, cmd.description.unwrap_or_default());
//! }
//!
//! // 提取模板提示
//! let hints = hints("Hello $1, your name is $2");
//! // 结果: ["$1", "$2"]
//! ```
//!

use crate::app::agent::bus;
use crate::app::agent::config;
use crate::app::agent::project::instance;
use crate::app::agent::skill;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::LazyLock;

/// 命令相关事件定义
///
/// 该模块定义了命令系统发出的事件类型，用于事件总线的类型安全发布。
pub mod event {
    use crate::app::agent::bus;

    /// 命令已执行事件定义
    ///
    /// 当一个命令被代理执行时，会发布此事件，携带执行的详细信息。
    /// 事件类型标识符为 `"command.executed"`。
    ///
    /// ## 用途
    /// - 记录命令执行日志
    /// - 触发后续处理流程
    /// - 统计和监控命令使用情况
    pub const EXECUTED: bus::Definition = bus::Definition { r#type: "command.executed" };
}

/// 命令执行事件数据结构
///
/// 记录命令执行时的完整上下文信息，用于事件发布和日志记录。
///
/// ## 字段说明
///
/// - `name`：执行的命令名称
/// - `session_id`：会话标识符，关联到具体的对话会话
/// - `arguments`：命令参数，通常是用户输入的原始文本
/// - `message_id`：消息标识符，用于追踪和去重
///
/// ## 序列化
///
/// 使用 camelCase 命名风格进行 JSON 序列化，以符合前端 API 规范。
///
/// ## 示例
///
/// ```rust,ignore
/// let event = ExecutedEvent {
///     name: "init".to_string(),
///     session_id: "session-123".to_string(),
///     arguments: "--force".to_string(),
///     message_id: "msg-456".to_string(),
/// };
/// publish_executed(event)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutedEvent {
    /// 命令名称
    pub name: String,

    /// 会话标识符（JSON 序列化为 sessionID）
    #[serde(rename = "sessionID")]
    pub session_id: String,

    /// 命令参数
    pub arguments: String,

    /// 消息标识符（JSON 序列化为 messageID）
    #[serde(rename = "messageID")]
    pub message_id: String,
}

/// 命令来源类型枚举
///
/// 标识命令的来源，用于区分不同类型的命令并提供统一的处理接口。
///
/// ## 变体说明
///
/// - `Command`：内置命令，由系统核心提供
/// - `Mcp`：MCP（Model Context Protocol）协议提供的命令
/// - `Skill`：技能命令，从技能系统中动态加载
///
/// ## 序列化
///
/// 枚举值序列化为小写字符串，例如：`"command"`、`"mcp"`、`"skill"`。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    /// 内置命令
    Command,
    /// MCP 协议命令
    Mcp,
    /// 技能命令
    Skill,
}

/// 命令信息结构
///
/// 完整描述一个可执行命令的所有属性，包括元数据、执行模板和行为配置。
///
/// ## 字段说明
///
/// - `name`：命令的唯一标识符，用于查询和引用
/// - `description`：命令的简短描述，用于帮助文档和 UI 展示
/// - `agent`：可选的代理标识符，指定执行该命令的特定代理
/// - `model`：可选的模型标识符，指定执行该命令的 AI 模型
/// - `source`：命令来源，用于追踪和分类
/// - `template`：命令模板，包含提示词和占位符
/// - `subtask`：是否作为子任务执行，影响执行流程
/// - `hints`：占位符提示列表，用于 UI 辅助输入
///
/// ## 序列化规则
///
/// - 可选字段（`description`、`agent`、`model`、`source`、`subtask`）
///   在值为 `None` 时不会序列化到 JSON 中
/// - 所有字段使用 snake_case 命名
///
/// ## 示例
///
/// ```rust,ignore
/// let cmd = Info {
///     name: "init".to_string(),
///     description: Some("初始化项目配置".to_string()),
///     agent: None,
///     model: None,
///     source: Some(Source::Command),
///     template: "请初始化项目...".to_string(),
///     subtask: None,
///     hints: vec!["$1".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    /// 命令的唯一名称标识符
    pub name: String,

    /// 命令的简短描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 指定执行此命令的代理（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// 指定执行此命令的 AI 模型（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// 命令来源类型（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    /// 命令模板内容，包含提示词和占位符
    pub template: String,

    /// 是否作为子任务执行（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,

    /// 占位符提示列表，用于参数提示
    pub hints: Vec<String>,
}

/// 命令系统状态
///
/// 维护命令名称到命令信息的映射表，是命令注册表的核心数据结构。
///
/// ## 设计说明
///
/// - 使用 `HashMap` 提供高效的名称查找
/// - 键为命令名称 `String`，值为命令信息 `Info`
/// - 实现 `Default` trait，默认为空命令表
///
/// ## 线程安全
///
/// 在实际使用中，`State` 会被包装在 `Arc` 中，
/// 通过 `instance::state` 函数实现线程安全的共享访问。
#[derive(Debug, Clone, Default)]
pub struct State {
    /// 命令名称到命令信息的映射表
    pub commands: HashMap<String, Info>,
}

/// 默认命令名称常量
///
/// 定义系统内置的默认命令标识符，确保在整个代码库中保持一致性。
///
/// ## 常量说明
///
/// - `INIT`：初始化命令，用于创建或更新 AGENTS.md 文件
/// - `REVIEW`：审查命令，用于审查代码变更
///
/// ## 使用示例
///
/// ```rust,ignore
/// use crate::app::agent::command::r#default;
///
/// if name == r#default::INIT {
///     // 处理初始化命令
/// }
/// ```
pub mod r#default {
    /// 初始化命令名称常量
    ///
    /// 用于创建或更新项目根目录下的 AGENTS.md 文件，
    /// 该文件定义了代理的工作协议和行为规范。
    pub const INIT: &str = "init";

    /// 审查命令名称常量
    ///
    /// 用于审查代码变更，支持三种模式：
    /// - commit：审查指定的提交
    /// - branch：审查分支变更
    /// - pr：审查 PR 变更
    /// 默认审查未提交的变更。
    pub const REVIEW: &str = "review";
}

/// 初始化命令模板内容
///
/// 从 `template/initialize.txt` 文件加载的初始化命令模板，
/// 包含初始化 AGENTS.md 文件的完整提示词。
///
/// ## 模板变量
///
/// - `${path}`：项目工作树的根路径
///
/// ## 使用
///
/// 在 `load_state` 函数中，`${path}` 会被替换为实际的工作树路径。
static PROMPT_INITIALIZE: &str = include_str!("template/initialize.txt");

/// 审查命令模板内容
///
/// 从 `template/review.txt` 文件加载的审查命令模板，
/// 包含审查代码变更的完整提示词。
///
/// ## 模板变量
///
/// - `${path}`：项目工作树的根路径
///
/// ## 使用
///
/// 在 `load_state` 函数中，`${path}` 会被替换为实际的工作树路径。
static PROMPT_REVIEW: &str = include_str!("template/review.txt");

/// 从命令模板中提取占位符提示
///
/// 解析模板字符串，提取所有 `$N` 格式的占位符（如 `$1`、`$2`），
/// 并检查是否包含特殊占位符 `$ARGUMENTS`。
///
/// ## 参数
///
/// - `template`：命令模板字符串，可能包含占位符
///
/// ## 返回值
///
/// 返回占位符字符串列表，已去重并按字母顺序排序。
/// 如果模板包含 `$ARGUMENTS`，会在列表末尾添加。
///
/// ## 占位符格式
///
/// - 数字占位符：`$1`、`$2`、`$10` 等（正则匹配 `\$\d+`）
/// - 特殊占位符：`$ARGUMENTS`（表示用户输入的原始参数）
///
/// ## 示例
///
/// ```rust,ignore
/// let hints = hints("Review $1 with $2 and $1");
/// // 返回: ["$1", "$2"]
///
/// let hints = hints("Process $ARGUMENTS");
/// // 返回: ["$ARGUMENTS"]
///
/// let hints = hints("Hello $1, use $ARGUMENTS");
/// // 返回: ["$1", "$ARGUMENTS"]
/// ```
///
/// ## 实现细节
///
/// 1. 使用静态正则表达式匹配所有 `$N` 格式的占位符
/// 2. 使用 `HashSet` 去重
/// 3. 对结果排序以保持稳定的输出顺序
/// 4. 单独检查 `$ARGUMENTS` 并追加到末尾
pub fn hints(template: &str) -> Vec<String> {
    // 静态正则表达式，用于匹配 $N 格式的占位符（如 $1, $2, $10）
    // 使用 Lazy 延迟初始化，只在首次使用时编译正则表达式
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\$\d+").unwrap());

    // 存储提取的占位符
    let mut out: Vec<String> = Vec::new();
    // 用于去重的集合
    let mut seen: HashSet<String> = HashSet::new();

    // 遍历模板中所有匹配的占位符
    for m in RE.find_iter(template) {
        let s = m.as_str().to_string();
        // 只有首次出现时才添加到输出列表
        if seen.insert(s.clone()) {
            out.push(s);
        }
    }

    // 对占位符排序，保证输出顺序稳定
    out.sort();

    // 检查是否包含特殊占位符 $ARGUMENTS
    // 该占位符表示用户输入的原始参数，需要单独处理
    if template.contains("$ARGUMENTS") {
        out.push("$ARGUMENTS".to_string());
    }

    out
}

/// 获取命令系统状态的实例化函数
///
/// 返回一个闭包，该闭包被调用时会异步加载并返回命令系统的状态。
/// 状态通过项目实例机制管理，支持按项目隔离。
///
/// ## 返回值
///
/// 返回一个闭包，该闭包：
/// - 无参数
/// - 返回 `BoxFuture<Arc<State>>`
/// - 是 `Send + Sync + 'static` 的，可以跨线程共享
///
/// ## 实现细节
///
/// - 使用 `instance::state` 函数创建状态访问器
/// - 模块名称为 `"command"`
/// - 状态通过 `load_state` 异步加载
/// - 未提供状态变更回调（传入 `None`）
///
/// ## 生命周期
///
/// 状态在首次访问时加载，之后会被缓存。
/// 不同的项目实例会维护各自独立的命令状态。
fn instance_state()
-> impl Fn() -> crate::app::agent::project::BoxFuture<Arc<State>> + Send + Sync + 'static {
    instance::state(
        "command",                                                           // 模块标识符
        || async { load_state().await },                                     // 状态加载函数
        None::<fn(Arc<State>) -> crate::app::agent::project::BoxFuture<()>>, // 无状态变更回调
    )
}

/// 按名称查询单个命令
///
/// 从命令注册表中查询指定名称的命令信息。
///
/// ## 参数
///
/// - `name`：命令名称（如 `"init"`、`"review"` 或技能名称）
///
/// ## 返回值
///
/// - `Some(Info)`：找到命令，返回命令信息的克隆
/// - `None`：命令不存在
///
/// ## 异步说明
///
/// 该函数是异步的，因为命令状态可能需要从磁盘或配置中加载。
///
/// ## 示例
///
/// ```rust,ignore
/// // 查询内置命令
/// if let Some(cmd) = get("init").await {
///     println!("描述: {:?}", cmd.description);
/// }
///
/// // 查询不存在的命令
/// assert!(get("nonexistent").await.is_none());
/// ```
///
/// ## 性能
///
/// 命令状态会被缓存，重复查询不会触发重新加载。
pub async fn get(name: &str) -> Option<Info> {
    instance_state()().await.commands.get(name).cloned()
}

/// 获取所有可用命令列表
///
/// 返回命令注册表中所有命令的信息列表。
///
/// ## 返回值
///
/// 返回 `Vec<Info>`，包含所有注册命令的克隆。
/// 列表顺序取决于内部 `HashMap` 的迭代顺序，不保证特定顺序。
///
/// ## 异步说明
///
/// 该函数是异步的，因为命令状态可能需要从磁盘或配置中加载。
///
/// ## 示例
///
/// ```rust,ignore
/// let commands = list().await;
/// println!("共有 {} 个命令", commands.len());
/// for cmd in commands {
///     println!("- {}: {:?}", cmd.name, cmd.description);
/// }
/// ```
///
/// ## 性能
///
/// - 时间复杂度：O(n)，n 为命令数量
/// - 命令状态会被缓存，重复调用不会触发重新加载
pub async fn list() -> Vec<Info> {
    instance_state()().await.commands.values().cloned().collect()
}

/// 发布命令执行事件
///
/// 将命令执行事件发布到事件总线，供订阅者处理。
///
/// ## 参数
///
/// - `evt`：命令执行事件数据
///
/// ## 返回值
///
/// - `Ok(())`：事件发布成功
/// - `Err(serde_json::Error)`：事件序列化失败
///
/// ## 事件类型
///
/// 事件类型为 `event::EXECUTED`（`"command.executed"`）。
///
/// ## 事件目录
///
/// 事件会被发布到当前项目实例的目录中。
///
/// ## 示例
///
/// ```rust,ignore
/// let event = ExecutedEvent {
///     name: "init".to_string(),
///     session_id: "session-123".to_string(),
///     arguments: "".to_string(),
///     message_id: "msg-456".to_string(),
/// };
/// publish_executed(event)?;
/// ```
///
/// ## 错误处理
///
/// 如果事件无法序列化为 JSON，会返回错误。
/// 调用者应适当处理此错误（如记录日志）。
pub fn publish_executed(evt: ExecutedEvent) -> Result<(), serde_json::Error> {
    bus::publish(event::EXECUTED, evt, Some(instance::directory()))
}

/// 异步加载命令系统状态
///
/// 从配置和技能系统中加载所有命令，构建完整的命令注册表。
///
/// ## 加载顺序
///
/// 1. 加载配置（虽然当前未使用，但保留扩展能力）
/// 2. 注册内置的 `init` 命令
/// 3. 注册内置的 `review` 命令
/// 4. 加载所有技能并转换为命令
///
/// ## 命令优先级
///
/// 如果技能名称与现有命令冲突，该技能会被跳过（优先保留已注册的命令）。
/// 这确保了内置命令不会被技能意外覆盖。
///
/// ## 内置命令
///
/// ### init 命令
/// - 描述：创建/更新 AGENTS.md
/// - 来源：Command
/// - 模板：使用 `PROMPT_INITIALIZE` 并替换 `${path}`
/// - 非子任务
///
/// ### review 命令
/// - 描述：审查变更 [commit|branch|pr]，默认审查未提交变更
/// - 来源：Command
/// - 模板：使用 `PROMPT_REVIEW` 并替换 `${path}`
/// - 作为子任务执行
///
/// ## 技能命令
///
/// 每个技能会被转换为命令：
/// - 名称：技能名称
/// - 描述：技能描述
/// - 来源：Skill
/// - 模板：技能内容
/// - 无提示（hints 为空）
///
/// ## 返回值
///
/// 返回初始化完成的 `State`，包含所有加载的命令。
///
/// ## 性能
///
/// 该函数在首次访问命令状态时被调用一次，
/// 之后状态会被缓存，不会重复加载。
async fn load_state() -> State {
    // 加载配置（当前未使用，但保留以备未来扩展）
    let _cfg = config::get().await;

    // 获取当前项目的工作树路径，用于替换模板中的路径变量
    let worktree = instance::worktree();

    // 创建命令映射表
    let mut commands: HashMap<String, Info> = HashMap::new();

    // 注册 init 命令
    // 该命令用于初始化或更新 AGENTS.md 文件
    commands.insert(
        r#default::INIT.to_string(),
        Info {
            name: r#default::INIT.to_string(),
            description: Some("Create/update AGENTS.md".to_string()),
            agent: None,
            model: None,
            source: Some(Source::Command),
            // 将模板中的 ${path} 替换为实际的工作树路径
            template: PROMPT_INITIALIZE.replace("${path}", &worktree),
            subtask: None,
            // 提取模板中的占位符提示
            hints: hints(PROMPT_INITIALIZE),
        },
    );

    // 注册 review 命令
    // 该命令用于审查代码变更，支持 commit/branch/pr 模式
    commands.insert(
        r#default::REVIEW.to_string(),
        Info {
            name: r#default::REVIEW.to_string(),
            description: Some(
                "Review changes [commit|branch|pr], defaults to uncommitted".to_string(),
            ),
            agent: None,
            model: None,
            source: Some(Source::Command),
            // 将模板中的 ${path} 替换为实际的工作树路径
            template: PROMPT_REVIEW.replace("${path}", &worktree),
            // review 命令作为子任务执行，以支持更复杂的审查流程
            subtask: Some(true),
            // 提取模板中的占位符提示
            hints: hints(PROMPT_REVIEW),
        },
    );

    // 加载所有技能并转换为命令
    // 技能系统提供了可扩展的命令机制
    for s in skill::all().await {
        // 如果命令名称已存在，跳过该技能（保留现有命令）
        // 这确保了内置命令不会被技能意外覆盖
        if commands.contains_key(&s.name) {
            continue;
        }

        // 将技能注册为命令
        commands.insert(
            s.name.clone(),
            Info {
                name: s.name,
                description: Some(s.description),
                agent: None,
                model: None,
                source: Some(Source::Skill),
                // 技能内容直接作为模板使用
                template: s.content,
                subtask: None,
                // 技能命令目前不提供占位符提示
                hints: Vec::new(),
            },
        );
    }

    // 构建并返回最终的状态
    State { commands }
}
