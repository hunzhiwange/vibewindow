好的，以下是完整的移植计划分析。

## 移植范围分析

TS `session-runtime.ts` 共 1662 行，包含以下顶层函数/类型：

| TS 导出 | Rust 对应状态 |
|---|---|
| `SessionAgentOptions` / `persistSessionOptions` | `prompt_runner.rs` 已有 `session_options_from_record`，但缺少 `persist_session_options` |
| `RunOnceOptions` / `runOnce()` | **缺失** |
| `SessionCreateOptions` / `createSession()` / `createSessionWithClient()` | **缺失** |
| `SessionSendOptions` / `sendSession()` / `sendSessionDirect()` | `sendSessionDirect` 已由 `run_session_prompt` 覆盖，但 `sendSession` 的队列调度逻辑**缺失** |
| `SessionEnsureOptions` / `ensureSession()` | **缺失** |
| `SessionCancelOptions` / `cancelSessionPrompt()` | **缺失** |
| `runSessionPrompt()` | **缺失**（核心函数，TS 611-928 行） |
| `runQueuedTask()` | **缺失** |
| `runSessionQueueOwner()` | **缺失**（TS 1241-1443 行，最大最复杂） |
| `QueueTaskOutputFormatter` / `DISCARD_OUTPUT_FORMATTER` | **缺失** |
| `normalizeQueueOwnerTtlMs()` | **缺失** |
| `filterRecoverableLoadFallbackOutput()` | **缺失** |
| `closeSession()` | 已在 `session_persistence` 中实现 |
| `isLikelyMatchingProcess()` / `firstAgentCommandToken()` | **缺失**（Linux `/proc` 特有） |

## 建议新增的文件结构

```
session_runtime/
├── connect_load.rs          (已有)
├── lifecycle.rs             (已有)
├── prompt_runner.rs         (已有)
├── queue_owner_process.rs   (已有)
├── runtime_types.rs         (新增) ← 所有 Options/Result 类型 + 常量 + 小工具函数
├── prompt_session.rs        (新增) ← runSessionPrompt + runQueuedTask + OutputFormatter 实现
├── run_once.rs              (新增) ← runOnce()
├── session_ops.rs           (新增) ← create/ensure/send/cancel 高层操作
├── queue_owner_runtime.rs   (新增) ← runSessionQueueOwner (队列主循环)
├── *_tests.rs               (新增，对应测试)
```

## 文件 1: `runtime_types.rs` (~200 行)

**内容：**
- 常量：`DEFAULT_QUEUE_OWNER_TTL_MS = 300_000`, `INTERRUPT_CANCEL_WAIT_MS = 2_500`, `QUEUE_OWNER_STARTUP_MAX_ATTEMPTS = 120`, `QUEUE_OWNER_HEARTBEAT_INTERVAL_MS = 5_000`
- Options 结构体（全部 `pub struct`，命名 `snake_case`）：
  - `RunOnceOptions`
  - `SessionCreateOptions`
  - `SessionCreateWithClientResult { record, client }`
  - `SessionSendOptions`
  - `SessionEnsureOptions`
  - `SessionCancelOptions` / `SessionCancelResult`
  - `RunSessionPromptOptions`（`RunSessionPromptOptions` 内部使用，不导出）
- `normalize_queue_owner_ttl_ms(ttl_ms: Option<u64>) -> u64`
- `persist_session_options(record: &mut SessionRecord, options: Option<&SessionAgentOptions>)`
- `session_options_from_record` 已在 `prompt_runner.rs` 中，可考虑移动到此文件并 re-export
- `filter_recoverable_load_fallback_output(messages: &mut Vec<AcpJsonRpcMessage>)` — 从 `runSessionPrompt` 中提取的 JSON-RPC 请求/响应过滤逻辑
- `emit_prompt_retry_notice(...)` — 可选，打印到 stderr

**依赖：** `types.rs`, `serde_json`

## 文件 2: `prompt_session.rs` (~350 行)

