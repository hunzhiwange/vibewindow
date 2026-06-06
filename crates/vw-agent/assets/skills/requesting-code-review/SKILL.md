---
name: requesting-code-review
description: 在完成任务、实现主要功能或合并前使用，以验证工作是否符合要求
---

# 请求代码审查

派遣 superpowers:code-reviewer 子代理，在问题扩散之前将其捕获。审查者获得的是精心构建的评估上下文——绝非你当前会话的历史记录。这使审查者专注于工作成果本身，而非你的思考过程，同时也为你自己的后续工作保留了上下文空间。

**核心原则：** 尽早审查，频繁审查。

## 何时请求审查

**必须审查：**
- 子代理驱动开发中的每个任务完成后
- 完成主要功能后
- 合并到主分支之前

**建议审查：**
- 遇到困难时（获取新视角）
- 重构前（基线检查）
- 修复复杂 bug 后

## 如何请求审查

**1. 获取 git SHA：**
```bash
BASE_SHA=$(git rev-parse HEAD~1)  # 或 origin/main
HEAD_SHA=$(git rev-parse HEAD)
```

**2. 派遣 code-reviewer 子代理：**

使用 Task 工具，类型为 superpowers:code-reviewer，填写 `code-reviewer.md` 中的模板

**模板占位符：**
- `{WHAT_WAS_IMPLEMENTED}` - 你刚刚构建的内容
- `{PLAN_OR_REQUIREMENTS}` - 预期功能描述
- `{BASE_SHA}` - 起始提交
- `{HEAD_SHA}` - 结束提交
- `{DESCRIPTION}` - 简要摘要

**3. 处理反馈：**
- 立即修复严重问题
- 在继续之前修复重要问题
- 记录次要问题以备后续处理
- 如果审查者判断有误，提出反驳（附上理由）

## 示例

```
[刚刚完成任务 2：添加验证功能]

你：在继续之前，让我请求代码审查。

BASE_SHA=$(git log --oneline | grep "Task 1" | head -1 | awk '{print $1}')
HEAD_SHA=$(git rev-parse HEAD)

[派遣 superpowers:code-reviewer 子代理]
  WHAT_WAS_IMPLEMENTED: 对话索引的验证和修复功能
  PLAN_OR_REQUIREMENTS: docs/superpowers/plans/deployment-plan.md 中的任务 2
  BASE_SHA: a7981ec
  HEAD_SHA: 3df7661
  DESCRIPTION: 添加了 verifyIndex() 和 repairIndex()，支持 4 种问题类型

[子代理返回结果]:
  优点: 架构清晰，测试真实有效
  问题:
    重要: 缺少进度指示器
    次要: 报告间隔使用了魔法数字 (100)
  评估: 可以继续

你: [修复进度指示器]
[继续执行任务 3]
```

## 与工作流的集成

**子代理驱动开发：**
- 每个任务后都进行审查
- 在问题累积之前发现它们
- 在进入下一个任务前完成修复

**执行计划：**
- 每批任务（3个）后进行审查
- 获取反馈，应用修复，继续推进

**日常开发：**
- 合并前审查
- 遇到困难时审查

## 红线

**绝不：**
- 因为"很简单"就跳过审查
- 忽视严重问题
- 在重要问题未修复的情况下继续推进
- 对合理的技术反馈进行争论

**如果审查者判断有误：**
- 用技术理由进行反驳
- 展示证明其有效的代码/测试
- 请求进一步澄清

模板文件位于：requesting-code-review/code-reviewer.md
