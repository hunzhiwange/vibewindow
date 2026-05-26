---
name: using-superpowers
description: 在开始任何对话时使用 - 建立如何查找和使用技能，需要在任何响应（包括澄清问题）之前调用 Skill 工具
---

<SUBAGENT-STOP>
如果您被作为子代理派发来执行特定任务，请跳过此技能。
</SUBAGENT-STOP>

<EXTREMELY-IMPORTANT>
如果您认为即使只有 1% 的可能性某个技能可能适用于您正在做的事情，您绝对必须调用该技能。

如果技能适用于您的任务，您没有选择。您必须使用它。

这是不可商量的。这不是可选的。您无法合理化自己不使用它。
</EXTREMELY-IMPORTANT>

## 指令优先级

Superpowers 技能覆盖默认系统提示行为，但**用户指令始终具有最高优先级**：

1. **用户的明确指令**（CLAUDE.md、GEMINI.md、AGENTS.md、直接请求）— 最高优先级
2. **Superpowers 技能** — 在冲突时覆盖默认系统行为
3. **默认系统提示** — 最低优先级

如果 CLAUDE.md、GEMINI.md 或 AGENTS.md 说"不要使用 TDD"，而技能说"始终使用 TDD"，请遵循用户指令。用户有控制权。

## 如何访问技能

**在 Claude Code 中：** 使用 `Skill` 工具。调用技能时，其内容会被加载并呈现给您 — 直接遵循。切勿对技能文件使用 Read 工具。

**在 Copilot CLI 中：** 使用 `skill` 工具。技能从已安装的插件自动发现。`skill` 工具与 Claude Code 的 `Skill` 工具工作方式相同。

**在 Gemini CLI 中：** 技能通过 `activate_skill` 工具激活。Gemini 在会话开始时加载技能元数据，并在需要时激活完整内容。

**在其他环境中：** 查看平台文档了解如何加载技能。

## 平台适配

技能使用 Claude Code 工具名称。非 CC 平台：请参阅 `references/copilot-tools.md`（Copilot CLI）、`references/codex-tools.md`（Codex）了解等价工具。Gemini CLI 用户通过 GEMINI.md 自动加载工具映射。

# 使用技能

## 规则

**在任何响应或操作之前调用相关或请求的技能。** 即使只有 1% 的可能性技能可能适用，您也应该调用技能进行检查。如果调用的技能被证明不适合该情况，您不需要使用它。

```dot
digraph skill_flow {
    "User message received" [shape=doublecircle];
    "About to EnterPlanMode?" [shape=doublecircle];
    "Already brainstormed?" [shape=diamond];
    "Invoke brainstorming skill" [shape=box];
    "Might any skill apply?" [shape=diamond];
    "Invoke Skill tool" [shape=box];
    "Announce: 'Using [skill] to [purpose]'" [shape=box];
    "Has checklist?" [shape=diamond];
    "Create TodoWrite todo per item" [shape=box];
    "Follow skill exactly" [shape=box];
    "Respond (including clarifications)" [shape=doublecircle];

    "About to EnterPlanMode?" -> "Already brainstormed?";
    "Already brainstormed?" -> "Invoke brainstorming skill" [label="no"];
    "Already brainstormed?" -> "Might any skill apply?" [label="yes"];
    "Invoke brainstorming skill" -> "Might any skill apply?";

    "User message received" -> "Might any skill apply?";
    "Might any skill apply?" -> "Invoke Skill tool" [label="yes, even 1%"];
    "Might any skill apply?" -> "Respond (including clarifications)" [label="definitely not"];
    "Invoke Skill tool" -> "Announce: 'Using [skill] to [purpose]'";
    "Announce: 'Using [skill] to [purpose]'" -> "Has checklist?";
    "Has checklist?" -> "Create TodoWrite todo per item" [label="yes"];
    "Has checklist?" -> "Follow skill exactly" [label="no"];
    "Create TodoWrite todo per item" -> "Follow skill exactly";
}
```

## 警示信号

这些想法意味着停止 — 您在合理化：

| 想法 | 现实 |
|---------|---------|
| "这只是一个简单问题" | 问题也是任务。检查技能。 |
| "我需要更多上下文" | 技能检查在澄清问题之前。 |
| "让我先探索代码库" | 技能告诉您如何探索。先检查。 |
| "我可以快速检查 git/文件" | 文件缺少对话上下文。检查技能。 |
| "让我先收集信息" | 技能告诉您如何收集信息。 |
| "这不需要正式技能" | 如果技能存在，就使用它。 |
| "我记得这个技能" | 技能会演变。阅读当前版本。 |
| "这不算是任务" | 行动 = 任务。检查技能。 |
| "这个技能是杀鸡用牛刀" | 简单的事情会变得复杂。使用它。 |
| "我就先做这一件事" | 在做任何事情之前检查。 |
| "这感觉很有成效" | 无纪律的行动浪费时间。技能可以防止这种情况。 |
| "我知道这意味着什么" | 了解概念 ≠ 使用技能。调用它。 |

## 技能优先级

当多个技能可能适用时，使用此顺序：

1. **流程技能优先**（头脑风暴、调试）- 这些确定如何处理任务
2. **实现技能其次**（前端设计、mcp-builder）- 这些指导执行

"让我们构建 X" → 首先头脑风暴，然后实现技能。
"修复这个错误" → 首先调试，然后特定领域技能。

## 技能类型

**严格型**（TDD、调试）：严格遵循。不要偏离纪律。

**灵活型**（模式）：根据上下文调整原则。

技能本身会告诉您是哪种类型。

## 用户指令

指令说明做什么，而不是怎么做。"添加 X" 或"修复 Y"并不意味着跳过工作流。
