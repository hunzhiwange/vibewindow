//! ACP 客户端门面。
//!
//! 该模块提供面向上层调用方的 `AcpClient`，负责配置 ACP 代理进程、
//! 管理后台 actor 线程、转发会话控制请求，并暴露提示词执行、取消、
//! 权限统计和生命周期查询等能力。具体的 actor 循环、事件回调和辅助函数
//! 拆分到 `client/*` 子模块中。

use std::collections::{HashMap, HashSet};
use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{ExitStatus, Stdio};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;

use agent_client_protocol::{self as acp, Agent as _};
use parking_lot::Mutex;
use serde_json::{Map, Value, json};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
use tokio::process::Child;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::time::{Duration, timeout};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::acp_jsonrpc::is_acp_json_rpc_message;
use crate::error::AcpError;
use crate::filesystem::{FileSystemHandlers, FileSystemHandlersOptions};
use crate::permissions::{
    PermissionDecision, classify_permission_decision, resolve_permission_request,
};
use crate::session_persistence::iso_now;
use crate::session_runtime::{AgentLifecycleExit, AgentLifecycleSnapshot};
use crate::spawn_command_options::build_spawn_command;
use crate::terminal::{TerminalManager, TerminalManagerOptions};
use crate::types::{
    AcpAgentConfig, AcpJsonRpcMessage, AcpMessageCallback, AcpMessageDirection, AcpSessionOptions,
    AuthPolicy, ClientOperationCallback, NonInteractivePermissionPolicy, PermissionMode,
    PermissionStats, PromptEvent, PromptRequest, PromptResult, PromptUsage, SessionInfo,
    SessionStrategy, SessionUpdateCallback,
};

#[path = "client/actor.rs"]
mod actor;
#[path = "client/event_client.rs"]
mod event_client;
#[path = "client/helpers.rs"]
mod helpers;
#[path = "client/message_io.rs"]
mod message_io;

#[cfg(test)]
#[path = "client/event_client_tests.rs"]
mod event_client_tests;
#[cfg(test)]
#[path = "client/helpers_tests.rs"]
mod helpers_tests;
#[cfg(test)]
#[path = "client/message_io_tests.rs"]
mod message_io_tests;

use self::helpers::*;

const DEFAULT_ACTOR_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Clone)]
/// ACP 客户端的可复用句柄。
///
/// 该类型是上层与 ACP 代理交互的主要入口。它持有启动配置、权限策略、
/// 回调和后台 actor 状态；克隆句柄会共享同一个 actor 状态和当前提示词控制。
pub struct AcpClient {
    agent_name: String,
    config: AcpAgentConfig,
    client_name: String,
    client_version: String,
    mcp_servers: Vec<acp::McpServer>,
    permission_mode: PermissionMode,
    non_interactive_permissions: Option<NonInteractivePermissionPolicy>,
    auth_credentials: HashMap<String, String>,
    auth_policy: AuthPolicy,
    session_options: Option<AcpSessionOptions>,
    verbose: bool,
    on_acp_message: Option<AcpMessageCallback>,
    on_acp_output_message: Option<AcpMessageCallback>,
    on_session_update: Option<SessionUpdateCallback>,
    on_client_operation: Option<ClientOperationCallback>,
    permission_stats: Arc<Mutex<PermissionStats>>,
    active_prompt: Arc<Mutex<Option<ActivePromptControl>>>,
    cancelling_session_ids: Arc<Mutex<HashSet<String>>>,
    actor_state: Arc<Mutex<AcpClientActorState>>,
    actor_idle_timeout: Duration,
}

#[derive(Debug)]
enum InternalEvent {
    Delta(String),
    SessionChanged { expected: String, actual: String },
}

#[derive(Debug, Clone)]
struct AuthSelection {
    method_id: String,
    source: &'static str,
}

#[derive(Clone)]
struct ActivePromptControl {
    session_id: String,
    cancel_tx: watch::Sender<bool>,
    completed_rx: watch::Receiver<bool>,
}

