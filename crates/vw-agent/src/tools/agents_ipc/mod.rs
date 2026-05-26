//! 代理间进程通信工具
//!
//! 提供基于共享 SQLite 数据库的 5 个 LLM 可调用工具，允许同一主机上
//! 独立的 VibeWindow 进程相互发现和交换消息。
//!
//! # 核心功能
//!
//! - **智能体发现**：`AgentsListTool` 列出当前在线的智能体
//! - **消息传递**：`AgentsSendTool` 发送消息，`AgentsInboxTool` 读取消息
//! - **状态共享**：`StateGetTool` 和 `StateSetTool` 管理共享键值存储
//!
//! # 架构说明
//!
//! 所有工具共享同一个 `IpcDb` 实例，通过 WAL 模式的 SQLite 数据库
//! 实现进程间的数据同步。每个智能体通过工作目录的哈希值作为唯一标识。

use super::traits::{Tool, ToolResult};
use crate::app::agent::config::AgentsIpcConfig;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::security::policy::ToolOperation;
use async_trait::async_trait;
use rusqlite::Connection;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// ── IpcDb 核心 ──────────────────────────────────────────────────

/// SQLite 性能优化 PRAGMA 配置语句
///
/// - `journal_mode=WAL`：启用预写日志模式，提高并发性能
/// - `synchronous=NORMAL`：平衡安全性和性能
/// - `busy_timeout=5000`：锁等待超时 5 秒
const PRAGMA_SQL: &str =
    "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;";

/// IPC 数据库表结构定义
///
/// 包含三个核心表：
/// - `agents`：注册的智能体列表，记录 ID、角色、状态和最后在线时间
/// - `messages`：智能体间的消息队列，支持点对点和广播消息
/// - `shared_state`：共享的键值存储，用于状态同步
const SCHEMA_SQL: &str = "CREATE TABLE IF NOT EXISTS agents (
    agent_id  TEXT PRIMARY KEY,
    role      TEXT,
    status    TEXT DEFAULT 'online',
    metadata  TEXT,
    last_seen INTEGER
);
CREATE TABLE IF NOT EXISTS messages (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    from_agent TEXT NOT NULL,
    to_agent   TEXT NOT NULL,
    payload    TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    read       INTEGER DEFAULT 0
);
CREATE TABLE IF NOT EXISTS shared_state (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    owner      TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);";

/// IPC 工具共享的 SQLite 数据库句柄
///
/// 每个 VibeWindow 进程持有一个实例，通过 `Arc<Mutex<Connection>>` 实现
/// 线程安全的数据库访问。智能体 ID 由工作目录路径的 SHA256 哈希生成。
pub(crate) struct IpcDb {
    /// SQLite 数据库连接（线程安全包装）
    conn: Arc<Mutex<Connection>>,
    /// 当前智能体的唯一标识符（工作目录路径哈希）
    agent_id: String,
    /// 智能体过期时间（秒），超过此时间未心跳的智能体视为离线
    staleness_secs: u64,
}

/// 获取当前 Unix 时间戳（秒）
///
/// # 返回值
///
/// 返回自 Unix 纪元以来的秒数，如果系统时间早于纪元则返回 0
fn now_epoch() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

impl IpcDb {
    fn team_key(team_id: &str) -> String {
        format!("team:{team_id}")
    }

