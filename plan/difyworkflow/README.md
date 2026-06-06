# Dify Workflow 节点支持计划

本目录把 Dify Workflow/Chatflow 中当前未完整支持的节点拆成可独立执行的实现计划。每个 `*.md` 都可以单独拿去做一个小任务：先读文件中的“当前缺口”，再按“实现步骤”和“验收 YAML / curl”完成验证。

## 当前仓库状态

入口模块：`crates/vw-agent/src/workflow`

当前 runner 已支持：

- `start`
- `llm`
- `if-else`
- `code`
- `answer`

当前仍需补齐或增强：

- 已支持节点的 Dify 高级能力
- `output` / `end`
- `template`
- `http-request`
- `parameter-extractor`
- `question-classifier`
- `variable-aggregator`
- `list-operator`
- `iteration`
- `knowledge-retrieval`
- `document-extractor`
- `variable-assigner`
- `tool`
- `agent`
- `loop`
- `human-input`

## 建议执行顺序

1. `00-existing-node-gaps.md`
2. `01-output-end.md`
3. `02-template.md`
4. `03-variable-aggregator.md`
5. `04-list-operator.md`
6. `05-http-request.md`
7. `06-parameter-extractor.md`
8. `07-question-classifier.md`
9. `08-iteration.md`
10. `09-knowledge-retrieval.md`
11. `10-document-extractor.md`
12. `11-variable-assigner.md`
13. `12-tool.md`
14. `13-agent.md`
15. `14-loop.md`
16. `15-human-input.md`

## 通用验收方式

每个计划文件都给出最小 YAML 和 curl。默认通过网关执行：

```bash
curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @request.json
```

代码变更完成后，按仓库要求执行：

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## 官方参考

- [Dify LLM](https://docs.dify.ai/en/use-dify/nodes/llm)
- [Dify Knowledge Retrieval](https://docs.dify.ai/en/use-dify/nodes/knowledge-retrieval)
- [Dify Question Classifier](https://docs.dify.ai/en/use-dify/nodes/question-classifier)
- [Dify If-Else](https://docs.dify.ai/en/use-dify/nodes/ifelse)
- [Dify Code](https://docs.dify.ai/en/use-dify/nodes/code)
- [Dify Template](https://docs.dify.ai/en/use-dify/nodes/template)
- [Dify Document Extractor](https://docs.dify.ai/en/use-dify/nodes/doc-extractor)
- [Dify List Operator](https://docs.dify.ai/en/use-dify/nodes/list-operator)
- [Dify Variable Aggregator](https://docs.dify.ai/en/use-dify/nodes/variable-aggregator)
- [Dify Variable Assigner](https://docs.dify.ai/en/use-dify/nodes/variable-assigner)
- [Dify Iteration](https://docs.dify.ai/en/use-dify/nodes/iteration)
- [Dify Parameter Extractor](https://docs.dify.ai/en/use-dify/nodes/parameter-extractor)
- [Dify HTTP Request](https://docs.dify.ai/en/use-dify/nodes/http-request)
- [Dify Tool Node](https://docs.dify.ai/en/use-dify/nodes/tools)
- [Dify Agent](https://docs.dify.ai/en/guides/workflow/node/agent)
- [Dify Human Input](https://docs.dify.ai/en/use-dify/nodes/human-input)
- [Dify Loop](https://legacy-docs.dify.ai/guides/workflow/node/loop)
- [Dify Answer](https://docs.dify.ai/en/use-dify/nodes/answer)
- [Dify Output](https://docs.dify.ai/en/use-dify/nodes/output)
