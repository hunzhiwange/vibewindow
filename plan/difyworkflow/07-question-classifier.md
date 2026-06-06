# 07 Question Classifier 节点支持计划

状态：未实现。

## 目标

支持 Dify Question Classifier 节点，用 LLM 对输入文本做语义分类，并按分类结果选择出边。

## 当前缺口

- runner 没有 `question-classifier` 分支。
- 当前只有 if-else 条件分支，不能由 LLM 分类直接选择 handle。

## 最小实现

1. 新增 `execute_question_classifier_node`。
2. 读取输入 selector。
3. 读取分类列表，兼容字段：
   - `classes`
   - `topics`
   - 每个 class 的 `id`、`name`、`description`
4. 构造 prompt，让 LLM 只返回分类 id。
5. 输出字段：
   - `class_id`
   - `class_name`
   - `class_label`
   - `text`
6. `selected_handle` 设置为分类 id，用于激活对应出边。
7. 分类失败时走 fallback handle：
   - 如果有 `false` 或 `default` 出边，选它。
   - 否则节点失败。

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
      - id: classifier
        data:
          type: question-classifier
          input_selector: [start, query]
          classes:
            - id: order
              name: 订单
              description: 订单查询、订单状态、订单金额
            - id: other
              name: 其他
              description: 不属于订单的问题
      - id: order_answer
        data:
          type: answer
          answer: order
      - id: other_answer
        data:
          type: answer
          answer: other
    edges:
      - source: start
        sourceHandle: source
        target: classifier
      - source: classifier
        sourceHandle: order
        target: order_answer
      - source: classifier
        sourceHandle: other
        target: other_answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat question-classifier-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{query:"查一下昨天订单"}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 订单问题激活 `order` 出边。
- 未识别分类错误可读或走 default。
- 分类 id 不存在时不能随便激活第一条分支。
