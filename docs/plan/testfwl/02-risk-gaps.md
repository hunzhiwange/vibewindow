# 风险缺口

## P0：安全与权限边界

优先保证默认拒绝、权限不静默扩大、敏感信息不泄漏。

- `vw-agent/tools/shell`：已有大量解析、只读、sandbox、注入防护测试；建议补跨层组合用例，例如 readonly + path extraction + sandbox allow 同时生效时的拒绝路径。
- `vw-agent/security/policy`：静态面显示测试入口存在，但测试行数较少；建议补 shell lexer、redirect、命令策略、域名策略的负例矩阵。
- `vw-agent/gateway/api`：handler 数量多、生产行数大，测试行数相对偏低；建议优先覆盖认证失败、幂等、rate limit、签名、allowlist、payload sanitize 的组合路径。
- `vw-acp/persisted_key_policy`、`permissions`、`config`：已有集成测试，仍需确认错误形状、降级行为、持久化 key 的拒绝路径。

## P1：异步运行时与流程调度

这些路径最容易出现 race、状态遗漏和错误吞没。

- `crates/vw-agent/src/session/llm/aisdk/stream.rs`：文件约 584 行，已有少量内联测试覆盖 reasoning 配置；仍缺流 chunk 错误映射、abort、usage 汇总、tool call 增量合并。
- `crates/vw-agent/src/workflow/runner.rs`：文件约 459 行，缺明显相邻测试入口；应补 start node、条件分支、循环上限、失败节点、变量池输出写回。
- `crates/vw-agent/src/workflow/code_runner.rs`：文件约 202 行，建议补安全失败、stdout/stderr、超时、非 JSON 输出。
- `vw-acp/session_runtime/*`：已有生命周期与 runtime tests；建议补 connect/load 异常、owner process 退出、prompt runner 中断。

## P2：桌面 UI 状态与配置加载

`vw-desktop` 的静态测试数量很高，但不少测试仅确认符号仍存在，例如 `config_agent_tests_keeps_planned_coverage_targets`。这些测试适合作为迁移提醒，不应当作为行为覆盖的主要证据。

- `crates/vw-desktop/src/app/config_agent.rs`：约 1725 行，测试主要是目标符号存在性；应补 gateway 成功/失败、全局配置合并、patch 参数、默认值回退。
- `crates/vw-desktop/src/app/config_redis.rs`、`config_desktop.rs`、`config_gateway.rs`：建议补解析、保存失败、空配置、敏感字段不展示。
- `crates/vw-desktop/src/app/components/task_pet.rs`：约 828 行，缺明显行为测试；建议先拆出可测状态/布局计算，再补暗黑主题与状态流转测试。
- `crates/vw-desktop/src/app/views/design/*`：画布/toolbar/overlay 以 UI 组合为主，建议把坐标、选择、缩放、渲染参数拆到纯函数测试。

## P3：协议类型与小 crate 回归

这些 crate 的生产面较小，适合保持稳定高覆盖。

- `vw-config-types`：测试行数很少但入口齐；建议按配置域补 serde 默认值、别名迁移、未知字段拒绝/兼容策略。
- `vw-api-types`：继续覆盖 DTO serde roundtrip、错误字段、可选字段默认值。
- `vw-gateway-client`：补 endpoint 拼接、stream 错误、HTTP status 到领域错误映射。
- `vw-provider-resolver`：补 provider key 稳定性、安装检测失败、cache 失效、环境变量读取边界。
- `vw-webview`：目前测试极少；若保持薄封装可接受，若承担更多平台逻辑需增加解析/启动参数测试。

## 静态高风险文件清单

这些文件生产行数较高，或测试信号偏弱，适合作为后续补测入口：

| 文件 | 风险点 | 建议 |
|---|---|---|
| `crates/vw-agent/src/session/llm/aisdk/stream.rs` | 流式协议、错误映射、工具调用 | 建 fake stream，覆盖 chunk 序列。 |
| `crates/vw-agent/src/workflow/runner.rs` | 图调度、循环上限、失败传播 | 用 fake provider 和 YAML fixture。 |
| `crates/vw-agent/src/workflow/code_runner.rs` | 代码执行与错误输出 | 补超时、非零退出、输出解析。 |
| `crates/vw-desktop/src/app/config_agent.rs` | gateway 配置加载与 patch | 抽纯函数或 fake gateway。 |
| `crates/vw-desktop/src/app/components/task_pet.rs` | UI 状态、资源路径、布局 | 先抽状态计算，再补单测。 |
| `crates/vw-acp/src/client/session_control.rs` | 客户端会话控制 | 补状态转换与错误上下文。 |
| `crates/vw-acp/src/client/protocol.rs` | 协议边界 | 补序列化和错误形状。 |
