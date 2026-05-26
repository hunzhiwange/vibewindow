//! 后台进程管理工具
//!
//! 本模块提供后台进程的完整生命周期管理能力，支持启动、监控、输出读取和终止操作。
//!
//! # 主要功能
//!
//! - **进程启动（spawn）**：启动长时间运行的后台命令，支持异步输出流读取
//! - **进程列表（list）**：列出所有已启动的进程及其状态
//! - **输出读取（output）**：获取指定进程的标准输出和标准错误
//! - **进程终止（kill）**：优雅终止指定进程
//!
//! # 设计说明
//!
//! - 与同步的 `ShellTool` 互补，支持超过 60 秒超时限制的长时运行命令
//! - 输出缓冲区有大小限制（512KB），超出时自动丢弃旧数据
//! - 最大并发进程数限制为 8 个，防止资源耗尽
//! - 复用 shell 工具的安全检查链（速率限制、命令验证、路径检查）

use super::shell::apply_allowed_shell_environment;
use super::traits::{Tool, ToolResult};
use crate::app::agent::runtime::RuntimeAdapter;
use crate::app::agent::security::SecurityPolicy;
use crate::app::agent::security::SyscallAnomalyDetector;
use crate::app::agent::security::policy::ToolOperation;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::io::AsyncReadExt;

/// 每个输出流（stdout/stderr）保留的最大字节数：512KB
/// 超出此限制时，旧数据会被丢弃以保持缓冲区大小稳定
const MAX_OUTPUT_BYTES: usize = 524_288;

/// 最大并发后台进程数
/// 防止资源耗尽，限制同时运行的后台进程数量
const MAX_PROCESSES: usize = 8;

/// 已结束进程在自动清理前的最大保留时间（秒）
const PROCESS_MAX_AGE_SECS: i64 = 3600;

/// 输出缓冲区
///
/// 存储进程的标准输出或标准错误内容，具有大小限制。
/// 当缓冲区超出限制时，旧数据会被丢弃，`dropped_prefix_bytes` 记录已丢弃的字节数。
#[derive(Debug, Default, Clone)]
struct OutputBuffer {
    /// 当前缓冲区中的输出数据
    data: String,
    /// 已丢弃的前缀字节数（用于追踪输出偏移量）
    dropped_prefix_bytes: u64,
}

/// 进程条目
///
/// 表示一个正在运行或已完成的后台进程，包含进程信息和输出缓冲区。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Completed,
    Failed,
    Killed,
}

impl ProcessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Killed => "killed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub id: usize,
    pub title: Option<String>,
    pub command: String,
    pub metadata: Value,
    pub pid: u32,
    pub status: ProcessStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
}

struct ProcessEntry {
    /// 进程的唯一标识符（由 ProcessTool 分配）
    id: usize,
    /// 启动进程的原始命令字符串
    command: String,
    /// 可选的人类可读标题
    title: Option<String>,
    /// 结构化元数据
    metadata: Value,
    /// 操作系统进程 ID
    pid: u32,
    /// 进程启动时间
    started_at: DateTime<Utc>,
    /// 最近一次状态或元数据更新时间
    updated_at: DateTime<Utc>,
    /// 进程完成时间
    completed_at: Option<DateTime<Utc>>,
    /// 当前状态
    status: ProcessStatus,
    /// 退出码（若已结束）
    exit_code: Option<i32>,
    /// 子进程句柄（使用 Mutex 保护以支持异步等待）
    child: Arc<Mutex<tokio::process::Child>>,
    /// 标准输出缓冲区
    stdout_buf: Arc<Mutex<OutputBuffer>>,
    /// 标准错误缓冲区
    stderr_buf: Arc<Mutex<OutputBuffer>>,
    /// 已分析的输出偏移量 (stdout_offset, stderr_offset)
    /// 用于系统调用异常检测器追踪已检查的输出范围
    analyzed_offsets: Mutex<(u64, u64)>,
}