    /// 初始化数据库连接
    ///
    /// 执行以下操作：
    /// 1. 设置 SQLite PRAGMA 参数优化性能
    /// 2. 创建必要的表结构（如果不存在）
    /// 3. 在 agents 表中注册或更新当前智能体
    ///
    /// # 参数
    ///
    /// - `conn`：SQLite 数据库连接
    /// - `agent_id`：智能体的唯一标识符
    /// - `staleness_secs`：智能体过期时间（秒）
    ///
    /// # 返回值
    ///
    /// 成功返回初始化后的 `IpcDb` 实例，失败返回错误信息
    ///
    /// # 错误
    ///
    /// - 设置 PRAGMA 失败
    /// - 创建表结构失败
    /// - 注册智能体失败
    fn init(conn: Connection, agent_id: String, staleness_secs: u64) -> Result<Self, String> {
        conn.execute_batch(PRAGMA_SQL).map_err(|e| format!("failed to set pragmas: {e}"))?;
        conn.execute_batch(SCHEMA_SQL).map_err(|e| format!("failed to create schema: {e}"))?;

        let now = now_epoch();
        // 使用 UPDATE + INSERT 模式：先尝试更新已存在的智能体记录，
        // 如果不存在（updated == 0）则插入新记录，保留原有的 role 和 metadata 字段
        let updated = conn
            .execute(
                "UPDATE agents SET status = 'online', last_seen = ?2 WHERE agent_id = ?1",
                rusqlite::params![agent_id, now],
            )
            .map_err(|e| format!("failed to update agent: {e}"))?;
        if updated == 0 {
            conn.execute(
                "INSERT INTO agents (agent_id, status, last_seen) VALUES (?1, 'online', ?2)",
                rusqlite::params![agent_id, now],
            )
            .map_err(|e| format!("failed to register agent: {e}"))?;
        }

        Ok(Self { conn: Arc::new(Mutex::new(conn)), agent_id, staleness_secs })
    }

    /// 打开（或创建）共享 IPC 数据库并注册当前智能体
    ///
    /// 数据库路径支持 `~` 展开。智能体 ID 由工作目录的规范路径
    /// 通过 SHA256 哈希生成，确保同一工作目录的进程共享相同的智能体身份。
    ///
    /// # 参数
    ///
    /// - `workspace_dir`：工作目录路径，用于生成稳定的智能体 ID
    /// - `config`：IPC 配置，包含数据库路径和过期时间
    ///
    /// # 返回值
    ///
    /// 成功返回初始化后的 `IpcDb` 实例，失败返回错误信息
    ///
    /// # 错误
    ///
    /// - 创建数据库目录失败
    /// - 打开数据库文件失败
    /// - 初始化数据库失败
    pub fn open(workspace_dir: &std::path::Path, config: &AgentsIpcConfig) -> Result<Self, String> {
        let db_path = shellexpand::tilde(&config.db_path).into_owned();

        // 确保数据库文件的父目录存在
        if let Some(parent) = std::path::Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create db directory: {e}"))?;
        }

        let conn =
            Connection::open(&db_path).map_err(|e| format!("failed to open IPC database: {e}"))?;

        // 从工作目录的规范路径生成智能体 ID
        // 使用规范路径（canonicalize）确保符号链接等被解析为唯一路径
        let canonical =
            workspace_dir.canonicalize().unwrap_or_else(|_| workspace_dir.to_path_buf());
        let hash = Sha256::digest(canonical.to_string_lossy().as_bytes());
        let agent_id = format!("{hash:x}");

