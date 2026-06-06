# 15 Human Input 节点支持计划

状态：未实现。

## 目标

支持 Dify Human Input 节点，让 workflow 可以暂停，等待人工表单输入或审批后继续执行。

## 当前缺口

- 当前 `/v1/workflow/run` 是同步 run response，没有“暂停态”和 resume API。
- `WorkflowRunStatus` 目前没有 human input required 状态。
- 没有表单 token、暂停上下文持久化和恢复调度。

## 最小实现

1. 扩展 workflow run 状态：
   - `Running`
   - `Paused`
   - `Succeeded`
   - `Failed`
2. 新增 Human Input 节点执行分支。
3. 节点执行时：
   - 保存当前 graph、VariablePool、executed_nodes、active_nodes。
   - 生成 form token。
   - 返回 paused response，包含表单 schema 和可选 action 分支。
4. 新增 resume API：
   - `POST /v1/workflow/resume`
   - 输入 run_id、form_token、form_values、action。
5. action 映射为 selected_handle，恢复后继续调度。
6. 超时策略先只支持显式配置：
   - fail
   - choose default action
7. 没有持久化 store 时显式报错，不能只存在内存里假装生产可用。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
      - id: review
        data:
          type: human-input
          form:
            fields:
              - name: comment
                type: text
                required: true
          actions:
            - id: approve
              label: Approve
            - id: reject
              label: Reject
      - id: ok
        data:
          type: answer
          answer: "approved {{#review.comment#}}"
      - id: no
        data:
          type: answer
          answer: "rejected"
    edges:
      - source: start
        sourceHandle: source
        target: review
      - source: review
        sourceHandle: approve
        target: ok
      - source: review
        sourceHandle: reject
        target: no
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat human-input-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{}, max_steps:20}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

恢复调用示例：

```bash
curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/resume' \
  -H 'Content-Type: application/json' \
  --data-binary '{"run_id":"RUN_ID","form_token":"TOKEN","action":"approve","form_values":{"comment":"ok"}}'
```

## 验收标准

- 首次 run 返回 paused。
- resume 后按 action 激活对应分支。
- token 不匹配时拒绝恢复。