/// 后台进程管理工具
///
/// 允许代理启动长时间运行的命令、检查其输出并终止它们。
/// 与同步的 `ShellTool` 互补，支持需要运行超过 60 秒超时限制的命令。
///
/// # 支持的操作
///
/// - `spawn`：启动新的后台进程
/// - `list`：列出所有已启动的进程及其状态
/// - `output`：获取指定进程的输出（stdout/stderr）
/// - `kill`：终止指定进程
///
/// # 安全特性
///
/// - 复用 shell 工具的安全检查链（速率限制、命令验证、路径检查）
/// - 支持系统调用异常检测（可选）
/// - 最大并发进程数限制防止资源耗尽
pub struct ProcessTool {
    /// 安全策略引用
    security: Arc<SecurityPolicy>,
    /// 运行时适配器
    runtime: Arc<dyn RuntimeAdapter>,
    /// 系统调用异常检测器（可选）
    syscall_detector: Option<Arc<SyscallAnomalyDetector>>,
    /// 活跃进程映射表（ID -> 进程条目）
    processes: Arc<RwLock<HashMap<usize, ProcessEntry>>>,
    /// 下一个可用的进程 ID
    next_id: Mutex<usize>,
}

impl ProcessTool {
    /// 创建新的进程管理工具实例
    ///
    /// # 参数
    ///
    /// - `security`：安全策略引用，用于命令验证和访问控制
    /// - `runtime`：运行时适配器，用于构建和执行命令
    ///
    /// # 返回
    ///
    /// 返回不启用系统调用异常检测的工具实例
    pub fn new(security: Arc<SecurityPolicy>, runtime: Arc<dyn RuntimeAdapter>) -> Self {
        Self::new_with_syscall_detector(security, runtime, None)
    }

    /// 创建带有系统调用异常检测的进程管理工具实例
    ///
    /// # 参数
    ///
    /// - `security`：安全策略引用
    /// - `runtime`：运行时适配器
    /// - `syscall_detector`：可选的系统调用异常检测器
    ///
    /// # 返回
    ///
    /// 返回配置完成的工具实例
    pub fn new_with_syscall_detector(
        security: Arc<SecurityPolicy>,
        runtime: Arc<dyn RuntimeAdapter>,
        syscall_detector: Option<Arc<SyscallAnomalyDetector>>,
    ) -> Self {
        Self {
            security,
            runtime,
            syscall_detector,
            processes: Arc::new(RwLock::new(HashMap::new())),
            next_id: Mutex::new(0),
        }
    }

    fn refresh_processes(&self) {
        let now = Utc::now();
        let mut stale_ids = Vec::new();
        let mut processes = self.processes.write().unwrap();
        for (id, entry) in processes.iter_mut() {
            if matches!(entry.status, ProcessStatus::Running)
                && let Ok(mut child) = entry.child.lock()
                && let Ok(Some(status)) = child.try_wait()
            {
                entry.exit_code = status.code();
                entry.status = if status.success() {
                    ProcessStatus::Completed
                } else {
                    ProcessStatus::Failed
                };
                entry.completed_at = Some(now);
                entry.updated_at = now;
            }

            if !matches!(entry.status, ProcessStatus::Running)
                && entry
                    .completed_at
                    .is_some_and(|completed| (now - completed).num_seconds() > PROCESS_MAX_AGE_SECS)
            {
                stale_ids.push(*id);
            }
        }

        for id in stale_ids {
            processes.remove(&id);
        }
    }

    fn snapshot_entry(entry: &ProcessEntry) -> ProcessSnapshot {
        ProcessSnapshot {
            id: entry.id,
            title: entry.title.clone(),
            command: entry.command.clone(),
            metadata: entry.metadata.clone(),
            pid: entry.pid,
            status: entry.status.clone(),
            started_at: entry.started_at,
            updated_at: entry.updated_at,
            completed_at: entry.completed_at,
            exit_code: entry.exit_code,
        }
    }

    pub fn list_snapshots(&self) -> Vec<ProcessSnapshot> {
        self.refresh_processes();
        self.processes
            .read()
            .unwrap()
            .values()
            .map(Self::snapshot_entry)
            .collect()
    }

