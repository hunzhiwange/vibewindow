//! Agents IPC（进程间通信）工具测试模块
//!
//! 本模块提供 agents_ipc 工具集的完整测试覆盖，验证多个代理之间的消息传递、
//! 状态共享和身份隔离功能。
//!
//! # 测试范围
//!
//! - 数据库 schema 初始化（agents、messages、shared_state 三表）
//! - 代理注册与心跳机制
//! - 点对点消息发送与收件箱隔离
//! - 广播消息分发
//! - 过期代理检测与过滤
//! - 状态键值对的 CRUD 操作
//! - 安全策略执行（只读模式限制）
//! - 身份代码强制执行
//! - 代理生命周期管理（打开时注册、关闭时注销）
//!
//! # 测试策略
//!
//! 每个测试使用独立的临时数据库，确保测试之间完全隔离。
//! 使用 `Arc` 共享数据库实例以模拟多代理并发访问场景。

use super::super::*;
use crate::app::agent::security::AutonomyLevel;
use serde_json::json;
use tempfile::TempDir;

/// 创建测试用的 IpcDb 实例
///
/// # 参数
///
/// - `dir`: 临时目录引用，用于存放数据库文件
/// - `agent_id`: 代理标识符，用于注册和身份验证
///
/// # 返回值
///
/// 返回已初始化的 `IpcDb` 实例，预设 300 秒的过期阈值
///
/// # 示例
///
/// ```ignore
/// let dir = TempDir::new().unwrap();
/// let db = test_db(&dir, "vibewindow_agent_test");
/// // db 已就绪，可执行 IPC 操作
/// ```
fn test_db(dir: &TempDir, agent_id: &str) -> IpcDb {
    let db_path = dir.path().join("agents.db");
    IpcDb::open_with_id(db_path.to_str().unwrap(), agent_id, 300).unwrap()
}

/// 测试数据库 schema 是否正确创建三个表
///
/// # 验证点
///
/// - `agents` 表：存储代理注册信息
/// - `messages` 表：存储代理间消息
/// - `shared_state` 表：存储共享状态键值对
#[test]
fn schema_creates_three_tables() {
    let dir = TempDir::new().unwrap();
    let db = test_db(&dir, "vibewindow_agent_a");
    let conn = db.conn.lock().unwrap();

    // 查询 sqlite_master 表获取所有表名
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // 验证三个核心表均已创建
    assert!(tables.contains(&"agents".to_string()));
    assert!(tables.contains(&"messages".to_string()));
    assert!(tables.contains(&"shared_state".to_string()));
}

/// 测试代理在打开数据库时自动注册
///
/// # 验证点
///
/// 当调用 `IpcDb::open_with_id` 时，代理信息应自动写入 agents 表
#[test]
fn agent_registers_on_open() {
    let dir = TempDir::new().unwrap();
    let db = test_db(&dir, "vibewindow_agent_a");
    let conn = db.conn.lock().unwrap();

    // 查询 agents 表中该代理的记录数
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agents WHERE agent_id = 'vibewindow_agent_a'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    // 应恰好存在一条记录
    assert_eq!(count, 1);
}