        Self::init(conn, agent_id, config.staleness_secs)
    }

    /// 更新智能体的最后在线时间戳
    ///
    /// 每次工具调用时附带执行心跳，保持智能体在线状态。
    /// 这是轻量级操作，不会影响工具调用的性能。
    pub fn heartbeat(&self) {
        let now = now_epoch();
        if let Ok(conn) = self.conn.lock() {
            let _ = conn.execute(
                "UPDATE agents SET last_seen = ?1 WHERE agent_id = ?2",
                rusqlite::params![now, self.agent_id],
            );
        }
    }

    /// 获取当前智能体的唯一标识符
    ///
    /// # 返回值
    ///
    /// 返回智能体 ID 字符串的引用
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// 使用指定的智能体 ID 打开数据库（仅用于测试）
    ///
    /// # 参数
    ///
    /// - `db_path`：数据库文件路径
    /// - `agent_id`：指定的智能体 ID
    /// - `staleness_secs`：智能体过期时间（秒）
    ///
    /// # 返回值
    ///
    /// 成功返回初始化后的 `IpcDb` 实例，失败返回错误信息
    #[cfg(test)]
    fn open_with_id(db_path: &str, agent_id: &str, staleness_secs: u64) -> Result<Self, String> {
        let conn =
            Connection::open(db_path).map_err(|e| format!("failed to open IPC database: {e}"))?;
        Self::init(conn, agent_id.to_string(), staleness_secs)
    }

    pub(crate) fn create_team(
        &self,
        team_id: &str,
        members: &[String],
    ) -> Result<serde_json::Value, String> {
        let team_id = team_id.trim();
        if team_id.is_empty() {
            return Err("team id must not be empty".to_string());
        }

        let mut deduped = members
            .iter()
            .map(|member| member.trim())
            .filter(|member| !member.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        deduped.sort();
        deduped.dedup();
        if deduped.is_empty() {
            return Err("team members must not be empty".to_string());
        }

        let value = serde_json::to_string(&deduped)
            .map_err(|error| format!("failed to encode team members: {error}"))?;
        let now = now_epoch();
        let key = Self::team_key(team_id);
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO shared_state (key, value, owner, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![key, value, self.agent_id, now],
        )
        .map_err(|error| format!("failed to persist team: {error}"))?;
        Ok(json!({
            "team_id": team_id,
            "members": deduped,
            "updated_at": now,
            "owner": self.agent_id,
        }))
    }

    pub(crate) fn delete_team(&self, team_id: &str) -> Result<bool, String> {
        let team_id = team_id.trim();
        if team_id.is_empty() {
            return Err("team id must not be empty".to_string());
        }

        let key = Self::team_key(team_id);
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let affected = conn
            .execute("DELETE FROM shared_state WHERE key = ?1", rusqlite::params![key])
            .map_err(|error| format!("failed to delete team: {error}"))?;
        Ok(affected > 0)
    }

    pub(crate) fn read_team_members(&self, team_id: &str) -> Result<Vec<String>, String> {
        let team_id = team_id.trim();
        if team_id.is_empty() {
            return Err("team id must not be empty".to_string());
        }

        let key = Self::team_key(team_id);
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let value: String = conn
            .query_row(
                "SELECT value FROM shared_state WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .map_err(|_| format!("unknown team '{team_id}'"))?;
        let members: Vec<String> = serde_json::from_str(&value)
            .map_err(|error| format!("failed to decode team members: {error}"))?;
        Ok(members)
    }

    pub(crate) fn send_message(&self, to_agent: &str, payload: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO messages (from_agent, to_agent, payload, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![self.agent_id, to_agent, payload, now_epoch()],
        )
        .map_err(|error| format!("failed to send message: {error}"))?;
        Ok(())
    }
}

impl Drop for IpcDb {
    /// 析构时清理智能体注册信息
    ///
    /// 从 agents 表中删除当前智能体的记录，通知其他进程该智能体已离线。
    /// 这是优雅退出的重要组成部分。
    fn drop(&mut self) {
        if let Ok(conn) = self.conn.lock() {
            let _ = conn.execute(
                "DELETE FROM agents WHERE agent_id = ?1",
                rusqlite::params![self.agent_id],
            );
        }
    }
}

// ── AgentsListTool ──────────────────────────────────────────────

/// 列出在线智能体工具
///
/// 返回在过期时间窗口内活跃的智能体列表，包括智能体 ID、角色和最后在线时间。
/// 这是只读操作，不会修改任何数据。
pub struct AgentsListTool {
    /// IPC 数据库句柄
    ipc_db: Arc<IpcDb>,
}

impl AgentsListTool {
    /// 创建新的智能体列表工具实例
    ///
    /// # 参数
    ///
    /// - `ipc_db`：共享的 IPC 数据库句柄
    pub(crate) fn new(ipc_db: Arc<IpcDb>) -> Self {
        Self { ipc_db }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for AgentsListTool {
    /// 返回工具名称
    fn name(&self) -> &str {
        "agents_list"
    }

    /// 返回工具功能描述（供 LLM 理解工具用途）
    fn description(&self) -> &str {
        "列出此主机上在线的 IPC 智能体。返回在过期时间窗口内的智能体 ID、角色和最后在线时间戳。"
    }

    /// 返回工具参数 JSON Schema
    ///
    /// 此工具无需参数
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    /// 执行智能体列表查询
    ///
    /// # 参数
    ///
    /// - `_args`：工具参数（此工具忽略所有参数）
    ///
    /// # 返回值
    ///
    /// 返回包含在线智能体列表的 `ToolResult`，每个智能体包含：
    /// - `agent_id`：智能体唯一标识符
    /// - `role`：智能体角色（可选）
    /// - `status`：智能体状态（默认 "online"）
    /// - `last_seen`：最后在线时间戳
    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        self.ipc_db.heartbeat();

        let conn = self.ipc_db.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        // 计算过期截止时间：当前时间减去过期窗口
        let cutoff = now_epoch() - self.ipc_db.staleness_secs as i64;

        let mut stmt = conn.prepare(
            "SELECT agent_id, role, status, last_seen FROM agents WHERE last_seen >= ?1",
        )?;

        // 查询所有在过期窗口内有心跳的智能体
        let rows: Vec<serde_json::Value> = stmt
            .query_map(rusqlite::params![cutoff], |row| {
                Ok(json!({
                    "agent_id": row.get::<_, String>(0)?,
                    "role": row.get::<_, Option<String>>(1)?,
                    "status": row.get::<_, String>(2)?,
                    "last_seen": row.get::<_, i64>(3)?
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&rows).unwrap_or_default(),
            error: None,
        })
    }
}

// ── AgentsSendTool ──────────────────────────────────────────────

/// 发送消息工具
///
/// 向指定的智能体发送消息，支持点对点发送和广播（`to_agent="*"`）。
/// 发送的消息存储在 messages 表中，等待目标智能体读取。
pub struct AgentsSendTool {
    /// IPC 数据库句柄
    ipc_db: Arc<IpcDb>,
    /// 安全策略，用于权限检查
    security: Arc<SecurityPolicy>,
}

impl AgentsSendTool {
    /// 创建新的消息发送工具实例
    ///
    /// # 参数
    ///
    /// - `ipc_db`：共享的 IPC 数据库句柄
    /// - `security`：安全策略引用
    pub(crate) fn new(ipc_db: Arc<IpcDb>, security: Arc<SecurityPolicy>) -> Self {
        Self { ipc_db, security }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for AgentsSendTool {
    /// 返回工具名称
    fn name(&self) -> &str {
        "agents_send"
    }

    /// 返回工具功能描述（供 LLM 理解工具用途）
    fn description(&self) -> &str {
        "按 ID 向另一个智能体发送消息，或使用 to_agent=\"*\" 广播给所有智能体。"
    }

    /// 返回工具参数 JSON Schema
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "to_agent": {
                    "type": "string",
                    "description": "目标智能体 ID 或 '*' 广播"
                },
                "payload": {
                    "type": "string",
                    "description": "消息内容（建议使用 JSON 字符串）"
                }
            },
            "required": ["to_agent", "payload"]
        })
    }

    /// 执行消息发送
    ///
    /// # 参数
    ///
    /// - `args`：工具参数，必须包含 `to_agent` 和 `payload`
    ///
    /// # 返回值
    ///
    /// 成功返回确认消息，失败返回错误信息
    ///
    /// # 安全检查
    ///
    /// 执行前检查安全策略是否允许 `agents_send` 操作
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 安全策略检查：验证是否有权限执行消息发送操作
        if let Err(error) = self.security.enforce_tool_operation(ToolOperation::Act, "agents_send")
        {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }

        self.ipc_db.heartbeat();

        // 提取并验证目标智能体参数
        let to_agent = match args.get("to_agent").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'to_agent' parameter".into()),
                });
            }
        };

        // 提取并验证消息载荷参数
        let payload = match args.get("payload").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'payload' parameter".into()),
                });
            }
        };

        let conn = self.ipc_db.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let now = now_epoch();
        // 将消息插入 messages 表，等待目标智能体读取
        conn.execute(
            "INSERT INTO messages (from_agent, to_agent, payload, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![self.ipc_db.agent_id, to_agent, payload, now],
        )?;

        Ok(ToolResult { success: true, output: format!("Message sent to {to_agent}"), error: None })
    }
}