    pub fn get_snapshot(&self, id: usize) -> Option<ProcessSnapshot> {
        self.refresh_processes();
        self.processes.read().unwrap().get(&id).map(Self::snapshot_entry)
    }

    pub fn update_metadata(
        &self,
        id: usize,
        title: Option<Option<String>>,
        metadata: Option<Value>,
    ) -> bool {
        let mut processes = self.processes.write().unwrap();
        let Some(entry) = processes.get_mut(&id) else {
            return false;
        };

        if let Some(title) = title {
            entry.title = title;
        }
        if let Some(metadata) = metadata {
            entry.metadata = metadata;
        }
        entry.updated_at = Utc::now();
        true
    }

    pub fn output_snapshot(&self, id: usize) -> anyhow::Result<Value> {
        let (_entry, stdout_snapshot, stderr_snapshot) = {
            let mut attempts = 0usize;
            loop {
                self.refresh_processes();
                let processes = self.processes.read().unwrap();
                let entry = match processes.get(&id) {
                    Some(e) => e,
                    None => anyhow::bail!("No process with id {id}"),
                };

                let stdout_snapshot = snapshot_output_buffer(&entry.stdout_buf);
                let stderr_snapshot = snapshot_output_buffer(&entry.stderr_buf);
                if !stdout_snapshot.data.is_empty()
                    || !stderr_snapshot.data.is_empty()
                    || !matches!(entry.status, ProcessStatus::Running)
                    || attempts >= 25
                {
                    break (Self::snapshot_entry(entry), stdout_snapshot, stderr_snapshot);
                }

                attempts += 1;
                drop(processes);
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        let stdout = stdout_snapshot.data;
        let stderr = stderr_snapshot.data;

        if let Some(detector) = &self.syscall_detector {
            let processes = self.processes.read().unwrap();
            let Some(entry) = processes.get(&id) else {
                anyhow::bail!("No process with id {id}");
            };
            let mut offsets = entry.analyzed_offsets.lock().unwrap();
            let stdout_delta =
                slice_unseen_output(&stdout, stdout_snapshot.dropped_prefix_bytes, &mut offsets.0);
            let stderr_delta =
                slice_unseen_output(&stderr, stderr_snapshot.dropped_prefix_bytes, &mut offsets.1);

            if !stdout_delta.is_empty() || !stderr_delta.is_empty() {
                let _ =
                    detector.inspect_command_output(&entry.command, stdout_delta, stderr_delta, None);
            }
        }

        Ok(json!({
            "stdout": stdout,
            "stderr": stderr,
        }))
    }

    /// 处理 spawn 操作：启动新的后台进程
    ///
    /// # 参数
    ///
    /// - `args`：包含 `command`（必需）、`approved`、`title`、`metadata` 的 JSON 对象
    ///
    /// # 返回
    ///
    /// 成功时返回包含进程 ID 和 PID 的 JSON 结果；失败时返回错误信息
    ///
    /// # 安全检查流程
    ///
    /// 1. 检查运行时是否支持长时运行进程
    /// 2. 清理已退出的进程
    /// 3. 检查并发进程数限制
    /// 4. 速率限制检查
    /// 5. 命令验证（含审批状态）
    /// 6. 路径安全检查
    /// 7. 记录操作
    fn handle_spawn(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        // 检查运行时是否支持长时运行进程
        if !self.runtime.supports_long_running() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Runtime does not support long-running processes".into()),
            });
        }

        self.refresh_processes();