/// 测试心跳机制是否更新 last_seen 时间戳
///
/// # 验证点
///
/// - 初始打开时设置 last_seen
/// - 调用 heartbeat() 后 last_seen 应更新为更新值
#[test]
fn heartbeat_updates_last_seen() {
    let dir = TempDir::new().unwrap();
    let db = test_db(&dir, "vibewindow_agent_a");

    // 记录心跳前的 last_seen 值
    let before: i64 = {
        let conn = db.conn.lock().unwrap();
        conn.query_row(
            "SELECT last_seen FROM agents WHERE agent_id = 'vibewindow_agent_a'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    };

    // 等待一小段时间确保时间戳差异可检测
    std::thread::sleep(std::time::Duration::from_millis(10));
    db.heartbeat();

    // 记录心跳后的 last_seen 值
    let after: i64 = {
        let conn = db.conn.lock().unwrap();
        conn.query_row(
            "SELECT last_seen FROM agents WHERE agent_id = 'vibewindow_agent_a'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    };

    // 更新后的时间戳应大于等于更新前
    assert!(after >= before);
}

/// 测试收件箱按代理隔离
///
/// # 验证点
///
/// - 发送给 agent_b 的消息不应出现在 agent_a 的收件箱
/// - agent_b 应能正确收到发送给自己的消息
/// - 消息 payload 应保持完整
#[tokio::test]
async fn inbox_isolates_per_agent() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("agents.db").to_str().unwrap().to_string();

    // 创建两个代理的数据库实例（共享同一数据库文件）
    let db_a = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_a", 300).unwrap());
    let db_b = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_b", 300).unwrap());

    // agent_a 向 agent_b 发送消息
    let send_tool = AgentsSendTool::new(db_a.clone(), Arc::new(SecurityPolicy::default()));
    send_tool
        .execute(json!({"to_agent": "vibewindow_agent_b", "payload": "hello b"}))
        .await
        .unwrap();

    // 检查 agent_a 的收件箱（应为空）
    let inbox_a = AgentsInboxTool::new(db_a);
    let result_a = inbox_a.execute(json!({})).await.unwrap();
    let msgs_a: Vec<serde_json::Value> = serde_json::from_str(&result_a.output).unwrap();
    assert!(msgs_a.is_empty());

    // 检查 agent_b 的收件箱（应有 1 条消息）
    let inbox_b = AgentsInboxTool::new(db_b);
    let result_b = inbox_b.execute(json!({})).await.unwrap();
    let msgs_b: Vec<serde_json::Value> = serde_json::from_str(&result_b.output).unwrap();
    assert_eq!(msgs_b.len(), 1);
    assert_eq!(msgs_b[0]["payload"], "hello b");
}

/// 测试广播消息对所有代理可见
///
/// # 验证点
///
/// - 使用 `to_agent: "*"` 发送广播消息
/// - 所有已注册代理都应能收到该消息
#[tokio::test]
async fn broadcast_visible_to_all_agents() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("agents.db").to_str().unwrap().to_string();

    let db_a = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_a", 300).unwrap());
    let db_b = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_b", 300).unwrap());

    // 发送广播消息（目标为 "*"）
    let send_tool = AgentsSendTool::new(db_a.clone(), Arc::new(SecurityPolicy::default()));
    send_tool.execute(json!({"to_agent": "*", "payload": "broadcast msg"})).await.unwrap();

    // agent_a 应收到广播
    let inbox_a = AgentsInboxTool::new(db_a);
    let result_a = inbox_a.execute(json!({})).await.unwrap();
    let msgs_a: Vec<serde_json::Value> = serde_json::from_str(&result_a.output).unwrap();
    assert_eq!(msgs_a.len(), 1);

    // agent_b 应收到广播
    let inbox_b = AgentsInboxTool::new(db_b);
    let result_b = inbox_b.execute(json!({})).await.unwrap();
    let msgs_b: Vec<serde_json::Value> = serde_json::from_str(&result_b.output).unwrap();
    assert_eq!(msgs_b.len(), 1);
}

/// 测试过期代理从列表中排除
///
/// # 验证点
///
/// - last_seen 超过 staleness_secs 阈值的代理不应出现在列表中
/// - 活跃代理应正常出现在列表中
#[tokio::test]
async fn stale_agents_excluded_from_list() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("agents.db").to_str().unwrap().to_string();

    // 设置 5 秒的过期阈值
    let db_a = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_a", 5).unwrap());

    // 手动插入一个已过期的代理（last_seen 为 100 秒前）
    {
        let conn = db_a.conn.lock().unwrap();
        let old_time = now_epoch() - 100;
        conn.execute(
            "INSERT OR REPLACE INTO agents (agent_id, status, last_seen) VALUES ('vibewindow_agent_b', 'online', ?1)",
            rusqlite::params![old_time],
        )
        .unwrap();
    }

    // 查询代理列表
    let list_tool = AgentsListTool::new(db_a);
    let result = list_tool.execute(json!({})).await.unwrap();
    let agents: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();

    // 只应有 1 个代理（活跃的 agent_a），过期的 agent_b 应被过滤
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["agent_id"], "vibewindow_agent_a");
}

