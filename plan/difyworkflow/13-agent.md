# 13 Agent 节点支持计划

状态：未实现。

## 目标

支持 Dify Agent 节点，让 LLM 在节点内部自主选择并调用工具，完成多步任务。

## 当前缺口

- 当前 workflow LLM 节点只调用 provider，不带 tools。
- 仓库已有 agent tool loop，但 workflow 节点没有接入。
- Dify Agent 节点需要策略、最大迭代次数、工具列表、日志、最终答案等输出。

## 最小实现

1. 不在 workflow runner 里重写 agent loop。
2. 定义窄接口或复用现有 agent query engine：
   - 输入 prompt/messages。
   - 工具白名单。
   - max_iterations。
   - model/temperature。
3. 新增 `execute_agent_node`。
4. 支持策略：
   - function_calling：如果 provider/tools 支持。
   - react：第一版可显式不支持或通过现有 agent loop 实现。
5. 输出：
   - `answer`
   - `text`
   - `tool_outputs`
   - `reasoning`
   - `iterations`
   - `success`
6. 超过最大迭代次数时节点失败，错误可读。
7. 工具授权和敏感参数必须脱敏。

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
      - id: agent
        data:
          type: agent
          strategy: function_calling
          max_iterations: 3
          prompt_template:
            - role: user
              text: "{{#start.query#}}"
          tools:
            - provider: demo
              tool_name: echo
      - id: answer
        data:
          type: answer
          answer: "{{#agent.answer#}}"
    edges:
      - source: start
        sourceHandle: source
        target: agent
      - source: agent
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat agent-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{query:"echo hello"}, max_steps:20}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- fake agent 能调用 fake tool 并返回最终 answer。
- max_iterations 生效。
- 未配置 tool provider 时不假装成功。
