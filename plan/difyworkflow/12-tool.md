# 12 Tool 节点支持计划

状态：未实现。

## 目标

支持 Dify Tool 节点，让 workflow 能调用已注册工具、插件工具或工作流工具。

## 当前缺口

- 当前 workflow runner 和 agent tool loop 是分离的。
- WorkflowRuntime 没有工具注册表。
- Dify Tool 节点需要授权、参数 schema、输入映射和输出 schema。

## 最小实现

1. 定义窄 trait：`WorkflowToolProvider`。
2. 节点解析：
   - tool provider/name/action。
   - credential id 或 auth 配置。
   - input mappings。
3. 执行：
   - 根据节点配置找到工具。
   - 按 schema 组装参数。
   - 调用工具 provider。
4. 输出：
   - `result`
   - `text`
   - `json`
   - `files`
5. 没有工具或未授权时显式失败。
6. 不把 workflow 节点直接耦合到 agent 内部 tool loop；只依赖窄 trait。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
          variables:
            - variable: q
      - id: tool
        data:
          type: tool
          provider: demo
          tool_name: echo
          inputs:
            q:
              value_selector: [start, q]
      - id: answer
        data:
          type: answer
          answer: "{{#tool.text#}}"
    edges:
      - source: start
        sourceHandle: source
        target: tool
      - source: tool
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat tool-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{q:"hello"}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- fake tool provider 能返回 `text=hello`。
- 未授权工具失败且不泄露 token。
- 参数类型不匹配时报错。