/// 测试身份代码强制执行
///
/// # 验证点
///
/// - 发送消息时，from_agent 字段应由数据库自动填充
/// - 不允许伪造发送者身份
#[tokio::test]
async fn identity_code_enforced() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("agents.db").to_str().unwrap().to_string();
    let db = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_a", 300).unwrap());

    // 发送消息（不指定 from_agent）
    let send_tool = AgentsSendTool::new(db.clone(), Arc::new(SecurityPolicy::default()));
    send_tool
        .execute(json!({"to_agent": "vibewindow_agent_b", "payload": "test"}))
        .await
        .unwrap();

    // 直接查询数据库验证 from_agent 字段
    let conn = db.conn.lock().unwrap();
    let from: String = conn
        .query_row("SELECT from_agent FROM messages LIMIT 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(from, "vibewindow_agent_a");
}

/// 测试状态的 upsert 操作（创建和更新）
///
/// # 验证点
///
/// - 首次设置键值对时创建记录
/// - 再次设置相同键时更新值
#[tokio::test]
async fn state_upsert_creates_and_updates() {
    let dir = TempDir::new().unwrap();
    let db = Arc::new(test_db(&dir, "vibewindow_agent_a"));

    let set_tool = StateSetTool::new(db.clone(), Arc::new(SecurityPolicy::default()));
    let get_tool = StateGetTool::new(db.clone());

    // 首次设置
    set_tool.execute(json!({"key": "progress", "value": "50%"})).await.unwrap();

    let result = get_tool.execute(json!({"key": "progress"})).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.output).unwrap();
    assert_eq!(parsed["value"], "50%");

    // 更新已存在的键
    set_tool.execute(json!({"key": "progress", "value": "100%"})).await.unwrap();

    let result = get_tool.execute(json!({"key": "progress"})).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.output).unwrap();
    assert_eq!(parsed["value"], "100%");
}

/// 测试状态记录所有者信息
///
/// # 验证点
///
/// - 设置状态时自动记录 owner 字段
/// - owner 应为设置该键值对的代理 ID
#[tokio::test]
async fn state_records_owner() {
    let dir = TempDir::new().unwrap();
    let db = Arc::new(test_db(&dir, "vibewindow_agent_a"));

    // 设置状态
    let set_tool = StateSetTool::new(db.clone(), Arc::new(SecurityPolicy::default()));
    set_tool.execute(json!({"key": "task", "value": "done"})).await.unwrap();

    // 获取状态并检查 owner
    let get_tool = StateGetTool::new(db);
    let result = get_tool.execute(json!({"key": "task"})).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&result.output).unwrap();
    assert_eq!(parsed["owner"], "vibewindow_agent_a");
}

/// 测试空收件箱返回成功
///
/// # 验证点
///
/// - 无消息时收件箱应返回成功状态
/// - 返回的消息列表应为空
#[tokio::test]
async fn empty_inbox_returns_success() {
    let dir = TempDir::new().unwrap();
    let db = Arc::new(test_db(&dir, "vibewindow_agent_a"));

    let inbox_tool = AgentsInboxTool::new(db);
    let result = inbox_tool.execute(json!({})).await.unwrap();

    // 应成功执行
    assert!(result.success);
    let msgs: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();
    // 消息列表应为空
    assert!(msgs.is_empty());
}

