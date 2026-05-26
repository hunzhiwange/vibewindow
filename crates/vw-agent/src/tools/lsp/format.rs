//! LSP 原始响应的结构化解析和面向模型的格式化。
//!
//! LSP server 返回的 JSON 形态会随操作不同而变化。本模块把这些响应归一为
//! 文本摘要、计数信息和可序列化 payload，供工具结果与 UI 渲染共同使用。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// LSP 位置类结果中的单个条目。
///
/// 行列号使用 1-based 值，便于直接展示给用户；`absolute_path` 保留完整路径，
/// `path` 则优先显示相对工作区路径。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LspLocationItem {
    pub path: String,
    pub absolute_path: String,
    pub uri: String,
    pub line: u32,
    pub character: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
}

/// 文档符号或工作区符号的展示条目。
///
/// `depth` 用于保留嵌套符号层级，格式化文本可据此缩进。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LspDocumentSymbolItem {
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub path: String,
    pub absolute_path: String,
    pub line: u32,
    pub character: u32,
    pub depth: usize,
}

/// 调用层级中一次具体调用发生的位置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LspCallSite {
    pub line: u32,
    pub character: u32,
}

/// 调用层级条目。
///
/// `call_sites` 只在入向/出向调用响应中填充，用于展示调用发生的多个位置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LspCallHierarchyItem {
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub path: String,
    pub absolute_path: String,
    pub uri: String,
    pub line: u32,
    pub character: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub call_sites: Vec<LspCallSite>,
}

/// LSP 工具返回给调用方的结构化 payload。
///
/// 不同变体对应不同 LSP 操作；未知或不支持的操作使用 `Message` 明确表达。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum LspPayload {
    Locations {
        items: Vec<LspLocationItem>,
    },
    Hover {
        contents: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        line: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        character: Option<u32>,
    },
    DocumentSymbols {
        items: Vec<LspDocumentSymbolItem>,
    },
    CallHierarchyItems {
        items: Vec<LspCallHierarchyItem>,
    },
    CallHierarchyCalls {
        items: Vec<LspCallHierarchyItem>,
    },
    Message {
        message: String,
    },
}

/// 已格式化的 LSP 操作结果。
///
/// `result_text` 面向模型阅读，`summary` 面向 UI 概览，计数字段用于快速判断结果规模，
/// `payload` 保留结构化数据。
#[derive(Debug, Clone)]
pub(crate) struct LspFormattedResponse {
    pub result_text: String,
    pub summary: String,
    pub result_count: usize,
    pub file_count: usize,
    pub payload: LspPayload,
}

/// 将原始 LSP JSON 响应格式化为统一结果。
///
/// `operation` 必须是工具层支持的操作名；`raw` 是语言服务器返回的原始 `result`；
/// `workspace_root` 用于把绝对路径压缩为相对显示路径。未知操作返回 `Message`，
/// 不会抛出错误。
pub(crate) fn format_operation_result(
    operation: &str,
    raw: &Value,
    workspace_root: &Path,
) -> LspFormattedResponse {
    match operation {
        "goToDefinition" => {
            format_locations_response("定义", "未找到定义。", parse_locations(raw, workspace_root))
        }
        "findReferences" => {
            format_locations_response("引用", "未找到引用。", parse_locations(raw, workspace_root))
        }
        "goToImplementation" => {
            format_locations_response("实现", "未找到实现。", parse_locations(raw, workspace_root))
        }
        "hover" => format_hover_response(raw),
        "documentSymbol" => format_document_symbols_response(raw, workspace_root),
        "workspaceSymbol" => format_workspace_symbols_response(raw, workspace_root),
        "prepareCallHierarchy" => format_call_hierarchy_items_response(raw, workspace_root),
        "incomingCalls" => format_call_hierarchy_calls_response("入向调用", raw, workspace_root),
        "outgoingCalls" => format_call_hierarchy_calls_response("出向调用", raw, workspace_root),
        _ => LspFormattedResponse {
            result_text: "不支持的 LSP 操作。".to_string(),
            summary: "LSP 操作不受支持".to_string(),
            result_count: 0,
            file_count: 0,
            payload: LspPayload::Message { message: "不支持的 LSP 操作。".to_string() },
        },
    }
}