struct AcpEventClient {
    expected_session_id: Arc<Mutex<Option<String>>>,
    event_tx: mpsc::UnboundedSender<InternalEvent>,
    filesystem: FileSystemHandlers,
    terminal_manager: TerminalManager,
    permission_mode: PermissionMode,
    non_interactive_permissions: Option<NonInteractivePermissionPolicy>,
    on_session_update: Option<SessionUpdateCallback>,
    permission_stats: Arc<Mutex<PermissionStats>>,
    cancelling_session_ids: Arc<Mutex<HashSet<String>>>,
}

/// 已启动 ACP 代理进程的基础句柄集合。
///
/// `child` 保留子进程控制权，`stderr_task` 异步收集标准错误输出，用于进程
/// 初始化或会话创建失败时补充诊断上下文。
pub(crate) struct ProcessHandles {
    child: Child,
    stderr_task: Option<tokio::task::JoinHandle<String>>,
}

#[derive(Default)]
struct AcpClientActorState {
    handle: Option<AcpClientActorHandle>,
    reusable_session_id: Option<String>,
    lifecycle: AgentLifecycleSnapshot,
}

struct AcpClientActorHandle {
    command_tx: mpsc::UnboundedSender<ActorCommand>,
    thread: Option<thread::JoinHandle<()>>,
}

struct ActorRuntime {
    cwd: PathBuf,
    child: Child,
    stderr_task: Option<tokio::task::JoinHandle<String>>,
    expected_session_id: Arc<Mutex<Option<String>>>,
    event_rx: mpsc::UnboundedReceiver<InternalEvent>,
    conn: acp::ClientSideConnection,
    io_closed_rx: oneshot::Receiver<()>,
    io_task: tokio::task::JoinHandle<()>,
}

enum ActorCommand {
    CreateSession {
        cwd: PathBuf,
        response_tx: oneshot::Sender<Result<SessionInfo, AcpError>>,
    },
    LoadSession {
        session_id: String,
        cwd: PathBuf,
        response_tx: oneshot::Sender<Result<SessionInfo, AcpError>>,
    },
    ResumeSession {
        session_id: String,
        cwd: PathBuf,
        response_tx: oneshot::Sender<Result<SessionInfo, AcpError>>,
    },
    SetSessionMode {
        session_id: String,
        cwd: PathBuf,
        mode_id: String,
        response_tx: oneshot::Sender<Result<(), AcpError>>,
    },
    SetSessionConfigOption {
        session_id: String,
        cwd: PathBuf,
        option_name: String,
        value_id: String,
        response_tx: oneshot::Sender<Result<acp::SetSessionConfigOptionResponse, AcpError>>,
    },
    SetSessionModel {
        session_id: String,
        cwd: PathBuf,
        model: String,
        response_tx: oneshot::Sender<Result<(), AcpError>>,
    },
    RunPrompt {
        request: PromptRequest,
        event_tx: mpsc::UnboundedSender<PromptEvent>,
        response_tx: oneshot::Sender<Result<PromptResult, AcpError>>,
    },
    Close {
        response_tx: oneshot::Sender<()>,
    },
}

#[derive(Clone, Default)]
struct ChildExitSummary {
    exit_code: Option<i32>,
    signal: Option<String>,
}

#[derive(Default)]
/// 已结束代理进程的收尾结果。
///
/// 该结构将退出状态和 stderr 输出分开保存，便于调用方按错误类型决定是否
/// 将进程上下文合并到用户可见错误里。
pub(crate) struct FinalizedChild {
    summary: ChildExitSummary,
    stderr_output: String,
}

#[derive(Clone)]
struct MessageTap {
    direction: AcpMessageDirection,
    on_message: Option<AcpMessageCallback>,
    on_output_message: Option<AcpMessageCallback>,
    buffer: Vec<u8>,
}

struct TappedReader<R> {
    inner: R,
    tap: MessageTap,
}

struct TappedWriter<W> {
    inner: W,
    tap: MessageTap,
}

