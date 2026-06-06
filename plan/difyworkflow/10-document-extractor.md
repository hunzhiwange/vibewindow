# 10 Document Extractor 节点支持计划

状态：未实现。

## 目标

支持 Dify Document Extractor 节点，将上传文件或文件数组转换为文本，供 LLM/Template/Code 使用。

## 当前缺口

- workflow 输入目前只处理 JSON inputs，没有文件对象解析和下载/读取流程。
- 没有文档解析 provider。
- 不支持 DOCX/PDF/CSV 等格式转换。

## 最小实现

1. 定义窄 trait：`WorkflowDocumentExtractor`。
2. 输入支持：
   - 单文件对象。
   - 文件数组。
3. 文件对象字段至少支持：
   - `name`
   - `mime_type`
   - `path` 或 `url`
   - `size`
4. 第一版只支持安全本地路径或已上传文件引用，不直接开放任意路径读取。
5. 文档类型分层：
   - TXT/Markdown/JSON/YAML：内置解析。
   - PDF/DOCX/CSV：通过 provider 或后续依赖实现。
6. 输出：
   - `text`
   - `result`
   - `files`，保留原文件元数据。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
          variables:
            - variable: file
      - id: doc
        data:
          type: document-extractor
          input_selector: [start, file]
      - id: answer
        data:
          type: answer
          answer: "{{#doc.text#}}"
    edges:
      - source: start
        sourceHandle: source
        target: doc
      - source: doc
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat document-extractor-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{file:{name:"note.txt", mime_type:"text/plain", text:"hello doc"}}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- 内联 text 文件返回 `text=hello doc`。
- 不支持格式显式报错。
- 不允许任意读取敏感本地路径。