**内容（核心 `runSessionPrompt` + `runQueuedTask`）：**

1. `QueueTaskOutputFormatter` struct — 实现 `OutputFormatter` trait，将消息通过 `QueueTask::send()` 转发
2. `DiscardOutputFormatter` struct — 实现 `OutputFormatter` trait，全部丢弃
3. `run_queued_task(session_record_id, task, options)` — TS 532-609 行
   - 根据 `task.wait_for_completion` 选择 formatter
   - 调用 `run_session_prompt`
   - 成功发送 Result，失败发送 Error，finally 关闭 task
4. `run_session_prompt(options) -> Result<SessionSendResult, PromptRunnerError>` — TS 611-928 行
   - 解析 session record，克隆 conversation/vwacp state
   - 创建/复用 `AcpClient`
   - 设置事件处理器（ACP message、session update、client operation）
   - 构造 `ActiveSessionController`
   - 调用 `connect_and_load_session`
   - 带 retry 的 prompt 循环（指数退避，最多 `prompt_retries` 次）
   - `with_interrupt` 包裹，处理 SIGINT
   - finally：刷新事件、关闭 writer、持久化 record

**关键差异（TS → Rust 适配）：**
- TS 的 `eventWriter`（`SessionEventWriter`）已存在 Rust 版本
- TS 的 `conversation` 状态跟踪在 Rust 中通过 `SessionRecord` 内联字段实现（`messages`, `cumulative_token_usage` 等），无需单独 clone conversation
- `AcpClient` 的事件回调通过 builder pattern 设置，不是构造参数
- prompt 调用使用 `client.run_prompt(request, &mut on_event)` 而非 `client.prompt(sessionId, prompt)`

**依赖：** `client.rs`, `connect_load`, `lifecycle`, `runtime_types`, `session_conversation_model`, `session_events`, `session_persistence`, `session_runtime_helpers`, `error_normalization`

## 文件 3: `run_once.rs` (~120 行)

**内容（`runOnce()` — TS 930-1030 行）：**
- `pub async fn run_once(options: RunOnceOptions) -> Result<RunPromptResult, PromptRunnerError>`
- 创建 `AcpClient`
- 启动 + `create_session`
- 可选 `set_session_model`（如果有 `session_options.model`）
- 带 retry 的 prompt 循环
- `with_interrupt` 包裹

**依赖：** `client.rs`, `prompt_session`（可能共享 retry 逻辑）, `runtime_types`, `session_runtime_helpers`

## 文件 4: `session_ops.rs` (~350 行)

**内容：**

1. `create_session_record_with_client(client, options) -> Result<SessionRecord, SessionRuntimeError>` — TS 1032-1124 行
   - 启动 client，create/load session
   - 构建 `SessionRecord` 并持久化
2. `create_session_with_client(options) -> Result<SessionCreateWithClientResult, SessionRuntimeError>` — TS 1126-1159
3. `create_session(options) -> Result<SessionRecord, SessionRuntimeError>` — TS 1161-1168
4. `ensure_session(options) -> Result<SessionEnsureResult, SessionRuntimeError>` — TS 1170-1220
   - 目录遍历查找已有 session
   - 找到则可选更新 model，否则创建新 session
5. `send_session(options) -> Result<SessionSendOutcome, SessionRuntimeError>` — TS 1446-1465
   - 尝试提交到运行中的 owner
   - 失败则 spawn queue owner 进程
   - 轮询重试直到成功或超时
6. `send_session_direct(options) -> Result<SessionSendResult, SessionRuntimeError>` — TS 1467-1486
   - 直接调用 `run_session_prompt`
7. `cancel_session_prompt(options) -> Result<SessionCancelResult, SessionRuntimeError>` — TS 1488-1496

**新增 Error 类型：**
```rust
#[derive(Debug, thiserror::Error)]
pub enum SessionRuntimeError {
    #[error(transparent)]
    Persistence(#[from] SessionRepositoryError),
    #[error(transparent)]
    PromptRunner(#[from] PromptRunnerError),
    #[error(transparent)]
    Acp(#[from] AcpError),
    #[error(transparent)]
    Interrupted(#[from] InterruptedError),
    #[error("session queue owner failed to start for session {0}")]
    QueueOwnerStartupTimeout(String),
}
```