// ── AgentsInboxTool ─────────────────────────────────────────────

/// 读取收件箱工具
///
/// 读取发送给当前智能体的未读消息，包括点对点消息和广播消息（`to_agent="*"`）。
/// 点对点消息在读取后自动标记为已读，广播消息保持未读状态以便其他智能体读取。
pub struct AgentsInboxTool {
    /// IPC 数据库句柄
    ipc_db: Arc<IpcDb>,
}

impl AgentsInboxTool {
    /// 创建新的收件箱工具实例
    ///
    /// # 参数
    ///
    /// - `ipc_db`：共享的 IPC 数据库句柄
    pub(crate) fn new(ipc_db: Arc<IpcDb>) -> Self {
        Self { ipc_db }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for AgentsInboxTool {
    /// 返回工具名称
    fn name(&self) -> &str {
        "agents_inbox"
    }

    /// 返回工具功能描述（供 LLM 理解工具用途）
    fn description(&self) -> &str {
        "读取此智能体收件箱中的未读消息（包括广播给 '*' 的消息）。直接消息在检索后标记为已读；广播消息保持未读。"
    }

    /// 返回工具参数 JSON Schema
    ///
    /// 此工具无需参数
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    /// 执行收件箱读取
    ///
    /// # 参数
    ///
    /// - `_args`：工具参数（此工具忽略所有参数）
    ///
    /// # 返回值
    ///
    /// 返回未读消息列表，每条消息包含：
    /// - `id`：消息 ID
    /// - `from_agent`：发送者智能体 ID
    /// - `payload`：消息内容
    /// - `created_at`：消息创建时间戳
    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<ToolResult> {
        self.ipc_db.heartbeat();

        let conn = self.ipc_db.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let agent_id = &self.ipc_db.agent_id;

        // 查询发给当前智能体的点对点消息和广播消息（to_agent = '*'）
        let mut stmt = conn.prepare(
            "SELECT id, from_agent, payload, created_at FROM messages WHERE (to_agent = ?1 OR to_agent = '*') AND read = 0 ORDER BY created_at ASC",
        )?;

        let messages: Vec<serde_json::Value> = stmt
            .query_map(rusqlite::params![agent_id], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "from_agent": row.get::<_, String>(1)?,
                    "payload": row.get::<_, String>(2)?,
                    "created_at": row.get::<_, i64>(3)?
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();

        // 将点对点消息（非广播）标记为已读
        // 广播消息（to_agent = '*'）保持未读，以便其他智能体也能读取
        let _ = conn.execute(
            "UPDATE messages SET read = 1 WHERE to_agent = ?1 AND read = 0",
            rusqlite::params![agent_id],
        );

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&messages).unwrap_or_default(),
            error: None,
        })
    }
}

