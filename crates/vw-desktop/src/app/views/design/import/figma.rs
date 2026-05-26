//! Figma 导入模块，负责把 Figma JSON 中的节点、几何、样式和辅助字段转换为设计模型。

use crate::app::views::design::models::{DesignDoc, DesignElement, DesignGroup};
use anyhow::{Context, Result, anyhow};
use serde_json::{Map, Value, json};
use std::path::Path;

use super::figma_node::{
    FigmaImportContext, figma_node_to_element_with_parent, match_raw_child, read_transform_value,
};
use super::figma_support::read_node_type;
use super::figma_style::color_from_value;
use super::shared::{generate_id, numeric_value};

#[derive(Debug, Clone)]
/// FigmaImportProgress 状态结构，保存当前 UI 或导入流程需要跨消息传递的数据。
pub struct FigmaImportProgress {
    pub completed_pages: usize,
    pub total_pages: usize,
    pub detail: String,
}

#[cfg(target_arch = "wasm32")]
fn figma_import_unsupported<T>() -> Result<T> {
    Err(anyhow!("Web 平台暂不支持导入 Figma .fig"))
}

#[cfg(target_arch = "wasm32")]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_elements(_bytes: &[u8]) -> Result<Vec<DesignElement>> {
    figma_import_unsupported()
}

#[cfg(target_arch = "wasm32")]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_elements_progress<F>(
    _bytes: &[u8],
    _on_progress: F,
) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    figma_import_unsupported()
}

#[cfg(target_arch = "wasm32")]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc(_bytes: &[u8]) -> Result<DesignDoc> {
    figma_import_unsupported()
}

#[cfg(target_arch = "wasm32")]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_progress<F>(_bytes: &[u8], _on_progress: F) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    figma_import_unsupported()
}

#[cfg(target_arch = "wasm32")]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_base_dir(
    _bytes: &[u8],
    _base_dir: Option<&Path>,
) -> Result<DesignDoc> {
    figma_import_unsupported()
}

#[cfg(target_arch = "wasm32")]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_base_dir_and_progress<F>(
    _bytes: &[u8],
    _base_dir: Option<&Path>,
    _on_progress: F,
) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    figma_import_unsupported()
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_elements(bytes: &[u8]) -> Result<Vec<DesignElement>> {
    Ok(figma_to_design_doc(bytes)?.children)
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_elements_progress<F>(
    bytes: &[u8],
    mut on_progress: F,
) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    figma_to_design_doc_with_progress(bytes, &mut on_progress)
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc(bytes: &[u8]) -> Result<DesignDoc> {
    figma_to_design_doc_with_base_dir(bytes, None)
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_progress<F>(bytes: &[u8], on_progress: F) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    figma_to_design_doc_with_base_dir_and_progress(bytes, None, on_progress)
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_base_dir(
    bytes: &[u8],
    base_dir: Option<&Path>,
) -> Result<DesignDoc> {
    let json =
        vw_figma_json::convert(bytes, base_dir).context("Failed to convert Figma .fig file")?;
    let raw_json =
        vw_figma_json::convert_raw(bytes).context("Failed to load raw Figma .fig file")?;
    figma_json_to_design_doc_with_raw(json, Some(&raw_json))
}

#[cfg(not(target_arch = "wasm32"))]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_to_design_doc_with_base_dir_and_progress<F>(
    bytes: &[u8],
    base_dir: Option<&Path>,
    on_progress: F,
) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    let json =
        vw_figma_json::convert(bytes, base_dir).context("Failed to convert Figma .fig file")?;
    let raw_json =
        vw_figma_json::convert_raw(bytes).context("Failed to load raw Figma .fig file")?;
    figma_json_to_design_doc_with_raw_progress(json, Some(&raw_json), on_progress)
}

#[cfg(test)]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub(crate) fn figma_json_to_design_doc(json: Value) -> Result<DesignDoc> {
    figma_json_to_design_doc_with_raw_progress(json, None, |_| {})
}

#[cfg(any(test, not(target_arch = "wasm32")))]
/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub(crate) fn figma_json_to_design_doc_with_raw(
    json: Value,
    raw_json: Option<&Value>,
) -> Result<DesignDoc> {
    figma_json_to_design_doc_with_raw_progress(json, raw_json, |_| {})
}

/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn count_figma_pages(json: &Value) -> usize {
    json.get("document")
        .and_then(|document| document.get("children"))
        .and_then(Value::as_array)
        .map(|root_nodes| {
            let (_, pages) = collect_grouped_pages(root_nodes);
            pages.len()
        })
        .unwrap_or(0)
}

