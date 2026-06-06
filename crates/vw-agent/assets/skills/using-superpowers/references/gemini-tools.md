# Gemini CLI 工具映射

技能使用 Claude Code 工具名称。在技能中遇到这些名称时，请使用平台的等价工具：

| 技能引用 | Gemini CLI 等价工具 |
|-----------------|----------------------|
| `Read`（文件读取）| `read_file` |
| `Write`（文件创建）| `write_file` |
| `Edit`（文件编辑）| `replace` |
| `Bash`（运行命令）| `run_shell_command` |
| `Grep`（搜索文件内容）| `grep_search` |
| `Glob`（按名称搜索文件）| `glob` |
| `TodoWrite`（任务跟踪）| `write_todos` |
| `Skill` 工具（调用技能）| `activate_skill` |
| `WebSearch` | `google_web_search` |
| `WebFetch` | `web_fetch` |
| `Task` 工具（调度子代理）| 无等价工具 — Gemini CLI 不支持子代理 |

## 无子代理支持

Gemini CLI 没有 Claude Code `Task` 工具的等价工具。依赖子代理调度的技能（`subagent-driven-development`、`dispatching-parallel-agents`）将通过 `executing-plans` 回退到单会话执行。

## 额外的 Gemini CLI 工具

这些工具在 Gemini CLI 中可用，但在 Claude Code 中没有等价工具：

| 工具 | 用途 |
|------|---------|
| `list_directory` | 列出文件和子目录 |
| `save_memory` | 将信息持久化到 GEMINI.md 以供跨会话使用 |
| `ask_user` | 请求用户结构化输入 |
| `tracker_create_task` | 丰富的任务管理（创建、更新、列出、可视化）|
| `enter_plan_mode` / `exit_plan_mode` | 在进行更改之前切换到只读研究模式 |