// ── StateGetTool ────────────────────────────────────────────────

/// 获取共享状态工具
///
/// 从共享的键值存储中获取指定键的值。键值存储是所有智能体共享的，
/// 任何智能体都可以读取其他智能体设置的值。
pub struct StateGetTool {
    /// IPC 数据库句柄
    ipc_db: Arc<IpcDb>,
}

impl StateGetTool {
    /// 创建新的状态获取工具实例
    ///
    /// # 参数
    ///
    /// - `ipc_db`：共享的 IPC 数据库句柄
    pub(crate) fn new(ipc_db: Arc<IpcDb>) -> Self {
        Self { ipc_db }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for StateGetTool {
    /// 返回工具名称
    fn name(&self) -> &str {
        "state_get"
    }

    /// 返回工具功能描述（供 LLM 理解工具用途）
    fn description(&self) -> &str {
        "从共享的智能体间键值存储中获取值。"
    }

    /// 返回工具参数 JSON Schema
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "要查找的键"
                }
            },
            "required": ["key"]
        })
    }

    /// 执行状态获取
    ///
    /// # 参数
    ///
    /// - `args`：工具参数，必须包含 `key`
    ///
    /// # 返回值
    ///
    /// 成功返回键值对信息（包括值、所有者和更新时间），键不存在时返回提示信息
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        self.ipc_db.heartbeat();

        // 提取并验证键参数
        let key = match args.get("key").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'key' parameter".into()),
                });
            }
        };

        let conn = self.ipc_db.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;

        // 查询共享状态表
        let result: Option<(String, String, i64)> = conn
            .query_row(
                "SELECT value, owner, updated_at FROM shared_state WHERE key = ?1",
                rusqlite::params![key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .ok();

        match result {
            Some((value, owner, updated_at)) => Ok(ToolResult {
                success: true,
                output: serde_json::to_string_pretty(&json!({
                    "key": key,
                    "value": value,
                    "owner": owner,
                    "updated_at": updated_at
                }))
                .unwrap_or_default(),
                error: None,
            }),
            None => Ok(ToolResult {
                success: true,
                output: format!("Key '{key}' not found"),
                error: None,
            }),
        }
    }
}