impl AcpClient {
    /// 创建一个新的 ACP 客户端。
    ///
    /// `agent_name` 用于日志和生命周期标识，`config` 描述实际启动命令。
    /// 返回的客户端尚未启动代理进程；首次会话或提示词请求会按需启动。
    pub fn new(agent_name: impl Into<String>, config: AcpAgentConfig) -> Self {
        Self {
            agent_name: agent_name.into(),
            config,
            client_name: "vibewindow-acp-client".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            mcp_servers: Vec::new(),
            permission_mode: PermissionMode::ApproveAll,
            non_interactive_permissions: None,
            auth_credentials: HashMap::new(),
            auth_policy: AuthPolicy::Skip,
            session_options: None,
            verbose: false,
            on_acp_message: None,
            on_acp_output_message: None,
            on_session_update: None,
            on_client_operation: None,
            permission_stats: Arc::new(Mutex::new(PermissionStats::default())),
            active_prompt: Arc::new(Mutex::new(None)),
            cancelling_session_ids: Arc::new(Mutex::new(HashSet::new())),
            actor_state: Arc::new(Mutex::new(AcpClientActorState::default())),
            actor_idle_timeout: DEFAULT_ACTOR_IDLE_TIMEOUT,
        }
    }

    /// 设置初始化握手时上报给 ACP 代理的客户端信息。
    ///
    /// 返回更新后的客户端，便于链式配置。该方法不执行 I/O，也不会校验
    /// 代理是否接受这些元数据。
    pub fn with_client_info(
        mut self,
        client_name: impl Into<String>,
        client_version: impl Into<String>,
    ) -> Self {
        self.client_name = client_name.into();
        self.client_version = client_version.into();
        self
    }

    /// 设置创建、加载或恢复会话时传递给代理的 MCP 服务器列表。
    pub fn with_mcp_servers(mut self, mcp_servers: Vec<acp::McpServer>) -> Self {
        self.mcp_servers = mcp_servers;
        self
    }

    /// 设置文件系统和终端能力的权限模式。
    pub fn with_permission_mode(mut self, permission_mode: PermissionMode) -> Self {
        self.permission_mode = permission_mode;
        self
    }

    /// 设置非交互模式下的权限策略。
    ///
    /// 当调用方无法弹出权限确认界面时，该策略决定 ACP 权限请求如何自动处理。
    pub fn with_non_interactive_permissions(
        mut self,
        non_interactive_permissions: Option<NonInteractivePermissionPolicy>,
    ) -> Self {
        self.non_interactive_permissions = non_interactive_permissions;
        self
    }

    /// 设置创建新会话时附带的会话选项。
    ///
    /// 当前这些选项会被转换到 ACP `meta` 字段中，主要用于兼容支持该扩展的代理。
    pub fn with_session_options(mut self, session_options: Option<AcpSessionOptions>) -> Self {
        self.session_options = session_options;
        self
    }

    /// 设置用于 ACP 认证的凭据映射。
    ///
    /// 凭据只用于环境变量注入和认证方法选择；日志路径不会输出原始凭据值。
    pub fn with_auth_credentials(mut self, auth_credentials: HashMap<String, String>) -> Self {
        self.auth_credentials = auth_credentials;
        self
    }

    /// 设置代理声明认证方法但找不到凭据时的处理策略。
    pub fn with_auth_policy(mut self, auth_policy: AuthPolicy) -> Self {
        self.auth_policy = auth_policy;
        self
    }

    /// 设置是否启用更详细的诊断日志。
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// 设置原始 ACP JSON-RPC 消息回调。
    pub fn with_acp_message_callback(mut self, callback: Option<AcpMessageCallback>) -> Self {
        self.on_acp_message = callback;
        self
    }

    /// 设置面向输出流的 ACP 消息回调。
    pub fn with_acp_output_message_callback(
        mut self,
        callback: Option<AcpMessageCallback>,
    ) -> Self {
        self.on_acp_output_message = callback;
        self
    }

    /// 设置 ACP 会话更新回调。
    pub fn with_session_update_callback(mut self, callback: Option<SessionUpdateCallback>) -> Self {
        self.on_session_update = callback;
        self
    }

    /// 设置客户端侧文件系统和终端操作回调。
    pub fn with_client_operation_callback(
        mut self,
        callback: Option<ClientOperationCallback>,
    ) -> Self {
        self.on_client_operation = callback;
        self
    }

