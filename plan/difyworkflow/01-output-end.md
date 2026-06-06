# 01 Output / End 节点支持计划

状态：未实现。当前 runner 遇到 `output` 或旧 DSL 中的 `end` 会报“不支持的 workflow 节点类型”。

## 目标

支持 Dify Workflow 的输出节点。Chatflow 用 `answer` 返回内容，Workflow 用 `output` / `end` 定义 API 返回字段。

## 当前缺口

- `execute_node` 没有 `output` / `end` 分支。
- `WorkflowRunResponse.outputs` 目前取最后一个节点输出，不会按 Dify 输出节点配置重命名字段。

## 最小实现

1. 新增 `execute_output_node`。
2. 兼容节点类型：`output`、`end`。
3. 从节点 data 中读取输出变量配置，常见字段可能是 `outputs`、`output_variables` 或 `variables`。
4. 每个输出项支持：
   - `variable` / `key` / `name` 作为返回字段名。
   - `value_selector` 作为来源。
5. 节点输出写入变量池，最终 `WorkflowRunResponse.outputs` 使用该节点 outputs。
6. 如果输出配置为空，显式失败，错误写清楚。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
          variables:
            - variable: name
      - id: out
        data:
          type: output
          outputs:
            - variable: greeting
              value_selector: [start, name]
    edges:
      - source: start
        sourceHandle: source
        target: out
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat output-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{name:"Alice"}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 返回 `status=succeeded`。
- `outputs.greeting == "Alice"`。
- `output` 与 `end` 两种 type 都有测试覆盖。