        // 提取并验证命令参数
        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter for spawn action"))?;
        let title = args.get("title").and_then(|value| value.as_str()).map(|value| value.trim());
        let title = title.filter(|value| !value.is_empty()).map(ToOwned::to_owned);
        let metadata = args
            .get("metadata")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default()));

        // 应用 shell 重定向策略
        let effective_command = self.security.apply_shell_redirect_policy(command);

        // 检查并发运行进程数量
        {
            let processes = self.processes.read().unwrap();
            let running =
                processes.values().filter(|entry| matches!(entry.status, ProcessStatus::Running)).count();
            if running >= MAX_PROCESSES {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Maximum concurrent processes ({MAX_PROCESSES}) reached")),
                });
            }
        }

        // 安全检查链：速率限制 → 命令验证 → 路径检查 → 记录操作
        if self.security.is_rate_limited() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Rate limit exceeded: too many actions in the last hour".into()),
            });
        }

        // 获取命令审批状态
        let approved = args.get("approved").and_then(|v| v.as_bool()).unwrap_or(false);

        // 验证命令执行权限
        if let Err(reason) = self.security.validate_command_execution(command, approved) {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(reason) });
        }

        // 检查是否包含禁止的路径参数
        if let Some(path) = self.security.forbidden_path_argument(&effective_command) {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Path blocked by security policy: {path}")),
            });
        }

        // 记录操作（检查操作预算）
        if !self.security.record_action() {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some("Rate limit exceeded: action budget exhausted".into()),
            });
        }

        // 通过运行时适配器构建命令
        let mut cmd = match self
            .runtime
            .build_shell_command(&effective_command, &self.security.workspace_dir)
        {
            Ok(cmd) => cmd,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to build runtime command: {e}")),
                });
            }
        };

        // 配置进程的标准输入输出
        cmd.stdin(Stdio::null()); // 不需要标准输入
        cmd.stdout(Stdio::piped()); // 管道化标准输出以异步读取
        cmd.stderr(Stdio::piped()); // 管道化标准错误以异步读取
        apply_allowed_shell_environment(&mut cmd, &self.security);

        // 启动进程
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to spawn process: {e}")),
                });
            }
        };

        // 获取进程 PID
        let Some(pid) = child.id() else {
            return Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(
                    "Failed to capture process PID for spawned child; process was not tracked"
                        .into(),
                ),
            });
        };

        // 设置后台输出读取器
        let stdout_buf = Arc::new(Mutex::new(OutputBuffer::default()));
        let stderr_buf = Arc::new(Mutex::new(OutputBuffer::default()));

        // 启动标准输出读取任务
        if let Some(stdout) = child.stdout.take() {
            spawn_reader_task(stdout, stdout_buf.clone());
        }
        // 启动标准错误读取任务
        if let Some(stderr) = child.stderr.take() {
            spawn_reader_task(stderr, stderr_buf.clone());
        }

        // 分配进程 ID
        let id = {
            let mut next = self.next_id.lock().unwrap();
            let id = *next;
            *next += 1;
            id
        };

        // 创建进程条目
        let now = Utc::now();
        let entry = ProcessEntry {
            id,
            command: command.to_string(),
            title,
            metadata,
            pid,
            started_at: now,
            updated_at: now,
            completed_at: None,
            status: ProcessStatus::Running,
            exit_code: None,
            child: Arc::new(Mutex::new(child)),
            stdout_buf,
            stderr_buf,
            analyzed_offsets: Mutex::new((0, 0)),
        };

        // 注册进程到映射表
        self.processes.write().unwrap().insert(id, entry);

        Ok(ToolResult {
            success: true,
            output: json!({
                "id": id,
                "pid": pid,
                "status": "running",
                "message": format!("Process started: {command}")
            })
            .to_string(),
            error: None,
        })
    }

    /// 处理 list 操作：列出所有已启动的进程
    ///
    /// # 返回
    ///
    /// 返回 JSON 数组，包含每个进程的 ID、命令、PID、状态和运行时间
    fn handle_list(&self) -> anyhow::Result<ToolResult> {
        // 检查读取进程的权限
        if let Err(e) = self.security.enforce_tool_operation(ToolOperation::Read, "process") {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(e) });
        }

        let entries = self
            .list_snapshots()
            .into_iter()
            .map(|entry| {
                let duration_secs = entry
                    .completed_at
                    .unwrap_or_else(Utc::now)
                    .signed_duration_since(entry.started_at)
                    .num_seconds()
                    .max(0);
                json!({
                    "id": entry.id,
                    "title": entry.title,
                    "command": entry.command,
                    "metadata": entry.metadata,
                    "pid": entry.pid,
                    "status": entry.status.as_str(),
                    "started_at": entry.started_at.to_rfc3339(),
                    "updated_at": entry.updated_at.to_rfc3339(),
                    "completed_at": entry.completed_at.map(|value| value.to_rfc3339()),
                    "exit_code": entry.exit_code,
                    "duration_secs": duration_secs,
                })
            })
            .collect::<Vec<_>>();

        Ok(ToolResult {
            success: true,
            output: serde_json::to_string_pretty(&entries).unwrap_or_default(),
            error: None,
        })
    }

    /// 处理 output 操作：获取指定进程的输出
    ///
    /// # 参数
    ///
    /// - `args`：包含 `id`（必需）的 JSON 对象
    ///
    /// # 返回
    ///
    /// 返回包含 `stdout` 和 `stderr` 字段的 JSON 结果
    ///
    /// # 系统调用检测
    ///
    /// 如果配置了系统调用异常检测器，会自动分析新增的输出内容
    fn handle_output(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        // 检查读取进程输出的权限
        if let Err(e) = self.security.enforce_tool_operation(ToolOperation::Read, "process") {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(e) });
        }

        // 解析进程 ID
        let id = parse_id(args, "output")?;
        let output = match self.output_snapshot(id) {
            Ok(output) => output,
            Err(_) => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("No process with id {id}")),
                });
            }
        };

        Ok(ToolResult {
            success: true,
            output: output.to_string(),
            error: None,
        })
    }

    /// 处理 kill 操作：终止指定进程
    ///
    /// # 参数
    ///
    /// - `args`：包含 `id`（必需）的 JSON 对象
    ///
    /// # 返回
    ///
    /// 成功时返回终止确认信息；失败时返回错误信息
    ///
    /// # 终止流程
    ///
    /// 1. 检查进程是否已退出
    /// 2. 发送终止信号
    /// 3. 等待最多 5 秒让进程退出
    /// 4. 超时则返回错误
    async fn handle_kill(&self, args: &serde_json::Value) -> anyhow::Result<ToolResult> {
        // 检查终止进程的权限
        if let Err(e) = self.security.enforce_tool_operation(ToolOperation::Act, "process") {
            return Ok(ToolResult { success: false, output: String::new(), error: Some(e) });
        }

        // 解析进程 ID
        let id = parse_id(args, "kill")?;
        self.refresh_processes();

        let pid = {
            let processes = self.processes.read().unwrap();
            let Some(entry) = processes.get(&id) else {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("No process with id {id}")),
                });
            };
            entry.pid
        };

        let already_done = self
            .get_snapshot(id)
            .filter(|snapshot| !matches!(snapshot.status, ProcessStatus::Running));
        if let Some(snapshot) = already_done {
            return Ok(ToolResult {
                success: true,
                output: format!(
                    "Process {id} (pid {pid}) already {} with exit status {:?}",
                    snapshot.status.as_str(),
                    snapshot.exit_code
                ),
                error: None,
            });
        }

        let child = {
            let processes = self.processes.read().unwrap();
            let Some(entry) = processes.get(&id) else {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!("No process with id {id}")),
                });
            };
            entry.child.clone()
        };

        {
            let mut child_guard = match child.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };

            if let Err(e) = child_guard.start_kill() {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Failed to initiate termination for process {id} (pid {pid}): {e}"
                    )),
                });
            }
        }

        // 等待进程退出（最多 5 秒）
        let wait_for_exit = async {
            loop {
                let status = {
                    let mut child = match child.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    child.try_wait()
                };

                match status {
                    Ok(Some(status)) => return Ok::<std::process::ExitStatus, std::io::Error>(status),
                    Ok(None) => tokio::time::sleep(Duration::from_millis(50)).await,
                    Err(error) => return Err(error),
                }
            }
        };

        match tokio::time::timeout(Duration::from_secs(5), wait_for_exit).await {
            Ok(Ok(status)) => {
                let exit_code = status.code();
                let now = Utc::now();
                if let Ok(mut processes) = self.processes.write()
                    && let Some(entry) = processes.get_mut(&id)
                {
                    entry.status = ProcessStatus::Killed;
                    entry.exit_code = exit_code;
                    entry.completed_at = Some(now);
                    entry.updated_at = now;
                }
                Ok(ToolResult {
                    success: true,
                    output: format!(
                        "Terminated process {id} (pid {pid}) with exit status {:?}",
                        exit_code
                    ),
                    error: None,
                })
            }
            Ok(Err(e)) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed waiting for process {id} (pid {pid}) to exit: {e}")),
            }),
            Err(_) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!(
                    "Timed out waiting for process {id} (pid {pid}) to exit after termination signal"
                )),
            }),
        }
    }
}

