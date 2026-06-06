# 09 Knowledge Retrieval 节点支持计划

状态：未实现。

## 目标

支持 Dify Knowledge Retrieval 节点，从知识库检索 query 相关片段，并输出 `result` 数组给 LLM 作为上下文。

## 当前缺口

- 当前仓库 workflow runner 没有知识库检索接口。
- gateway 里有 REST API 和数据运行时相关代码，但 workflow 没有抽象出检索 provider。
- Dify 支持多知识库、rerank、metadata filter、top_k、score threshold；这些都未接入。

## 最小实现

1. 先定义窄 trait，例如 `WorkflowKnowledgeProvider`：
   - `retrieve(query, dataset_ids, top_k, score_threshold, metadata_filter) -> Vec<Chunk>`
2. `WorkflowRuntime` 增加可选 knowledge provider。
3. 新增 `execute_knowledge_retrieval_node`。
4. 解析：
   - query selector
   - dataset_ids
   - top_k
   - score_threshold
5. 输出 `result`，每项至少包含：
   - `content`
   - `title`
   - `metadata`
   - `score`
6. 没有 provider 时显式报错，不做假数据。

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
      - id: kb
        data:
          type: knowledge-retrieval
          query_selector: [start, query]
          dataset_ids: ["demo"]
          top_k: 3
      - id: answer
        data:
          type: answer
          answer: "{{#kb.result#}}"
    edges:
      - source: start
        sourceHandle: source
        target: kb
      - source: kb
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat knowledge-retrieval-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{query:"退货规则"}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 使用 fake knowledge provider 的测试可返回固定 chunks。
- 无 provider 时错误包含“knowledge provider 未配置”。
- LLM 节点可引用 `{{#kb.result#}}`。
