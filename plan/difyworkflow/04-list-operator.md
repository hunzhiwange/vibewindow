# 04 List Operator 节点支持计划

状态：未实现。

## 目标

支持 Dify List Operator 节点，对数组进行过滤、排序、取首尾元素，常用于文件数组分流。

## 当前缺口

- runner 没有 `list-operator` 分支。
- 模板/selector 目前不支持 `items[0].field` 这类深层访问。

## 最小实现

1. 新增 `execute_list_operator_node`。
2. 支持输入数组 selector。
3. 支持输出：
   - `result`：处理后的数组。
   - `first_record`：首个元素或 null。
   - `last_record`：最后元素或 null。
4. 第一版只支持过滤：
   - 字段等于、包含、in。
   - 字段路径以点分隔，如 `type`、`metadata.mime_type`。
5. 排序先支持字符串/数字升序、降序。
6. 不支持的文件类型判断必须显式报错。

## 验收 YAML

```yaml
workflow:
  graph:
    nodes:
      - id: start
        data:
          type: start
          variables:
            - variable: files
      - id: docs
        data:
          type: list-operator
          input_selector: [start, files]
          filter:
            field: type
            operator: in
            value: ["document"]
      - id: answer
        data:
          type: answer
          answer: "{{#docs.first_record#}}"
    edges:
      - source: start
        sourceHandle: source
        target: docs
      - source: docs
        sourceHandle: source
        target: answer
```

## curl

```bash
jq -n --arg workflow_yaml "$(cat list-operator-demo.yml)" \
  '{workflow_yaml:$workflow_yaml, inputs:{files:[{type:"image", name:"a.png"}, {type:"document", name:"b.pdf"}]}, max_steps:10}' \
| curl -sS -X POST 'http://127.0.0.1:18080/v1/workflow/run' \
  -H 'Content-Type: application/json' \
  --data-binary @-
```

## 验收标准

- `result` 只包含 `type=document` 的元素。
- `first_record.name == "b.pdf"`。
- 空数组返回 `first_record=null`，不报错。