/// 从操作参数中解析 `id` 字段
///
/// # 参数
///
/// - `args`：JSON 参数对象
/// - `action`：操作名称（用于错误信息）
///
/// # 返回
///
/// 成功时返回解析后的 usize 类型的 ID；失败时返回错误
fn parse_id(args: &serde_json::Value, action: &str) -> anyhow::Result<usize> {
    args.get("id")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
        .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter for {action} action"))
}

/// 向有界缓冲区追加数据
///
/// 当缓冲区超出限制时，自动丢弃最旧的字节以保持大小稳定。
/// 丢弃时会确保不破坏 UTF-8 字符边界。
///
/// # 参数
///
/// - `buf`：输出缓冲区（使用 Mutex 保护）
/// - `new_data`：要追加的新数据
fn append_bounded(buf: &Mutex<OutputBuffer>, new_data: &str) {
    let mut guard = buf.lock().unwrap();
    guard.data.push_str(new_data);
    if guard.data.len() > MAX_OUTPUT_BYTES {
        let excess = guard.data.len() - MAX_OUTPUT_BYTES;
        // 找到 excess 位置之后第一个有效的字符边界
        let mut drain_to = excess;
        while drain_to < guard.data.len() && !guard.data.is_char_boundary(drain_to) {
            drain_to += 1;
        }
        // 丢弃前缀数据
        guard.data.drain(..drain_to);
        // 更新已丢弃字节数计数器
        guard.dropped_prefix_bytes =
            guard.dropped_prefix_bytes.saturating_add(u64::try_from(drain_to).unwrap_or(u64::MAX));
    }
}