fn format_locations_response(
    noun: &str,
    empty_message: &str,
    items: Vec<LspLocationItem>,
) -> LspFormattedResponse {
    if items.is_empty() {
        return LspFormattedResponse {
            result_text: empty_message.to_string(),
            summary: empty_message.to_string(),
            result_count: 0,
            file_count: 0,
            payload: LspPayload::Locations { items },
        };
    }

    let file_count = unique_file_count(items.iter().map(|item| item.absolute_path.as_str()));
    let mut lines = vec![format!("找到 {} 个{}，涉及 {} 个文件：", items.len(), noun, file_count)];
    for item in &items {
        lines.push(format!("path: {}", item.path));
        lines.push(format!("  - {}:{}", item.line, item.character));
    }

    LspFormattedResponse {
        result_text: lines.join("\n"),
        summary: format!("找到 {} 个{}，涉及 {} 个文件", items.len(), noun, file_count),
        result_count: items.len(),
        file_count,
        payload: LspPayload::Locations { items },
    }
}

fn format_hover_response(raw: &Value) -> LspFormattedResponse {
    let contents = raw
        .get("contents")
        .map(extract_markup_text)
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| "没有可用的悬停信息。".to_string());
    let line = raw
        .get("range")
        .and_then(|range| range.get("start"))
        .and_then(|start| start.get("line"))
        .and_then(Value::as_u64)
        .map(|line| line as u32 + 1);
    let character = raw
        .get("range")
        .and_then(|range| range.get("start"))
        .and_then(|start| start.get("character"))
        .and_then(Value::as_u64)
        .map(|character| character as u32 + 1);
    let has_content = contents != "没有可用的悬停信息。";

    LspFormattedResponse {
        result_text: contents.clone(),
        summary: if has_content { "悬停信息可用".to_string() } else { contents.clone() },
        result_count: usize::from(has_content),
        file_count: usize::from(has_content),
        payload: LspPayload::Hover { contents, line, character },
    }
}

fn format_document_symbols_response(raw: &Value, workspace_root: &Path) -> LspFormattedResponse {
    let mut items = Vec::new();
    if let Some(symbols) = raw.as_array() {
        // 部分 server 对 documentSymbol 返回 workspace symbol 形态，需兼容 location 字段。
        if symbols.first().is_some_and(|item| item.get("location").is_some()) {
            for symbol in symbols {
                if let Some(item) = parse_workspace_symbol(symbol, workspace_root) {
                    items.push(LspDocumentSymbolItem {
                        name: item.name.unwrap_or_else(|| "<unknown>".to_string()),
                        kind: item.kind.unwrap_or_else(|| "Unknown".to_string()),
                        detail: item.detail.or(item.container_name),
                        path: item.path,
                        absolute_path: item.absolute_path,
                        line: item.line,
                        character: item.character,
                        depth: 0,
                    });
                }
            }
        } else {
            for symbol in symbols {
                flatten_document_symbol(symbol, workspace_root, 0, &mut items);
            }
        }
    }

    if items.is_empty() {
        return LspFormattedResponse {
            result_text: "文档中未找到符号。".to_string(),
            summary: "文档中未找到符号".to_string(),
            result_count: 0,
            file_count: 0,
            payload: LspPayload::DocumentSymbols { items },
        };
    }

    let mut lines = vec!["文档符号：".to_string()];
    for item in &items {
        let indent = "  ".repeat(item.depth);
        let detail = item.detail.as_deref().filter(|detail| !detail.trim().is_empty());
        match detail {
            Some(detail) => lines.push(format!(
                "{}{} ({}) {} - {}:{}",
                indent, item.name, item.kind, detail, item.path, item.line
            )),
            None => lines.push(format!(
                "{}{} ({}) - {}:{}",
                indent, item.name, item.kind, item.path, item.line
            )),
        }
    }

    LspFormattedResponse {
        result_text: lines.join("\n"),
        summary: format!("找到 {} 个文档符号", items.len()),
        result_count: items.len(),
        file_count: unique_file_count(items.iter().map(|item| item.absolute_path.as_str())),
        payload: LspPayload::DocumentSymbols { items },
    }
}

fn format_workspace_symbols_response(raw: &Value, workspace_root: &Path) -> LspFormattedResponse {
    let items = raw
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| parse_workspace_symbol(item, workspace_root))
        .collect::<Vec<_>>();

    format_locations_response("工作区符号", "工作区中未找到符号。", items)
}

fn format_call_hierarchy_items_response(
    raw: &Value,
    workspace_root: &Path,
) -> LspFormattedResponse {
    let items = raw
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| parse_call_hierarchy_item(item, workspace_root))
        .collect::<Vec<_>>();

    if items.is_empty() {
        return LspFormattedResponse {
            result_text: "当前位置没有可用的调用层级项。".to_string(),
            summary: "没有可用的调用层级项".to_string(),
            result_count: 0,
            file_count: 0,
            payload: LspPayload::CallHierarchyItems { items },
        };
    }

    let mut lines = vec![format!("找到 {} 个调用层级项：", items.len())];
    for item in &items {
        lines.push(format!("- {} ({}) - {}:{}", item.name, item.kind, item.path, item.line));
    }

    LspFormattedResponse {
        result_text: lines.join("\n"),
        summary: format!("找到 {} 个调用层级项", items.len()),
        result_count: items.len(),
        file_count: unique_file_count(items.iter().map(|item| item.absolute_path.as_str())),
        payload: LspPayload::CallHierarchyItems { items },
    }
}