    #[cfg(test)]
    fn with_actor_idle_timeout(mut self, actor_idle_timeout: Duration) -> Self {
        self.actor_idle_timeout = actor_idle_timeout;
        self
    }

    /// 返回最近一次提示词或权限流程累计的权限统计。
    pub fn permission_stats(&self) -> PermissionStats {
        self.permission_stats.lock().clone()
    }

    /// 判断当前是否存在仍在运行的提示词请求。
    pub fn has_active_prompt(&self) -> bool {
        self.active_prompt.lock().is_some()
    }

    /// 启动后台 actor 线程。
    ///
    /// 如果 actor 已在运行则直接返回 `Ok(())`。启动失败会返回初始化或线程创建
    /// 相关错误，并清理已记录的 actor 状态。
    pub async fn start(&self) -> Result<(), AcpError> {
        {
            let state = self.actor_state.lock();
            if let Some(handle) = state.handle.as_ref()
                && !handle.command_tx.is_closed()
            {
                return Ok(());
            }
        }

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (startup_tx, startup_rx) = oneshot::channel();
        let actor_client = self.clone();
        let thread = thread::Builder::new()
            .name(format!("vw-acp-{}", self.agent_name))
            .spawn(move || actor_client.run_actor_thread(command_rx, startup_tx))
            .map_err(AcpError::Spawn)?;

        {
            let mut state = self.actor_state.lock();
            state.handle = Some(AcpClientActorHandle { command_tx, thread: Some(thread) });
            state.reusable_session_id = None;
        }

        match startup_rx.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => {
                self.invalidate_actor();
                Err(err)
            }
            Err(err) => {
                self.invalidate_actor();
                Err(AcpError::Initialize(err.to_string()))
            }
        }
    }

    /// 在指定工作目录创建新的 ACP 会话。
    ///
    /// 该方法会按需启动代理进程。返回值包含代理生成的会话 ID；初始化、
    /// 进程启动或 `session/new` 失败会以 [`AcpError`] 返回。
    pub async fn create_session(&self, cwd: impl AsRef<Path>) -> Result<SessionInfo, AcpError> {
        let cwd = cwd.as_ref().to_path_buf();
        self.send_actor_request(move |response_tx| ActorCommand::CreateSession { cwd, response_tx })
            .await
    }

    /// 加载一个已存在的 ACP 会话。
    ///
    /// `session_id` 是调用方期望加载的会话，`cwd` 是代理侧会话工作目录。
    /// 如果代理拒绝加载或通信失败，返回对应的 [`AcpError`]。
    pub async fn load_session(
        &self,
        session_id: impl Into<String>,
        cwd: impl AsRef<Path>,
    ) -> Result<SessionInfo, AcpError> {
        let session_id = session_id.into();
        let cwd = cwd.as_ref().to_path_buf();
        self.send_actor_request(move |response_tx| ActorCommand::LoadSession {
            session_id,
            cwd,
            response_tx,
        })
        .await
    }

    /// 恢复一个已存在的 ACP 会话。
    ///
    /// 与 [`load_session`](Self::load_session) 类似，但调用 ACP `session/resume`
    /// 能力；不支持该能力的代理会返回错误。
    pub async fn resume_session(
        &self,
        session_id: impl Into<String>,
        cwd: impl AsRef<Path>,
    ) -> Result<SessionInfo, AcpError> {
        let session_id = session_id.into();
        let cwd = cwd.as_ref().to_path_buf();
        self.send_actor_request(move |response_tx| ActorCommand::ResumeSession {
            session_id,
            cwd,
            response_tx,
        })
        .await
    }

    /// 设置已有会话的模式。
    ///
    /// 方法会先解析并确认目标会话，再调用 `session/set_mode`。代理不支持该
    /// 控制能力或模式无效时会返回错误。
    pub async fn set_session_mode(
        &self,
        session_id: impl Into<String>,
        cwd: impl AsRef<Path>,
        mode_id: impl Into<String>,
    ) -> Result<(), AcpError> {
        let session_id = session_id.into();
        let cwd = cwd.as_ref().to_path_buf();
        let mode_id = mode_id.into();
        self.send_actor_request(move |response_tx| ActorCommand::SetSessionMode {
            session_id,
            cwd,
            mode_id,
            response_tx,
        })
        .await
    }

    /// 设置已有会话的配置选项。
    ///
    /// 返回代理响应中的可用选项状态；代理拒绝、参数无效或通信失败时返回
    /// [`AcpError`]，错误信息会尽量保留 ACP 错误摘要。
    pub async fn set_session_config_option(
        &self,
        session_id: impl Into<String>,
        cwd: impl AsRef<Path>,
        option_name: impl Into<String>,
        value_id: impl Into<String>,
    ) -> Result<acp::SetSessionConfigOptionResponse, AcpError> {
        let session_id = session_id.into();
        let cwd = cwd.as_ref().to_path_buf();
        let option_name = option_name.into();
        let value_id = value_id.into();
        self.send_actor_request(move |response_tx| ActorCommand::SetSessionConfigOption {
            session_id,
            cwd,
            option_name,
            value_id,
            response_tx,
        })
        .await
    }

    /// 设置已有会话的模型。
    ///
    /// 该方法依赖代理实现 `session/set_model`；不支持或模型不可用时返回错误。
    pub async fn set_session_model(
        &self,
        session_id: impl Into<String>,
        cwd: impl AsRef<Path>,
        model: impl Into<String>,
    ) -> Result<(), AcpError> {
        let session_id = session_id.into();
        let cwd = cwd.as_ref().to_path_buf();
        let model = model.into();
        self.send_actor_request(move |response_tx| ActorCommand::SetSessionModel {
            session_id,
            cwd,
            model,
            response_tx,
        })
        .await
    }

    /// 在目标会话策略下执行提示词。
    ///
    /// `request` 描述工作目录、会话策略和提示词文本；`on_event` 会接收文本增量
    /// 和会话切换等运行时事件。返回值包含最终会话 ID、增量集合、结束原因和
    /// 用量信息。启动、会话解析、提示词执行或取消流程失败时返回 [`AcpError`]。
    pub async fn run_prompt(
        &self,
        request: PromptRequest,
        on_event: &mut impl FnMut(PromptEvent),
    ) -> Result<PromptResult, AcpError> {
        self.start().await?;
        let command_tx = self.actor_command_tx()?;
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let (response_tx, response_rx) = oneshot::channel();
        command_tx.send(ActorCommand::RunPrompt { request, event_tx, response_tx }).map_err(
            |_| {
                self.invalidate_actor();
                AcpError::Initialize("ACP client actor is unavailable".to_string())
            },
        )?;

        let mut events_open = true;
        tokio::pin!(response_rx);

        loop {
            tokio::select! {
                result = &mut response_rx => {
                    let result = result.map_err(|err| {
                        self.invalidate_actor();
                        AcpError::PromptJoin(err.to_string())
                    })?;
                    while let Ok(event) = event_rx.try_recv() {
                        on_event(event);
                    }
                    return result;
                }
                maybe_event = event_rx.recv(), if events_open => {
                    match maybe_event {
                        Some(event) => on_event(event),
                        None => events_open = false,
                    }
                }
            }
        }
    }

    async fn shutdown_io_task(&self, io_task: tokio::task::JoinHandle<()>) {
        io_task.abort();
        let _ = io_task.await;
    }

    /// 关闭后台 actor 和当前代理进程。
    ///
    /// 该方法会等待 actor 线程退出；没有运行中的 actor 时返回 `Ok(())`。
    pub async fn close(&self) -> Result<(), AcpError> {
        let mut handle = {
            let mut state = self.actor_state.lock();
            state.reusable_session_id = None;
            state.handle.take()
        };
        let Some(mut handle) = handle.take() else {
            return Ok(());
        };

        let (response_tx, response_rx) = oneshot::channel();
        if handle.command_tx.send(ActorCommand::Close { response_tx }).is_ok() {
            let _ = response_rx.await;
        }

        if let Some(thread) = handle.thread.take() {
            let _ = tokio::task::spawn_blocking(move || thread.join()).await;
        }

        Ok(())
    }

    fn actor_command_tx(&self) -> Result<mpsc::UnboundedSender<ActorCommand>, AcpError> {
        self.actor_state
            .lock()
            .handle
            .as_ref()
            .filter(|handle| !handle.command_tx.is_closed())
            .map(|handle| handle.command_tx.clone())
            .ok_or_else(|| AcpError::Initialize("ACP client actor is not running".to_string()))
    }

    async fn send_actor_request<T, F>(&self, build: F) -> Result<T, AcpError>
    where
        T: Send + 'static,
        F: FnOnce(oneshot::Sender<Result<T, AcpError>>) -> ActorCommand,
    {
        self.start().await?;
        let command_tx = self.actor_command_tx()?;
        let (response_tx, response_rx) = oneshot::channel();
        command_tx.send(build(response_tx)).map_err(|_| {
            self.invalidate_actor();
            AcpError::Initialize("ACP client actor is unavailable".to_string())
        })?;
        response_rx.await.map_err(|err| {
            self.invalidate_actor();
            AcpError::PromptJoin(err.to_string())
        })?
    }

    fn invalidate_actor(&self) {
        let mut state = self.actor_state.lock();
        state.handle = None;
        state.reusable_session_id = None;
        state.lifecycle.pid = None;
    }

    /// 判断指定会话是否正被当前 actor 运行时复用。
    pub fn has_reusable_session(&self, session_id: &str) -> bool {
        self.actor_state
            .lock()
            .reusable_session_id
            .as_deref()
            .is_some_and(|active_session_id| active_session_id == session_id)
    }

    /// 获取当前代理进程生命周期快照。
    pub fn get_agent_lifecycle_snapshot(&self) -> AgentLifecycleSnapshot {
        self.actor_state.lock().lifecycle.clone()
    }

    /// 请求取消指定会话上的活动提示词。
    ///
    /// 返回 `Ok(true)` 表示已向活动提示词发送取消信号，`Ok(false)` 表示没有匹配
    /// 的活动提示词。底层取消通道发送失败时返回 [`AcpError::Cancel`]。
    pub async fn cancel(&self, session_id: impl AsRef<str>) -> Result<bool, AcpError> {
        let session_id = session_id.as_ref();
        let active_prompt = self.active_prompt.lock().clone();
        let Some(active_prompt) = active_prompt else {
            return Ok(false);
        };
        if active_prompt.session_id != session_id {
            return Ok(false);
        }

        self.cancelling_session_ids.lock().insert(session_id.to_string());
        active_prompt.cancel_tx.send(true).map_err(|err| AcpError::Cancel(err.to_string()))?;
        Ok(true)
    }

    /// 请求取消当前活动提示词，但不等待其完成。
    pub async fn request_cancel_active_prompt(&self) -> Result<bool, AcpError> {
        let active_prompt = self.active_prompt.lock().clone();
        let Some(active_prompt) = active_prompt else {
            return Ok(false);
        };
        self.cancel(active_prompt.session_id).await
    }

    /// 请求取消当前活动提示词，并可选择等待完成。
    ///
    /// `wait_ms` 为 `0` 时只发送取消请求；大于 `0` 时最多等待对应毫秒数。
    /// 返回 `false` 表示没有活动提示词、取消未发出或等待超时。
    pub async fn cancel_active_prompt(&self, wait_ms: u64) -> Result<bool, AcpError> {
        let active_prompt = self.active_prompt.lock().clone();
        let Some(active_prompt) = active_prompt else {
            return Ok(false);
        };

        let requested = self.cancel(&active_prompt.session_id).await?;
        if !requested || wait_ms == 0 {
            return Ok(requested);
        }

        let mut completed_rx = active_prompt.completed_rx.clone();
        if *completed_rx.borrow() {
            return Ok(true);
        }

        match timeout(Duration::from_millis(wait_ms), completed_rx.changed()).await {
            Ok(Ok(_)) | Ok(Err(_)) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