/// 执行 Figma 数据到设计文档的转换。
///
/// 转换失败时返回错误，调用方据此中止导入而不是生成半成品。
pub fn figma_json_to_design_doc_with_progress<F>(
    json: Value,
    on_progress: F,
) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    figma_json_to_design_doc_with_raw_progress(json, None, on_progress)
}

fn figma_json_to_design_doc_with_raw_progress<F>(
    json: Value,
    raw_json: Option<&Value>,
    mut on_progress: F,
) -> Result<DesignDoc>
where
    F: FnMut(FigmaImportProgress),
{
    let document = json
        .get("document")
        .ok_or_else(|| anyhow!("Figma document is missing a root document node"))?;
    let root_nodes = document
        .get("children")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("Figma document does not contain any pages"))?;
    let (groups, pages) = collect_grouped_pages(root_nodes);
    let raw_pages = raw_json
        .and_then(|raw| raw.get("document"))
        .and_then(|document| document.get("children"))
        .and_then(Value::as_array)
        .map(|root_nodes| collect_grouped_pages(root_nodes).1)
        .unwrap_or_default();
    let import_context = FigmaImportContext::from_raw(raw_json);
    let total_pages = pages.len();
    let mut page_elements = Vec::new();

    for (index, (group_id, page)) in pages.into_iter().enumerate() {
        let raw_page = raw_pages.get(index).map(|(_, page)| *page);
        page_elements.extend(figma_page_to_elements(page, raw_page, group_id, &import_context));
        on_progress(FigmaImportProgress {
            completed_pages: index.saturating_add(1),
            total_pages,
            detail: figma_page_progress_detail(page, index.saturating_add(1), total_pages),
        });
    }

    if page_elements.is_empty() {
        return Err(anyhow!("Figma document does not contain importable pages"));
    }

    let mut doc = serde_json::from_value::<DesignDoc>(json!({
        "version": "2.6",
        "children": page_elements,
        "variables": {},
        "groups": groups,
        "theme": {
            "Mode": "Light"
        }
    }))
    .context("Failed to build imported design document")?;
    doc.normalize_fill_flags();
    doc.normalize_groups();
    Ok(doc)
}

fn figma_page_progress_detail(page: &Value, current: usize, total: usize) -> String {
    let page_name = page
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("未命名页面");
    format!("正在处理页面 {}/{}：{}", current, total, page_name)
}

fn collect_grouped_pages(root_nodes: &[Value]) -> (Vec<DesignGroup>, Vec<(u32, &Value)>) {
    let mut groups = Vec::new();
    let mut pages = Vec::new();
    let mut next_group_id = 0u32;

    for node in root_nodes {
        let Some(object) = node.as_object() else {
            continue;
        };
        let children =
            object.get("children").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
        if is_figma_page_group(object, children) {
            let group_id = next_group_id;
            next_group_id = next_group_id.saturating_add(1);
            groups.push(DesignGroup {
                id: group_id,
                name: object
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or("未命名分组")
                    .to_string(),
            });
            for child in children {
                if child.get("visible").and_then(Value::as_bool).unwrap_or(true) {
                    pages.push((group_id, child));
                }
            }
        } else {
            pages.push((u32::MAX, node));
        }
    }

    let has_ungrouped_pages = pages.iter().any(|(group_id, _)| *group_id == u32::MAX);

    if has_ungrouped_pages {
        let default_group_id = if groups.is_empty() { 0 } else { next_group_id };
        let group_name =
            infer_ungrouped_group_name(&pages).unwrap_or_else(|| DesignDoc::default_group_name(0));
        groups.push(DesignGroup { id: default_group_id, name: group_name });
        for (group_id, _) in &mut pages {
            if *group_id == u32::MAX {
                *group_id = default_group_id;
            }
        }
    } else if groups.is_empty() {
        groups.push(DesignGroup { id: 0, name: DesignDoc::default_group_name(0) });
    }

    (groups, pages)
}

fn infer_ungrouped_group_name(pages: &[(u32, &Value)]) -> Option<String> {
    let mut names = pages
        .iter()
        .filter(|(group_id, _)| *group_id == u32::MAX)
        .filter_map(|(_, page)| page.get("name").and_then(Value::as_str))
        .map(str::trim)
        .filter(|name| !name.is_empty());

    let first_name = names.next()?;
    if names.next().is_none() { Some(first_name.to_string()) } else { None }
}

fn is_figma_page_group(object: &Map<String, Value>, children: &[Value]) -> bool {
    if children.is_empty() {
        return false;
    }

    let node_type = read_node_type(object).unwrap_or_default();
    if matches!(node_type, "SECTION" | "GROUP" | "FOLDER") {
        return true;
    }

    if children.iter().any(|child| child.get("type").and_then(Value::as_str) == Some("CANVAS")) {
        return true;
    }

    let mut visible_children = children
        .iter()
        .filter(|child| child.get("visible").and_then(Value::as_bool).unwrap_or(true))
        .peekable();
    if visible_children.peek().is_none() {
        return false;
    }

    visible_children.all(|child| {
        child
            .get("children")
            .and_then(Value::as_array)
            .is_some_and(|grandchildren| !grandchildren.is_empty())
    })
}