fn format_call_hierarchy_calls_response(
    label: &str,
    raw: &Value,
    workspace_root: &Path,
) -> LspFormattedResponse {
    let items = raw
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| parse_call_hierarchy_call(item, workspace_root))
        .collect::<Vec<_>>();

    if items.is_empty() {
        return LspFormattedResponse {
            result_text: format!("未找到{label}。"),
            summary: format!("未找到{label}"),
            result_count: 0,
            file_count: 0,
            payload: LspPayload::CallHierarchyCalls { items },
        };
    }

    let mut lines = vec![format!("找到 {} 个{label}：", items.len())];
    for item in &items {
        let call_sites = item
            .call_sites
            .iter()
            .map(|site| format!("{}:{}", site.line, site.character))
            .collect::<Vec<_>>();
        if call_sites.is_empty() {
            lines.push(format!("- {} ({}) - {}:{}", item.name, item.kind, item.path, item.line));
        } else {
            lines.push(format!(
                "- {} ({}) - {}:{} [{}]",
                item.name,
                item.kind,
                item.path,
                item.line,
                call_sites.join(", ")
            ));
        }
    }

    LspFormattedResponse {
        result_text: lines.join("\n"),
        summary: format!("找到 {} 个{label}", items.len()),
        result_count: items.len(),
        file_count: unique_file_count(items.iter().map(|item| item.absolute_path.as_str())),
        payload: LspPayload::CallHierarchyCalls { items },
    }
}

fn parse_locations(raw: &Value, workspace_root: &Path) -> Vec<LspLocationItem> {
    // LSP 的位置响应可能是单对象或数组，统一成 Vec 让上层格式化逻辑保持简单。
    if let Some(array) = raw.as_array() {
        return array.iter().filter_map(|item| parse_location(item, workspace_root)).collect();
    }

    parse_location(raw, workspace_root).into_iter().collect()
}

fn parse_workspace_symbol(raw: &Value, workspace_root: &Path) -> Option<LspLocationItem> {
    let name = raw.get("name").and_then(Value::as_str).map(ToOwned::to_owned);
    let kind =
        raw.get("kind").and_then(Value::as_u64).map(symbol_kind_label).map(ToOwned::to_owned);
    let detail = raw.get("detail").and_then(Value::as_str).map(ToOwned::to_owned);
    let container_name = raw.get("containerName").and_then(Value::as_str).map(ToOwned::to_owned);
    let mut item = parse_location(raw.get("location")?, workspace_root)?;
    item.name = name;
    item.kind = kind;
    item.detail = detail;
    item.container_name = container_name;
    Some(item)
}

fn parse_location(raw: &Value, workspace_root: &Path) -> Option<LspLocationItem> {
    let uri = raw.get("uri").or_else(|| raw.get("targetUri")).and_then(Value::as_str)?.to_string();
    let range = raw
        .get("range")
        .or_else(|| raw.get("targetSelectionRange"))
        .or_else(|| raw.get("targetRange"))?;
    let start = range.get("start")?;
    let line = start.get("line")?.as_u64()? as u32 + 1;
    let character = start.get("character")?.as_u64()? as u32 + 1;
    let absolute_path = decode_file_uri(&uri)?;
    Some(LspLocationItem {
        path: display_path(workspace_root, &absolute_path),
        absolute_path: absolute_path.to_string_lossy().to_string(),
        uri,
        line,
        character,
        name: None,
        kind: None,
        detail: None,
        container_name: None,
    })
}