// ── StateSetTool ────────────────────────────────────────────────

/// 设置共享状态工具
///
/// 在共享的键值存储中设置键值对。如果键已存在，将覆盖原有值。
/// 每个键值对记录所有者智能体和更新时间。
pub struct StateSetTool {
    /// IPC 数据库句柄
    ipc_db: Arc<IpcDb>,
    /// 安全策略，用于权限检查
    security: Arc<SecurityPolicy>,
}

impl StateSetTool {
    /// 创建新的状态设置工具实例
    ///
    /// # 参数
    ///
    /// - `ipc_db`：共享的 IPC 数据库句柄
    /// - `security`：安全策略引用
    pub(crate) fn new(ipc_db: Arc<IpcDb>, security: Arc<SecurityPolicy>) -> Self {
        Self { ipc_db, security }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for StateSetTool {
    /// 返回工具名称
    fn name(&self) -> &str {
        "state_set"
    }

    /// 返回工具功能描述（供 LLM 理解工具用途）
    fn description(&self) -> &str {
        "在共享的智能体间状态存储中设置键值对。覆盖该键的任何现有值。"
    }

    /// 返回工具参数 JSON Schema
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "key": {
                    "type": "string",
                    "description": "要设置的键"
                },
                "value": {
                    "type": "string",
                    "description": "要存储的值"
                }
            },
            "required": ["key", "value"]
        })
    }

    /// 执行状态设置
    ///
    /// # 参数
    ///
    /// - `args`：工具参数，必须包含 `key` 和 `value`
    ///
    /// # 返回值
    ///
    /// 成功返回确认信息，失败返回错误信息
    ///
    /// # 安全检查
    ///
    /// 执行前检查安全策略是否允许 `state_set` 操作
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 安全策略检查：验证是否有权限执行状态设置操作
        if let Err(error) = self.security.enforce_tool_operation(ToolOperation::Act, "state_set") {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(error) });
        }

        self.ipc_db.heartbeat();

        // 提取并验证键参数
        let key = match args.get("key").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'key' parameter".into()),
                });
            }
        };

        // 提取并验证值参数
        let value = match args.get("value").and_then(|v| v.as_str()) {
            Some(v) => v,
            None => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some("Missing 'value' parameter".into()),
                });
            }
        };

        let conn = self.ipc_db.conn.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
        let now = now_epoch();

        // 使用 INSERT OR REPLACE 确保键唯一性，存在则更新，不存在则插入
        conn.execute(
            "INSERT OR REPLACE INTO shared_state (key, value, owner, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![key, value, self.ipc_db.agent_id, now],
        )?;

        Ok(ToolResult { success: true, output: format!("State '{key}' updated"), error: None })
    }
}

// ── 测试模块 ───────────────────────────────────────────────

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
