# 05 HTTP Request 节点支持计划

状态：未实现。当前复杂样例里的 HTTP 是 Python code 节点内嵌 `requests`，不是 Dify 独立 HTTP Request 节点。

## 目标

支持 Dify HTTP Request 节点，通过网关 runner 直接发起外部 HTTP 调用。

## 当前缺口

- runner 没有 `http-request` 分支。
- 没有 HTTP 节点的安全边界：方法、URL、超时、重定向、响应大小、敏感 header 脱敏。
- 当前模板替换不支持 Dify HTTP 节点中的普通 `{{variable_name}}` 写法和深层对象访问。

## 最小实现

1. 新增 `execute_http_request_node`。
2. 使用已有 `reqwest` 依赖。
3. 支持方法：GET、POST、PUT、PATCH、DELETE、HEAD。
4. 支持字段：
   - `url`
   - `method`
   - `headers`
   - `params`
   - `body`
   - `timeout`
   - `authorization` / `auth`
5. 响应输出：
   - `status_code`
   - `headers`
   - `body`
   - `json`，仅当 body 可解析为 JSON。
6. 默认拒绝非 `http://` / `https://`。
7. 默认超时 30s，最大 60s。
8. 响应体限制建议 2MB，超过显式报错。
9. 对 header 中的 token、authorization、skey、password 做脱敏。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
      - id: http
        data:
          type: http-request
          method: GET
          url: "https://httpbin.org/json"
          timeout: 10
      - id: answer
        data:
          type: answer
          answer: "{{#http.status_code#}}"
    edges:
      - source: start
        sourceHandle: source
        target: http
      - source: http
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat http-request-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 正常请求返回 `status_code=200`。
- JSON 响应填充 `json`。
- 非 http/https URL 被拒绝。
- 超时和超大响应可读报错。
