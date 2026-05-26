# Codex 工具映射

技能使用 Claude Code 工具名称。在技能中遇到这些名称时，请使用平台的等价工具：

| 技能引用 | Codex 等价工具 |
|-----------------|------------------|
| `Task` 工具（调度子代理）| `spawn_agent`（参见 [命名代理调度](#named-agent-dispatch)) |
| 多个 `Task` 调用（并行）| 多个 `spawn_agent` 调用 |
| 任务返回结果 | `wait` |
| 任务自动完成 | `close_agent` 释放槽位 |
| `TodoWrite`（任务跟踪）| `update_plan` |
| `Skill` 工具（调用技能）| 技能原生加载 — 只需遵循指令 |
| `Read`、`Write`、`Edit`（文件）| 使用原生文件工具 |
| `Bash`（运行命令）| 使用原生 shell 工具 |

## 子代理调度需要多代理支持

添加到您的 Codex 配置（`~/.codex/config.toml`）：

```toml
[features]
multi_agent = true
```

这为 `dispatching-parallel-agents` 和 `subagent-driven-development` 等技能启用了 `spawn_agent`、`wait` 和 `close_agent`。

## 命名代理调度

Claude Code 技能引用命名代理类型，如 `superpowers:code-reviewer`。
Codex 没有命名代理注册表 — `spawn_agent` 从内置角色（`default`、`explorer`、`worker`）创建通用代理。

当技能要求调度命名代理类型时：

1. 找到代理的提示文件（如 `agents/code-reviewer.md` 或技能的本地提示模板如 `code-quality-reviewer-prompt.md`）
2. 读取提示内容
3. 填充模板占位符（`{BASE_SHA}`、`{WHAT_WAS_IMPLEMENTED}` 等）
4. 使用填充的内容作为 `message` 生成 `worker` 代理

| 技能指令 | Codex 等价方式 |
|-------------------|------------------|
| `Task tool (superpowers:code-reviewer)` | `spawn_agent(agent_type="worker", message=...)` 配合 `code-reviewer.md` 内容 |
| `Task tool (general-purpose)` 配合内联提示 | `spawn_agent(message=...)` 配合相同提示 |

### 消息框架

`message` 参数是用户级输入，不是系统提示。结构化以最大化指令遵循度：

```
你的任务是执行以下操作。请完全遵循以下指令。

<agent-instructions>
[来自代理 .md 文件的已填充提示内容]
</agent-instructions>

现在执行。仅输出符合上述指令格式的结构化响应。
```

- 使用任务委托框架（"你的任务是..."），而不是人设框架（"你是..."）
- 使用 XML 标签包装指令 — 模型将标记块视为权威内容
- 以明确的执行指令结尾，防止对指令进行总结

### 何时可移除此变通方案

此方法补偿了 Codex 插件系统尚不支持 `plugin.json` 中的 `agents` 字段。当 `RawPluginManifest` 获得 `agents` 字段时，插件可以符号链接到 `agents/`（镜像现有的 `skills/` 符号链接），技能可以直接调度命名代理类型。

## 环境检测

创建工作树或完成分支的技能应在继续之前使用只读 git 命令检测其环境：

```bash
GIT_DIR=$(cd "$(git rev-parse --git-dir)" 2>/dev/null && pwd -P)
GIT_COMMON=$(cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)
BRANCH=$(git branch --show-current)
```

- `GIT_DIR != GIT_COMMON` → 已在链接的工作树中（跳过创建）
- `BRANCH` 为空 → 分离 HEAD（无法从沙箱分支/推送/PR）

请参阅 `using-git-worktrees` 步骤 0 和 `finishing-a-development-branch` 步骤 1，了解每个技能如何使用这些信号。

## Codex App 完成

当沙箱阻止分支/推送操作（外部管理工作树中的分离 HEAD）时，代理提交所有工作并通知用户使用 App 的原生控制：

- **"Create branch"** — 命名分支，然后通过 App UI 提交/推送/PR
- **"Hand off to local"** — 将工作转移到用户的本地检出

代理仍可运行测试、暂存文件，并输出建议的分支名称、提交消息和 PR 描述供用户复制。
