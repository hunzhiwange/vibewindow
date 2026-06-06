# 00 已支持节点的高级能力补齐计划

状态：部分支持。当前 runner 已有 `start`、`llm`、`if-else`、`code`、`answer`，但只覆盖了基础行为。

## 目标

在继续补新节点前，先把已支持节点中影响 Dify DSL 兼容性的高级能力列清楚，后续按需拆小任务实现。

## Start

当前已支持普通输入变量。

待补：

- 文件变量。
- sys 变量扩展，如 user、conversation、app metadata。
- 环境变量读取，尤其是密钥型变量必须默认脱敏。

## LLM

当前已支持 prompt template、模型调用、基础输出别名和模型 fallback。

待补：

- Dify structured output / JSON schema。
- Context 变量，尤其是知识检索结果作为上下文。
- Vision / 文件输入。
- Jinja2 prompt mode。
- retry / backoff / error fallback。
- token usage 透传。
- streaming chunk 事件。

## If-Else

当前已支持常见比较运算和分支 handle。

待补：

- Dify 新 DSL 字段名兼容。
- 深层字段路径比较，如 `a.b[0].c`。
- 日期/时间类型比较。
- 更完整的 array/object 条件判断。

## Code

当前已支持 Python / JavaScript，本地执行，且已补 `requests` 最小兼容。

待补：

- Dify sandbox 的输出限制：字符串长度、数字范围、对象深度。
- 重试和失败分支。
- 依赖清单配置。
- 网络访问策略，不应无限扩大能力。
- JS 版本和 Python 版本的参数调用形式统一。

## Answer

当前已支持文本 answer 和 `answer/text` 输出别名。

待补：

- 多模态输出。
- Chatflow 中多 answer 节点的流式语义。
- 富文本/文件输出。
- answer 节点不可单独 step-run 的行为兼容。

## 验收标准

- 每个高级能力都应有独立小测试。
- 未实现能力必须显式报错，不能返回空字符串假装成功。
- 涉及敏感变量时，节点输入输出和错误都要脱敏。
