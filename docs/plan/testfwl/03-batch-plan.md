# 分批补测计划

## 执行原则

- 每批只处理一个风险面，避免把测试补齐、重构、功能改动混在一起。
- 新增 Rust 单元测试继续放到独立 `*_tests.rs` 或 `tests/` 文件中。
- 不为测试便利新增重型依赖；优先 fake trait 实现、内存 fixture、临时目录。
- 安全路径优先负例，确保拒绝原因明确，不记录 secret、token 或原始敏感 payload。
- UI 相关测试优先抽可测纯函数；Iced 视觉验证另开任务，避免单元测试承担截图职责。

## Batch 0：真实覆盖率基线

目标：在明确授权后生成一次 `cargo llvm-cov` HTML 作为基线。

范围：
- workspace 总覆盖率。
- `vw-agent`、`vw-desktop`、`vw-acp` 三个主 crate 分包覆盖率。

验收：
- 产物位于 `coverage/workspace`、`coverage/vw-agent`、`coverage/vw-desktop`、`coverage/vw-acp`。
- 从 HTML 摘录行覆盖率、函数覆盖率、低覆盖文件 Top 20，回填到本目录。

备注：该批会执行测试，需用户明确授权。

## Batch 1：Agent 安全与 runtime

目标：提升高风险执行路径的行为覆盖。

范围：
- `vw-agent/tools/shell`：readonly、sandbox、path、security validator 的组合拒绝路径。
- `vw-agent/gateway/api`：auth、allowlist、idempotency、signature、sanitize、rate limiter 组合场景。
- `vw-agent/session/llm/aisdk/stream.rs`：fake stream chunk、abort、error、usage、tool call 合并。
- `vw-agent/workflow/runner.rs` 与 `code_runner.rs`：调度、条件分支、失败传播、循环上限。

验收：
- 每个新增测试名按行为/结果命名。
- 每个安全拒绝用例断言错误类别或关键错误信息。
- fake provider/fake stream 不访问真实网络。

## Batch 2：Desktop 配置与 UI 状态

目标：把“符号存在性”护栏升级为行为测试。

范围：
- `app/config_agent.rs`、`config_desktop.rs`、`config_gateway.rs`、`config_redis.rs`。
- `app/components/task_pet.rs` 中资源选择、状态转换、布局计算。
- `app/views/design/*` 中坐标、缩放、选择、overlay 纯逻辑。
- `app/components/chat_panel/*` 中 tool parse、message render metadata、权限展示。

验收：
- 测试断言输入到输出的行为，而不只断言源码包含符号。
- UI 颜色/布局逻辑覆盖暗黑主题关键分支。
- 滚动条宽度等 Iced UI 约束进入可测常量或样式函数。

## Batch 3：ACP 队列、会话与协议

目标：降低 ACP 客户端/队列/持久化的状态风险。

范围：
- `src/client/session_control.rs`、`protocol.rs`、`state.rs`、`runtime.rs`。
- `queue_ipc.rs`、`queue_ipc_server.rs`、`queue_lease_store.rs`。
- `session_persistence/*` 与 `session_runtime/*`。

验收：
- 覆盖 owner lease 获取/释放/过期。
- 覆盖 session load 缺文件、坏 JSON、版本字段缺失。
- 覆盖 prompt 中断、进程退出、错误归一化。

## Batch 4：契约 crate 快速补齐

目标：用低成本测试锁住跨 crate 契约。

范围：
- `vw-api-types`：serde roundtrip、字段默认值、错误 DTO。
- `vw-config-types`：配置默认值、兼容字段、未知字段策略。
- `vw-gateway-client`：endpoint、HTTP status、stream 错误映射。
- `vw-provider-resolver`：provider key、cache、安装检测、环境变量。
- `vw-shared`：task store、permission、json util、time util。

验收：
- 每个契约类型至少有 roundtrip 或默认值测试。
- 工厂 key 测试固定小写用户可见值，避免别名扩散。
- 错误路径断言保持稳定消息或稳定错误类型。

## Batch 5：回归维护

目标：避免覆盖率回升后再次滑落。

范围：
- 将真实覆盖率摘要写回 `docs/plan/testfwl`。
- 为高风险模块建立最小覆盖清单。
- 对“只做符号存在性”的测试标记迁移状态，逐步替换为行为测试。

验收：
- 每个新增高风险功能 PR 都有相邻行为测试。
- 文档记录最新覆盖率来源、命令和日期。
- 不新增没有真实调用者的测试抽象或测试框架层。