**依赖：** `client.rs`, `connect_load`, `prompt_session`, `runtime_types`, `session_persistence`, `queue_ipc`, `queue_owner_process`, `queue_lease_store`, `session_mode_preference`

## 文件 5: `queue_owner_runtime.rs` (~250 行)

**内容（`runSessionQueueOwner` — TS 1241-1443 行）：**
- `pub async fn run_session_queue_owner(options: QueueOwnerRuntimeOptions) -> Result<(), SessionRuntimeError>`
- 获取 lease
- 解析 session record，创建 shared `AcpClient`
- 创建 `QueueOwnerTurnController`（带 fallback 函数）
- 启动 `SessionQueueOwner`
- 心跳定时器（`QUEUE_OWNER_HEARTBEAT_INTERVAL_MS`）
- 主循环：`owner.next_task(poll_timeout)` → `run_queued_task`
- finally：清理 timer、关闭 owner、释放 lease、持久化 record

**关键差异：**
- TS 使用 `setInterval` → Rust 使用 `tokio::time::interval` 或 `tokio::spawn` 的周期任务
- TS 的 `NodeJS.Timeout` → Rust 用 `JoinHandle` 管理
- 心跳是 fire-and-forget，用 `tokio::spawn` 处理

**依赖：** `client.rs`, `prompt_session`, `queue_ipc_server`, `queue_lease_store`, `queue_owner_turn_controller`, `runtime_types`, `session_persistence`, `lifecycle`

## 文件 6: 更新 `session_runtime.rs`（mod 声明 + re-export）

```rust
pub mod connect_load;
pub mod lifecycle;
pub mod prompt_runner;
pub mod queue_owner_process;
pub mod runtime_types;      // 新增
pub mod prompt_session;     // 新增
pub mod run_once;           // 新增
pub mod session_ops;        // 新增
pub mod queue_owner_runtime;// 新增

// 新增 re-exports
pub use runtime_types::{...};
pub use prompt_session::{run_queued_task, run_session_prompt, ...};
pub use run_once::{run_once, RunOnceOptions, ...};
pub use session_ops::{create_session, ensure_session, send_session, ...};
pub use queue_owner_runtime::run_session_queue_owner;
```

## 不移植的部分（有意跳过）

| TS 函数 | 原因 |
|---|---|
| `isLikelyMatchingProcess()` | 读取 `/proc/{pid}/cmdline`，Linux 专属，macOS 上无意义 |
| `firstAgentCommandToken()` | 仅供 `isLikelyMatchingProcess` 使用 |
| `closeSession()` 的进程终止逻辑 | 已在 `session_persistence::close_session` 中实现 |
| `normalizeRuntimeSessionId` 的直接使用 | 已在 `runtime_session_id` 模块中 |
| `cloneSessionConversation` / `recordPromptSubmission` 等 conversation 操作 | 已在 `session_conversation_model` 中 |

## 实现顺序建议

1. **`runtime_types.rs`** — 无依赖，纯类型定义
2. **`prompt_session.rs`** — 核心逻辑，依赖 1
3. **`run_once.rs`** — 依赖 1、2
4. **`session_ops.rs`** — 依赖 1、2、3
5. **`queue_owner_runtime.rs`** — 依赖 1、2、4
6. **`session_runtime.rs` 更新** — 最后串接
7. **`cargo check --all-features`** 验证

## 总量估算

| 文件 | 预估行数 |
|---|---|
| `runtime_types.rs` | ~200 |
| `prompt_session.rs` | ~350 |
| `run_once.rs` | ~120 |
| `session_ops.rs` | ~350 |
| `queue_owner_runtime.rs` | ~250 |
| `session_runtime.rs` 更新 | ~30 |
| **合计** | **~1300 行** |