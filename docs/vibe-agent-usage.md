# Vibe Agent 使用指南

Vibe Agent 是 Vibe Window 的核心 AI 代理组件，提供交互式聊天、任务自动化、网关服务等功能。

## 1. 编译与安装

首先，确保你已经安装了 Rust 开发环境。

```bash
# 编译 release 版本
cargo build --bin vibe-agent --release

# 或者直接运行开发版本
cargo run --bin vibe-agent -- --help
```

编译完成后，二进制文件位于 `target/release/vibe-agent` (或 `target/debug/vibe-agent`)。

## 2. 快速开始

### 启动交互式 Agent

最简单的使用方式是启动交互式命令行 Agent：

```bash
cargo run --bin vibe-agent -- agent
```

你也可以直接发送单条消息：

```bash
cargo run --bin vibe-agent -- agent -m "你好，请介绍一下你自己"
```

### 指定模型与提供商

你可以通过参数指定使用的 AI 提供商和模型：

```bash
cargo run --bin vibe-agent -- agent --provider openai --model gpt-4o
```

## 3. 核心模式

Vibe Agent 支持多种运行模式：

### Agent 模式 (`agent`)
交互式命令行聊天界面，支持工具调用、上下文记忆等。

### Gateway 模式 (`gateway`)
启动 HTTP/WebSocket 网关，用于接收 Webhook 事件和外部连接。

```bash
# 启动网关（默认端口由配置文件决定）
cargo run --bin vibe-agent -- gateway

# 指定端口
cargo run --bin vibe-agent -- gateway --port 8080
```

### Daemon 模式 (`daemon`)
启动长运行的守护进程，包含网关、心跳检测、定时任务调度器等。这是生产环境推荐的运行方式。

```bash
cargo run --bin vibe-agent -- daemon
cargo run --bin vibe-agent --features channel-lark -- daemon
```

## 4. 常用命令

### 查看系统状态 (`status`)
查看当前配置、服务状态、内存后端等信息。

```bash
cargo run --bin vibe-agent -- status
```

### 系统诊断 (`doctor`)
运行诊断工具，检查模型连接、运行时追踪等。

```bash
cargo run --bin vibe-agent -- doctor
```

### 配置管理 (`config`)
查看当前配置的 Schema 或导出配置。

```bash
cargo run --bin vibe-agent -- config schema
```

### 定时任务 (`cron`)
管理定时任务。

```bash
# 列出任务
cargo run --bin vibe-agent -- cron list

# 添加任务
cargo run --bin vibe-agent -- cron add "0 9 * * *" "echo 'Good Morning'"
```

### 项目任务 (`task`)
在项目目录下创建与读取任务看板数据（存储于 `.vibewindow/tasks`）。

```bash
# 当前目录项目：创建任务
cargo run --bin vibe-agent -- task create --prompt "检查 OAuth 回调并修复"

# 指定项目目录：创建任务
cargo run --bin vibe-agent -- task --project-dir /path/to/repo create --description "补充测试" --priority 3 --subtask "单元测试" --subtask "集成测试"

# 读取任务列表
cargo run --bin vibe-agent -- task read --status pending --limit 20

# 按任务 ID 读取
cargo run --bin vibe-agent -- task read --id T20260317.0001
```

## 5. 配置

Vibe Agent 的配置文件通常位于 `~/.vibewindow/vibewindow.json` (macOS/Linux) 或 `%APPDATA%/vibewindow/vibewindow.json` (Windows)。

你可以通过 `status` 命令查看当前加载的配置文件路径。

## 6. 常见问题

**Q: 启动时报错 "folder 'web/dist/' does not exist"？**
A: 这是因为 `RustEmbed` 需要在编译时嵌入前端静态文件。确保 `web/dist/` 目录存在且包含 `index.html`。如果不需要前端界面，可以创建一个空的占位文件（我们已经自动修复了这个问题）。

**Q: 如何查看详细日志？**
A: 设置 `RUST_LOG` 环境变量：

```bash
RUST_LOG=debug cargo run --bin vibe-agent -- agent
```
