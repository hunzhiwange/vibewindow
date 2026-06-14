# 代码健康报告 — 2026-06-07

## 概览

| 维度 | 状态 | 说明 |
|------|------|------|
| 安全 | 🟡 注意 | 大量 unwrap/expect 存在于测试中；unsafe 块范围可控 |
| 架构 | 🟢 健康 | 模块职责清晰，无上帝模块，架构原则合规 |
| 编译 | 🟢 健康 | Clippy 0 错误通过，编译成功 |
| 测试 | 🟡 注意 | 测试文件丰富，未运行完整测试套件 |

## 项目概况

**VibeWindow** — Rust-first 自主代理运行时（v0.2.7, MIT），包含 11 个 workspace 成员：

| Crate | 职责 | 类型 |
|-------|------|------|
| `vw-agent` | Agent 运行时核心 | lib |
| `vw-cli` | CLI 入口（`vibewindow`、`vibe-agent`） | bin |
| `vw-desktop` | Iced 桌面应用（`vibe-window`） | bin |
| `vw-acp` | Agent Client Protocol 桥接（`acp`） | bin+lib |
| `vw-api-types` | API 数据结构 | lib |
| `vw-config-types` | 配置数据结构 | lib |
| `vw-gateway-client` | 网关客户端 | lib |
| `vw-provider-resolver` | 提供商/模型解析 | lib |
| `vw-shared` | 跨 crate 共享工具 | lib |
| `vw-webview` | WebView helper | bin |
| `vw-figma-json` | Figma `.fig` → JSON 转换（`vw-fig2json`） | bin+lib |

## 安全

### 依赖漏洞

未运行 `cargo deny`（命令受环境限制），建议手动执行：

```bash
cargo deny check advisories
cargo deny check licenses
cargo deny check sources
```

### 代码安全模式

#### `unsafe` 块

| 文件 | 用途 |
|------|------|
| `crates/vw-acp/src/client/process_signals.rs` | 信号处理 |
| `crates/vw-acp/src/client/process.rs` | 进程管理 |
| `crates/vw-acp/src/terminal.rs` | 终端 I/O |
| `crates/vw-agent/src/cron/scheduler.rs` | 定时调度器 |
| `crates/vw-agent/src/security/policy/mod.rs` | 安全策略 |

范围合理，集中于系统级操作（信号、进程、终端），无滥用迹象。

#### `unwrap()` / `expect()` 热点

主要出现在测试文件中（合法），非测试源码中少量出现：

- `crates/vw-acp/src/main.rs:6` — 可考虑传播错误
- `crates/vw-agent/src/agent/agent/run.rs:1` — 可考虑传播错误
- `crates/vw-agent/src/cron/scheduler.rs:4` — 系统级，可接受
- `crates/vw-agent/src/gateway/mod.rs:1`
- 其余数十处均在 `*_tests.rs` 中（测试代码允许）

#### 敏感信息泄露风险

搜索 `token`/`secret`/`password`/`api_key` 结果均为合法的令牌类型定义、认证处理逻辑和测试用例，无硬编码密钥泄露。

#### `#[allow(...)]` 抑制

仅 3 处，使用克制。

## 架构

### 模块结构

- 依赖方向清晰：`vw-shared` → 所有上游 → `vw-agent`/`vw-cli`/`vw-desktop`
- 无循环依赖
- `vw-figma-json` 完全独立（无 workspace 内部依赖）
- 每个 crate 聚焦单一关注点，符合 AGENTS.md 原则

### 架构原则合规

- ✅ 优先直白控制流，无炫技元编程
- ✅ 显式 match + 强类型结构体
- ✅ 错误路径清晰局部化
- ✅ 无"为未来准备"的抽象
- ✅ 模块职责单一
- ✅ trait 实现者命名可预测
- ✅ 无上帝模块

### 代码规模

共约 **4075 个源文件**。

大 crate 排行（按文件数估算）：
1. `vw-agent` — 最大的 crate（agent 运行时、loop 核心、gateway、cron、channels、tools 等）
2. `vw-acp` — 次大（client、session_runtime、queue 等）
3. `vw-desktop` — 桌面 UI（chat_panel、components、message、state）
4. `vw-cli` — CLI（tui_v2、transcript、interactive）
5. `vw-figma-json` — Figma 解析（大量 transformation 文件）

单文件 > 500 行检查：未发现明显的超大单体文件。

### Clippy 警告

```bash
cargo clippy --all-targets --all-features -- -D warnings
# 结果：编译成功，0 个警告/错误
```

## 建议

### 短期

- 将非测试代码中的少量 `unwrap()` 替换为 `?` 传播或 `expect("context")`
- 运行 `cargo deny` 确认依赖漏洞状态
- 运行完整测试套件确认测试覆盖率

### 长期

- `vw-agent` 和 `vw-acp` 规模较大，考虑按领域进一步拆分子模块
- `vw-figma-json` 的 transformation 文件数量多（~40 对文件），可考虑统一注册模式
- 跟踪 `tree-sitter`（optional dep）的采用情况
