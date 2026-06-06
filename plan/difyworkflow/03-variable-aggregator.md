# 03 Variable Aggregator 节点支持计划

状态：未实现。

## 目标

支持 Dify Variable Aggregator 节点，把互斥分支中的同类变量聚合成统一输出，避免下游节点为每条分支重复一份。

## 当前缺口

- runner 的 DAG 调度能处理分支，但没有“从已执行分支中选择可用变量”的聚合节点。
- 变量池没有“按候选 selector 顺序取第一个存在值”的工具函数。

## 最小实现

1. 新增 `execute_variable_aggregator_node`。
2. 支持节点 type：`variable-aggregator`。
3. 读取 `variables` 或 `groups` 配置。
4. 最小策略：
   - 每个输出字段配置多个 `value_selector`。
   - 按顺序返回第一个存在且非 null 的值。
5. 类型约束先做保守校验：同一聚合输出的非空值类型必须一致。
6. 找不到值时输出 `null`，但节点成功。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
          variables:
            - variable: query
      - id: branch
        data:
          type: if-else
          cases:
            - case_id: a
              conditions:
                - variable_selector: [start, query]
                  comparison_operator: contains
                  value: A
      - id: a
        data:
          type: code
          code_language: python3
          code: |
            def main():
                return {"text": "from-a"}
      - id: b
        data:
          type: code
          code_language: python3
          code: |
            def main():
                return {"text": "from-b"}
      - id: agg
        data:
          type: variable-aggregator
          variables:
            - variable: text
              selectors:
                - [a, text]
                - [b, text]
      - id: answer
        data:
          type: answer
          answer: "{{#agg.text#}}"
    edges:
      - source: start
        sourceHandle: source
        target: branch
      - source: branch
        sourceHandle: a
        target: a
      - source: branch
        sourceHandle: false
        target: b
      - source: a
        sourceHandle: source
        target: agg
      - source: b
        sourceHandle: source
        target: agg
      - source: agg
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat variable-aggregator-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{query:"A"}, max_steps:20}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- `query=A` 时 answer 为 `from-a`。
- `query=B` 时 answer 为 `from-b`。
- 未执行分支不会阻塞聚合节点。