fn flatten_document_symbol(
    raw: &Value,
    workspace_root: &Path,
    depth: usize,
    output: &mut Vec<LspDocumentSymbolItem>,
) {
    let name = raw.get("name").and_then(Value::as_str).unwrap_or("<unknown>");
    let kind = raw.get("kind").and_then(Value::as_u64).map(symbol_kind_label).unwrap_or("Unknown");
    let detail = raw.get("detail").and_then(Value::as_str).map(ToOwned::to_owned);
    let line = raw
        .get("range")
        .and_then(|range| range.get("start"))
        .and_then(|start| start.get("line"))
        .and_then(Value::as_u64)
        .map(|line| line as u32 + 1)
        .unwrap_or(1);
    let character = raw
        .get("range")
        .and_then(|range| range.get("start"))
        .and_then(|start| start.get("character"))
        .and_then(Value::as_u64)
        .map(|character| character as u32 + 1)
        .unwrap_or(1);
    let absolute_path = raw
        .get("uri")
        .and_then(Value::as_str)
        .and_then(decode_file_uri)
        .unwrap_or_else(|| workspace_root.to_path_buf());

    output.push(LspDocumentSymbolItem {
        name: name.to_string(),
        kind: kind.to_string(),
        detail,
        path: display_path(workspace_root, &absolute_path),
        absolute_path: absolute_path.to_string_lossy().to_string(),
        line,
        character,
        depth,
    });

    for child in raw.get("children").and_then(Value::as_array).into_iter().flatten() {
        flatten_document_symbol(child, workspace_root, depth + 1, output);
    }
}

fn parse_call_hierarchy_item(raw: &Value, workspace_root: &Path) -> Option<LspCallHierarchyItem> {
    let uri = raw.get("uri").and_then(Value::as_str)?.to_string();
    let absolute_path = decode_file_uri(&uri)?;
    Some(LspCallHierarchyItem {
        name: raw.get("name").and_then(Value::as_str).unwrap_or("<unknown>").to_string(),
        kind: raw
            .get("kind")
            .and_then(Value::as_u64)
            .map(symbol_kind_label)
            .unwrap_or("Unknown")
            .to_string(),
        detail: raw.get("detail").and_then(Value::as_str).map(ToOwned::to_owned),
        path: display_path(workspace_root, &absolute_path),
        absolute_path: absolute_path.to_string_lossy().to_string(),
        uri,
        line: raw
            .get("range")
            .and_then(|range| range.get("start"))
            .and_then(|start| start.get("line"))
            .and_then(Value::as_u64)
            .map(|line| line as u32 + 1)
            .unwrap_or(1),
        character: raw
            .get("range")
            .and_then(|range| range.get("start"))
            .and_then(|start| start.get("character"))
            .and_then(Value::as_u64)
            .map(|character| character as u32 + 1)
            .unwrap_or(1),
        call_sites: Vec::new(),
    })
}

fn parse_call_hierarchy_call(raw: &Value, workspace_root: &Path) -> Option<LspCallHierarchyItem> {
    let target = raw.get("from").or_else(|| raw.get("to"))?;
    let mut item = parse_call_hierarchy_item(target, workspace_root)?;
    item.call_sites = raw
        .get("fromRanges")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|range| {
            let start = range.get("start")?;
            Some(LspCallSite {
                line: start.get("line")?.as_u64()? as u32 + 1,
                character: start.get("character")?.as_u64()? as u32 + 1,
            })
        })
        .collect();
    Some(item)
}

fn extract_markup_text(raw: &Value) -> String {
    // hover 内容在不同 server 中可能是字符串、MarkupContent 或 MarkedString 数组。
    match raw {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(extract_markup_text)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        Value::Object(map) => {
            map.get("value").and_then(Value::as_str).map(ToOwned::to_owned).unwrap_or_default()
        }
        _ => String::new(),
    }
}

fn decode_file_uri(uri: &str) -> Option<PathBuf> {
    let path = uri.strip_prefix("file://")?;
    let path = urlencoding::decode(path).ok()?.into_owned();
    Some(PathBuf::from(path))
}

fn display_path(workspace_root: &Path, absolute_path: &Path) -> String {
    if let Ok(relative) = absolute_path.strip_prefix(workspace_root) {
        return relative.to_string_lossy().replace('\\', "/");
    }
    absolute_path.to_string_lossy().replace('\\', "/")
}

fn unique_file_count<'a>(paths: impl Iterator<Item = &'a str>) -> usize {
    paths.collect::<BTreeSet<_>>().len()
}

fn symbol_kind_label(kind: u64) -> &'static str {
    match kind {
        1 => "File",
        2 => "Module",
        3 => "Namespace",
        4 => "Package",
        5 => "Class",
        6 => "Method",
        7 => "Property",
        8 => "Field",
        9 => "Constructor",
        10 => "Enum",
        11 => "Interface",
        12 => "Function",
        13 => "Variable",
        14 => "Constant",
        15 => "String",
        16 => "Number",
        17 => "Boolean",
        18 => "Array",
        19 => "Object",
        20 => "Key",
        21 => "Null",
        22 => "EnumMember",
        23 => "Struct",
        24 => "Event",
        25 => "Operator",
        26 => "TypeParameter",
        _ => "Unknown",
    }
}
#[cfg(test)]
#[path = "format_tests.rs"]
mod format_tests;