/// 启动后台任务读取异步流到有界缓冲区
///
/// 创建一个 tokio 任务持续从指定的异步读取器读取数据，
/// 并将数据追加到有界缓冲区中。当读取器关闭或出错时任务自动退出。
///
/// # 参数
///
/// - `reader`：异步读取器（如 stdout/stderr 管道）
/// - `buf`：目标输出缓冲区
fn spawn_reader_task<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    mut reader: R,
    buf: Arc<Mutex<OutputBuffer>>,
) {
    tokio::spawn(async move {
        let mut chunk = vec![0u8; 8192]; // 8KB 读取缓冲区
        loop {
            match reader.read(&mut chunk).await {
                Ok(n) if n > 0 => {
                    // 将读取的字节转换为字符串（处理非 UTF-8 序列）
                    let text = String::from_utf8_lossy(&chunk[..n]);
                    append_bounded(&buf, &text);
                }
                _ => break, // EOF 或错误时退出
            }
        }
    });
}

/// 获取输出缓冲区的快照
///
/// 克隆缓冲区当前内容，用于安全读取而不长时间持有锁。
fn snapshot_output_buffer(buf: &Mutex<OutputBuffer>) -> OutputBuffer {
    buf.lock().unwrap().clone()
}

/// 提取未分析过的输出增量
///
/// 根据已丢弃的前缀字节数和已分析的偏移量，计算当前缓冲区中
/// 尚未被分析的部分。同时更新已分析偏移量。
///
/// # 参数
///
/// - `current`：当前缓冲区内容
/// - `dropped_prefix_bytes`：已丢弃的前缀字节数
/// - `analyzed`：已分析的偏移量（会被更新）
///
/// # 返回
///
/// 返回未分析过的输出切片
fn slice_unseen_output<'a>(
    current: &'a str,
    dropped_prefix_bytes: u64,
    analyzed: &mut u64,
) -> &'a str {
    let len_u64 = u64::try_from(current.len()).unwrap_or(u64::MAX);
    // 计算当前数据的逻辑结束位置
    let available_end = dropped_prefix_bytes.saturating_add(len_u64);

    // 计算在当前缓冲区中的起始位置
    let start = if *analyzed <= dropped_prefix_bytes {
        // 已分析的部分已全部丢弃，从头开始
        0
    } else {
        // 计算相对于当前缓冲区的偏移量
        usize::try_from(analyzed.saturating_sub(dropped_prefix_bytes))
            .unwrap_or(current.len())
            .min(current.len())
    };

    // 确保起始位置在字符边界上
    let mut boundary = start;
    while boundary < current.len() && !current.is_char_boundary(boundary) {
        boundary += 1;
    }

    // 更新已分析偏移量
    *analyzed = available_end;
    &current[boundary..]
}

