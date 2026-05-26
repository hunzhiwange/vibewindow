# Copilot CLI 工具映射

技能使用 Claude Code 工具名称。在技能中遇到这些名称时，请使用平台的等价工具：

| 技能引用 | Copilot CLI 等价工具 |
|-----------------|----------------------|
| `Read`（文件读取）| `view` |
| `Write`（文件创建）| `create` |
| `Edit`（文件编辑）| `edit` |
| `Bash`（运行命令）| `bash` |
| `Grep`（搜索文件内容）| `grep` |
| `Glob`（按名称搜索文件）| `glob` |
| `Skill` 工具（调用技能）| `skill` |
| `WebFetch` | `web_fetch` |
| `Task` 工具（调度子代理）| `task`（参见 [代理类型](#agent-types)) |
| 多个 `Task` 调用（并行）| 多个 `task` 调用 |
| 任务状态/输出 | `read_agent`、`list_agents` |
| `TodoWrite`（任务跟踪）| `sql` 配合内置的 `todos` 表 |
| `WebSearch` | 无等价工具 — 使用 `web_fetch` 配合搜索引擎 URL |
| `EnterPlanMode` / `ExitPlanMode` | 无等价工具 — 保持主会话 |

## 代理类型

Copilot CLI 的 `task` 工具接受 `agent_type` 参数：

| Claude Code 代理 | Copilot CLI 等价类型 |
|-------------------|----------------------|
| `general-purpose` | `"general-purpose"` |
| `Explore` | `"explore"` |
| 命名插件代理（如 `superpowers:code-reviewer`）| 从已安装插件自动发现 |

## 异步 shell 会话

Copilot CLI 支持持久化异步 shell 会话，这些在 Claude Code 中没有直接等价功能：

| 工具 | 用途 |
|------|---------|
| `bash` 配合 `async: true` | 在后台启动长时间运行的命令 |
| `write_bash` | 向运行的异步会话发送输入 |
| `read_bash` | 从异步会话读取输出 |
| `stop_bash` | 终止异步会话 |
| `list_bash` | 列出所有活动的 shell 会话 |

## 额外的 Copilot CLI 工具

| 工具 | 用途 |
|------|---------|
| `store_memory` | 持久化代码库相关信息，供未来会话使用 |
| `report_intent` | 用当前意图更新 UI 状态栏 |
| `sql` | 查询会话的 SQLite 数据库（待办事项、元数据）|
| `fetch_copilot_cli_documentation` | 查询 Copilot CLI 文档 |
| GitHub MCP 工具（`github-mcp-server-*`）| 原生 GitHub API 访问（issues、PR、代码搜索）|