fn figma_page_to_elements(
    page: &Value,
    raw_page: Option<&Value>,
    group_id: u32,
    context: &FigmaImportContext,
) -> Vec<DesignElement> {
    let Some(page_children) = page.get("children").and_then(Value::as_array) else {
        return Vec::new();
    };
    let Some(page_object) = page.as_object() else {
        return Vec::new();
    };
    let raw_page_children =
        raw_page.and_then(|page| page.get("children")).and_then(Value::as_array);
    let page_abs_x = read_transform_value(page_object, "x");
    let page_abs_y = read_transform_value(page_object, "y");
    let visible_children: Vec<DesignElement> = page_children
        .iter()
        .enumerate()
        .filter(|(_, child)| child.get("visible").and_then(Value::as_bool).unwrap_or(true))
        .filter_map(|(index, child)| {
            figma_node_to_element_with_parent(
                child,
                raw_page_children.and_then(|children| match_raw_child(children, child, index)),
                0.0,
                0.0,
                context,
            )
        })
        .collect();

    if !visible_children.is_empty() {
        let mut page_element =
            figma_node_to_element_with_parent(page, raw_page, page_abs_x, page_abs_y, context)
                .unwrap_or_else(|| {
                    figma_page_fallback_element(page, page_children, page_abs_x, page_abs_y)
                });
        if page_element.kind != "frame" && page_element.kind != "group" {
            page_element.kind = "group".to_string();
        }
        if page_element.name.is_none() {
            page_element.name = Some("Imported Figma Page".to_string());
        }
        page_element.children = visible_children;
        page_element.set_group_id_recursive(group_id);
        ensure_frame_size_from_children(&mut page_element);
        return vec![page_element];
    }

    let mut page_element =
        figma_node_to_element_with_parent(page, raw_page, page_abs_x, page_abs_y, context)
            .unwrap_or_else(|| {
                figma_page_fallback_element(page, page_children, page_abs_x, page_abs_y)
            });
    if page_element.kind != "frame" && page_element.kind != "group" {
        page_element.kind = "group".to_string();
    }
    if page_element.name.is_none() {
        page_element.name = Some("Imported Figma Page".to_string());
    }
    if page_element.children.is_empty() {
        page_element.children = page_children
            .iter()
            .enumerate()
            .filter_map(|(index, child)| {
                figma_node_to_element_with_parent(
                    child,
                    raw_page_children.and_then(|children| match_raw_child(children, child, index)),
                    0.0,
                    0.0,
                    context,
                )
            })
            .collect();
    }
    page_element.set_group_id_recursive(group_id);
    ensure_frame_size_from_children(&mut page_element);
    vec![page_element]
}

fn figma_page_fallback_element(
    page: &Value,
    page_children: &[Value],
    page_abs_x: f32,
    page_abs_y: f32,
) -> DesignElement {
    let object = page.as_object();
    let mut element = DesignElement {
        id: object
            .and_then(|page| page.get("id"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(generate_id),
        kind: "frame".to_string(),
        name: object
            .and_then(|page| page.get("name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        x: page_abs_x,
        y: page_abs_y,
        fill: object
            .and_then(|page| page.get("backgroundColor"))
            .and_then(color_from_value)
            .map(Value::String),
        visible: Some(true),
        enabled: Some(true),
        children: page_children
            .iter()
            .filter_map(|child| {
                figma_node_to_element_with_parent(
                    child,
                    None,
                    page_abs_x,
                    page_abs_y,
                    &FigmaImportContext::default(),
                )
            })
            .collect(),
        ..DesignElement::default()
    };
    ensure_frame_size_from_children(&mut element);
    element
}

fn ensure_frame_size_from_children(element: &mut DesignElement) {
    if element.width.is_some() && element.height.is_some() {
        return;
    }

    let mut max_x = 0.0_f32;
    let mut max_y = 0.0_f32;

    for child in &element.children {
        max_x = max_x.max(child.x + numeric_value(&child.width).unwrap_or(0.0));
        max_y = max_y.max(child.y + numeric_value(&child.height).unwrap_or(0.0));
    }

    if element.width.is_none() && max_x > 0.0 {
        element.width = Some(json!(max_x));
    }
    if element.height.is_none() && max_y > 0.0 {
        element.height = Some(json!(max_y));
    }
}
