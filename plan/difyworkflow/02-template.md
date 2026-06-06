# 02 Template 节点支持计划

状态：未实现。当前只在 answer/llm 文本中做简单 `{{#node.var#}}` 替换，不支持 Dify Template 节点和 Jinja2。

## 目标

支持 Dify Template 节点，把多个上游变量转换成单一文本输出。

## 当前缺口

- runner 没有 `template` / `template-transform` 分支。
- 当前模板系统不是 Jinja2，只支持 Dify selector 替换。
- 不支持数组循环、条件、对象字段访问。

## 最小实现

1. 新增 `execute_template_node`。
2. 先支持保守子集：
   - `template` 字段。
   - `variables` 中的 `variable` + `value_selector`。
   - 输出字段固定为 `output` 和 `result`。
3. 模板引擎优先用 Rust 侧轻量实现：
   - 普通 `{{ variable }}` 替换。
   - 变量值为对象/数组时 JSON 序列化。
4. 对完整 Jinja2 语法先显式报错或走 Python `jinja2` 子进程，不能静默错渲染。
5. 后续再补 `{% for %}`、`{% if %}`。

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
            - variable: count
      - id: tpl
        data:
          type: template
          template: "客户 {{ name }} 有 {{ count }} 条订单"
          variables:
            - variable: name
              value_selector: [start, name]
            - variable: count
              value_selector: [start, count]
      - id: answer
        data:
          type: answer
          answer: "{{#tpl.output#}}"
    edges:
      - source: start
        sourceHandle: source
        target: tpl
      - source: tpl
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat template-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{name:"Alice", count:3}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- `answer == "客户 Alice 有 3 条订单"`。
- 遇到未支持 Jinja2 控制语法时错误可读。