/// Tool trait 实现
///
/// 将 ProcessTool 注册为可调用的工具，支持 spawn、list、output、kill 四种操作。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ProcessTool {
    /// 返回工具名称
    fn name(&self) -> &str {
        "process"
    }

    /// 返回工具描述
    fn description(&self) -> &str {
        "管理后台进程：启动长时间运行的命令、检查输出并终止它们"
    }

    /// 返回参数 JSON Schema
    ///
    /// 定义了以下参数：
    /// - `action`（必需）：操作类型（spawn/list/output/kill）
    /// - `command`：spawn 操作的命令字符串
    /// - `id`：output 和 kill 操作的进程 ID
    /// - `approved`：是否批准中/高风险命令
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["spawn", "list", "output", "kill"],
                    "description": "要执行的操作：spawn 启动进程，list 列出全部，output 获取输出，kill 终止进程"
                },
                "command": {
                    "type": "string",
                    "description": "在后台运行的 Shell 命令（'spawn' 必需）"
                },
                "title": {
                    "type": ["string", "null"],
                    "description": "可选的人类可读标题。"
                },
                "metadata": {
                    "type": "object",
                    "description": "可选的结构化元数据。"
                },
                "id": {
                    "type": "integer",
                    "description": "spawn 返回的进程 ID（'output' 和 'kill' 必需）"
                },
                "approved": {
                    "type": "boolean",
                    "description": "批准中/高风险命令（用于 'spawn'）",
                    "default": false
                }
            },
            "required": ["action"]
        })
    }

    /// 执行工具操作
    ///
    /// 根据 `action` 参数分发到对应的处理方法。
    ///
    /// # 参数
    ///
    /// - `args`：包含操作类型和参数的 JSON 对象
    ///
    /// # 返回
    ///
    /// 返回操作结果，成功时包含相关数据，失败时包含错误信息
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "spawn" => self.handle_spawn(&args),
            "list" => self.handle_list(),
            "output" => self.handle_output(&args),
            "kill" => self.handle_kill(&args).await,
            other => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown action '{other}'. Use: spawn, list, output, kill")),
            }),
        }
    }
}

/// Drop 实现
///
/// 当工具被销毁时，终止所有仍在运行的进程，防止孤儿进程。
impl Drop for ProcessTool {
    fn drop(&mut self) {
        if let Ok(processes) = self.processes.read() {
            for entry in processes.values() {
                if let Ok(mut child) = entry.child.lock() {
                    // 发送终止信号，忽略错误（进程可能已退出）
                    let _ = child.start_kill();
                }
            }
        }
    }
}

/// 单元测试模块
///
/// 测试文件位于同目录下的 tests.rs 中
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
