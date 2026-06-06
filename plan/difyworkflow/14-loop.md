# 14 Loop 节点支持计划

状态：未实现。

## 目标

支持 Dify Loop 节点，在满足终止条件前重复执行内部子图。Loop 与 Iteration 不同：Iteration 遍历已有数组，Loop 按条件循环直到结束或达到最大次数。

## 当前缺口

- 当前 runner 没有子图循环执行器。
- 没有 loop 局部变量，如当前循环次数、上轮输出。
- 没有最大循环次数保护。

## 最小实现

1. 新增 `execute_loop_node`。
2. 解析内部子图，复用 iteration 的子图执行器。
3. 注入局部变量：
   - `index`
   - `loop_count`
   - `last_output`
4. 支持终止条件：
   - 基于 selector 的 if-else 条件。
   - 基于内部节点输出字段。
5. 强制最大次数：
   - 默认 100。
   - DSL 配置不能超过安全上限。
6. 输出：
   - `result`
   - `last_output`
   - `iterations`
7. 超过最大次数时显式失败。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
      - id: loop
        data:
          type: loop
          max_count: 3
          output_selector: [step, value]
          graph:
            nodes:
              - id: step
                data:
                  type: code
                  code_language: python3
                  variables:
                    - variable: index
                      value_selector: [loop, index]
                  code: |
                    def main(index):
                        return {"value": index + 1}
            edges: []
      - id: answer
        data:
          type: answer
          answer: "{{#loop.last_output#}}/{{#loop.iterations#}}"
    edges:
      - source: start
        sourceHandle: source
        target: loop
      - source: loop
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat loop-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{}, max_steps:20}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 最大次数生效。
- 输出最后一轮结果和循环次数。
- 无限循环不会发生。
