# 11 Variable Assigner 节点支持计划

状态：未实现。

## 目标

支持 Dify Variable Assigner 节点，把数据写入会话变量。该节点主要服务 Chatflow 的多轮状态，不是单次 Workflow 的普通临时变量。

## 当前缺口

- 当前 `VariablePool` 只在一次 workflow run 内存在。
- `WorkflowRunRequest` 没有 conversation/session 变量输入输出协议。
- 没有持久化会话变量存储。

## 最小实现

1. 第一版只做“运行内变量赋值”，不做持久会话变量：
   - type 支持 `variable-assigner`。
   - 将变量写入 `conversation.<name>` 或指定 selector。
   - 节点输出写出本次更新结果。
2. 对真正持久化能力显式标记未支持：
   - 如果 DSL 要求持久 conversation variable，但 runtime 未配置 store，报错。
3. 支持操作：
   - overwrite
   - clear
   - set
   - number arithmetic
   - array append/extend/remove_first/remove_last
4. 变更前后值不要记录敏感原文。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
          variables:
            - variable: item
      - id: assign
        data:
          type: variable-assigner
          assignments:
            - variable: favorites
              operation: append
              value_selector: [start, item]
      - id: answer
        data:
          type: answer
          answer: "{{#assign.favorites#}}"
    edges:
      - source: start
        sourceHandle: source
        target: assign
      - source: assign
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat variable-assigner-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{item:"tea"}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- append 能得到 `["tea"]`。
- number arithmetic 正常。
- 持久化未配置时不假装成功。