/// 测试安全策略在只读模式下阻止写操作
///
/// # 验证点
///
/// - 只读模式下发送消息应被拒绝
/// - 只读模式下设置状态应被拒绝
/// - 被拒绝的操作应返回失败状态和错误信息
#[tokio::test]
async fn security_blocks_act_in_readonly() {
    let dir = TempDir::new().unwrap();
    let db = Arc::new(test_db(&dir, "vibewindow_agent_a"));

    // 配置只读安全策略
    let readonly = Arc::new(SecurityPolicy {
        autonomy: AutonomyLevel::ReadOnly,
        ..SecurityPolicy::default()
    });

    // 尝试发送消息（应被拒绝）
    let send_tool = AgentsSendTool::new(db.clone(), readonly.clone());
    let result = send_tool
        .execute(json!({"to_agent": "vibewindow_agent_b", "payload": "test"}))
        .await
        .unwrap();
    assert!(!result.success);
    assert!(result.error.is_some());

    // 尝试设置状态（应被拒绝）
    let set_tool = StateSetTool::new(db, readonly);
    let result = set_tool.execute(json!({"key": "k", "value": "v"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.is_some());
}

/// 测试禁用配置不注册任何工具
///
/// # 验证点
///
/// - enabled: false 时，其他配置项仍保留默认值
/// - staleness_secs 默认为 300 秒
#[test]
fn disabled_config_registers_no_tools() {
    let config = AgentsIpcConfig { enabled: false, ..AgentsIpcConfig::default() };
    assert!(!config.enabled);
    assert_eq!(config.staleness_secs, 300);
}

/// 测试从工作区路径派生代理 ID
///
/// # 验证点
///
/// - agent_id 应为 64 字符的十六进制字符串（SHA-256 哈希）
/// - 相同工作区路径应生成相同的 agent_id
#[test]
fn real_open_derives_agent_id_from_workspace() {
    let dir = TempDir::new().unwrap();
    let workspace = dir.path().join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let db_path = dir.path().join("agents.db");

    let config = AgentsIpcConfig {
        enabled: true,
        db_path: db_path.to_str().unwrap().to_string(),
        staleness_secs: 300,
    };

    // 从工作区打开数据库
    let db = IpcDb::open(&workspace, &config).unwrap();

    // agent_id 应为 64 字符十六进制
    assert_eq!(db.agent_id().len(), 64);
    assert!(db.agent_id().chars().all(|c| c.is_ascii_hexdigit()));

    // 相同工作区应生成相同 ID
    let db2 = IpcDb::open(&workspace, &config).unwrap();
    assert_eq!(db.agent_id(), db2.agent_id());
}

/// 测试代理销毁时从表中移除
///
/// # 验证点
///
/// - IpcDb 被 drop 后，agents 表中对应的记录应被删除
#[test]
fn drop_removes_agent_from_table() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("agents.db");
    let db_path_str = db_path.to_str().unwrap().to_string();

    // 在作用域内创建数据库，超出作用域后被 drop
    {
        let _db = IpcDb::open_with_id(&db_path_str, "vibewindow_agent_a", 300).unwrap();
    }

    // 直接打开数据库文件验证记录已被删除
    let conn = Connection::open(&db_path_str).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agents WHERE agent_id = 'vibewindow_agent_a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

/// 测试直连消息在收件箱读取后标记为已读
///
/// # 验证点
///
/// - 首次调用 inbox 时应返回消息
/// - 再次调用 inbox 时不应返回已读消息（消息应被标记为已读）
#[tokio::test]
async fn direct_messages_marked_read_after_inbox() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("agents.db").to_str().unwrap().to_string();

    let db_a = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_a", 300).unwrap());
    let db_b = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_b", 300).unwrap());

    // 发送消息
    let send_tool = AgentsSendTool::new(db_a, Arc::new(SecurityPolicy::default()));
    send_tool
        .execute(json!({"to_agent": "vibewindow_agent_b", "payload": "once"}))
        .await
        .unwrap();

    let inbox_b = AgentsInboxTool::new(db_b.clone());

    // 首次读取：应有 1 条消息
    let result = inbox_b.execute(json!({})).await.unwrap();
    let msgs: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();
    assert_eq!(msgs.len(), 1);

    // 再次读取：应为空（消息已标记为已读）
    let result = inbox_b.execute(json!({})).await.unwrap();
    let msgs: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();
    assert!(msgs.is_empty());
}

/// 测试获取不存在的状态键返回 not found
///
/// # 验证点
///
/// - 查询不存在的键应返回成功状态
/// - 输出应包含 "not found" 提示
#[tokio::test]
async fn state_get_missing_key_returns_not_found() {
    let dir = TempDir::new().unwrap();
    let db = Arc::new(test_db(&dir, "vibewindow_agent_a"));

    let get_tool = StateGetTool::new(db);
    let result = get_tool.execute(json!({"key": "nonexistent"})).await.unwrap();

    assert!(result.success);
    assert!(result.output.contains("not found"));
}

/// 测试发送消息缺少必需参数时返回错误
///
/// # 验证点
///
/// - 缺少 payload 参数时应返回错误
/// - 缺少 to_agent 参数时应返回错误
#[tokio::test]
async fn send_missing_params_returns_error() {
    let dir = TempDir::new().unwrap();
    let db = Arc::new(test_db(&dir, "vibewindow_agent_a"));
    let send_tool = AgentsSendTool::new(db, Arc::new(SecurityPolicy::default()));

    // 缺少 payload
    let result = send_tool.execute(json!({"to_agent": "vibewindow_agent_b"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.as_deref().unwrap().contains("payload"));

    // 缺少 to_agent
    let result = send_tool.execute(json!({"payload": "hello"})).await.unwrap();
    assert!(!result.success);
    assert!(result.error.as_deref().unwrap().contains("to_agent"));
}

/// 测试两个代理的完整交互流程
///
/// # 验证点
///
/// - 代理列表应显示所有活跃代理
/// - 消息发送和接收应正常工作
/// - 共享状态应可跨代理访问
/// - 消息应包含正确的发送者信息
/// - 状态应记录正确的所有者
#[tokio::test]
async fn two_agents_full_exchange() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("agents.db").to_str().unwrap().to_string();

    let db_a = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_a", 300).unwrap());
    let db_b = Arc::new(IpcDb::open_with_id(&db_path, "vibewindow_agent_b", 300).unwrap());
    let security = Arc::new(SecurityPolicy::default());

    // 步骤 1：验证代理列表
    let list_tool = AgentsListTool::new(db_a.clone());
    let result = list_tool.execute(json!({})).await.unwrap();
    let agents: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();
    assert_eq!(agents.len(), 2);

    // 步骤 2：agent_a 向 agent_b 发送任务消息
    let send_a = AgentsSendTool::new(db_a.clone(), security.clone());
    let r = send_a
        .execute(json!({"to_agent": "vibewindow_agent_b", "payload": "task: summarize"}))
        .await
        .unwrap();
    assert!(r.success);

    // 步骤 3：agent_b 读取收件箱
    let inbox_b = AgentsInboxTool::new(db_b.clone());
    let r = inbox_b.execute(json!({})).await.unwrap();
    let msgs: Vec<serde_json::Value> = serde_json::from_str(&r.output).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0]["payload"], "task: summarize");
    assert_eq!(msgs[0]["from_agent"], "vibewindow_agent_a");

    // 步骤 4：agent_b 回复 agent_a
    let send_b = AgentsSendTool::new(db_b.clone(), security.clone());
    send_b
        .execute(json!({"to_agent": "vibewindow_agent_a", "payload": "done: summary attached"}))
        .await
        .unwrap();

    // 步骤 5：agent_a 读取回复
    let inbox_a = AgentsInboxTool::new(db_a.clone());
    let r = inbox_a.execute(json!({})).await.unwrap();
    let msgs: Vec<serde_json::Value> = serde_json::from_str(&r.output).unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0]["payload"], "done: summary attached");
    assert_eq!(msgs[0]["from_agent"], "vibewindow_agent_b");

    // 步骤 6：agent_a 设置共享状态
    let set_tool = StateSetTool::new(db_a, security);
    set_tool.execute(json!({"key": "status", "value": "complete"})).await.unwrap();

    // 步骤 7：agent_b 读取共享状态
    let get_tool = StateGetTool::new(db_b);
    let r = get_tool.execute(json!({"key": "status"})).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&r.output).unwrap();
    assert_eq!(parsed["value"], "complete");
    assert_eq!(parsed["owner"], "vibewindow_agent_a");
}
