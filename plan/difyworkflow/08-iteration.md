# 08 Iteration 节点支持计划

状态：未实现。

## 目标

支持 Dify Iteration 节点，对数组中每个元素执行内部子图，并收集结果数组。

## 当前缺口

- 当前 graph 是平面 DAG 调度，没有子图执行器。
- 变量池没有 iteration 局部作用域，如 `items`、`index`。
- answer 节点会更新全局 last_answer，iteration 内需要区分流式中间输出和最终输出。

## 最小实现

1. 新增 `execute_iteration_node`。
2. 从节点 data 读取数组输入 selector。
3. 解析内部子图配置，常见字段可能是 `graph`、`children`、`sub_graph`。
4. 顺序模式优先：
   - 每个 item 创建局部 VariablePool。
   - 注入 `[iteration_node_id, "item"]`、`[iteration_node_id, "index"]`。
   - 执行内部节点。
   - 收集指定输出 selector。
5. 输出：
   - `output` / `result`：数组。
6. 错误策略：
   - `terminate`
   - `continue_on_error` 输出 null。
   - `remove_failed` 跳过失败项。
7. 并行模式先显式报错，后续再做，避免调度复杂度一次膨胀。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
          variables:
            - variable: nums
      - id: iter
        data:
          type: iteration
          input_selector: [start, nums]
          output_selector: [double, value]
          graph:
            nodes:
              - id: double
                data:
                  type: code
                  code_language: python3
                  variables:
                    - variable: item
                      value_selector: [iter, item]
                  code: |
                    def main(item):
                        return {"value": item * 2}
            edges: []
      - id: answer
        data:
          type: answer
          answer: "{{#iter.result#}}"
    edges:
      - source: start
        sourceHandle: source
        target: iter
      - source: iter
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat iteration-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{nums:[1,2,3]}, max_steps:20}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 输出 `[2,4,6]`。
- 空数组输出 `[]`。
- 子图失败时按配置处理。
