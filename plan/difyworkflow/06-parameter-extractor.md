# 06 Parameter Extractor 节点支持计划

状态：未实现。

## 目标

支持 Dify Parameter Extractor 节点，用 LLM 从自然语言中提取结构化参数，供 HTTP/tool/code 节点使用。

## 当前缺口

- runner 没有 `parameter-extractor` 分支。
- LLM 节点已有 provider 调用，但没有按参数 schema 组织 prompt 和解析结果。
- 失败状态变量 `__is_success`、`__reason` 未支持。

## 最小实现

1. 新增 `execute_parameter_extractor_node`。
2. 读取输入 selector，得到待抽取文本。
3. 读取参数定义：
   - name
   - type：string、number、boolean、array、object
   - description
   - required
4. 构造结构化 JSON prompt，优先要求模型只输出 JSON。
5. 解析 LLM 返回：
   - 成功时每个参数作为独立输出。
   - `__is_success=1`
   - `__reason=""`
6. 必填字段缺失时：
   - `__is_success=0`
   - `__reason` 写明缺失字段。
   - 节点本身仍可成功，方便下游 if-else 处理。
7. Dify 节点指定模型不可用时沿用 LLM fallback 策略。

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
      - id: extractor
        data:
          type: parameter-extractor
          input_selector: [start, query]
          parameters:
            - name: city
              type: string
              required: true
              description: 城市名称
            - name: days
              type: number
              required: false
              description: 查询天数
      - id: answer
        data:
          type: answer
          answer: "{{#extractor.city#}}/{{#extractor.days#}}/{{#extractor.__is_success#}}"
    edges:
      - source: start
        sourceHandle: source
        target: extractor
      - source: extractor
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat parameter-extractor-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{query:"查上海未来3天天气"}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 可提取 `city=上海`、`days=3`。
- 缺少必填字段时 `__is_success=0`。
- LLM 返回非 JSON 时错误可读，不能静默吞掉。
