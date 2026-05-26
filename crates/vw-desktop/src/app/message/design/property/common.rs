//! 提供设计属性编辑时共享的解析、校验和状态更新辅助逻辑。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::views::design::models::{DesignElement, ThemeCondition, VariableValue};
use crate::app::views::design::properties::fill::types::{FillItem, FillObject};
use std::collections::HashMap;

/// upsert_variable_value 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn upsert_variable_value(
    values: &mut Vec<VariableValue>,
    mode: Option<&str>,
    new_value: String,
) {
    let target_mode = mode.map(str::trim).filter(|value| !value.is_empty());
    if let Some(existing) = values.iter_mut().find(|entry| match (&entry.theme, target_mode) {
        (None, None) => true,
        (Some(theme), Some(target)) => theme.mode.eq_ignore_ascii_case(target),
        _ => false,
    }) {
        if new_value.is_empty() {
            existing.value.clear();
        } else {
            existing.value = new_value;
        }
    } else if !new_value.is_empty() {
        values.push(VariableValue {
            value: new_value,
            theme: target_mode.map(|target| ThemeCondition { mode: target.to_string() }),
        });
    }

    values.retain(|entry| !entry.value.trim().is_empty());
}

fn next_page_clone_id(counter: &mut u64) -> String {
    *counter += 1;
    let now = crate::app::time::now_ms();
    format!("page_{}_{}", now, counter)
}

/// clone_page_elements 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn clone_page_elements(elements: &[DesignElement], new_group_id: u32) -> Vec<DesignElement> {
    let mut counter = 0;
    let mut id_map = HashMap::new();
    let mut cloned = elements.to_vec();
    for element in &mut cloned {
        rewrite_page_clone_tree(element, new_group_id, &mut counter, &mut id_map);
    }
    for element in &mut cloned {
        rewrite_page_clone_references(element, &id_map);
    }
    cloned
}

fn rewrite_page_clone_tree(
    element: &mut DesignElement,
    new_group_id: u32,
    counter: &mut u64,
    id_map: &mut HashMap<String, String>,
) {
    let old_id = element.id.clone();
    let new_id = next_page_clone_id(counter);
    element.id = new_id.clone();
    element.group_id = new_group_id;
    id_map.insert(old_id, new_id);
    for child in &mut element.children {
        rewrite_page_clone_tree(child, new_group_id, counter, id_map);
    }
}

fn rewrite_page_clone_references(element: &mut DesignElement, id_map: &HashMap<String, String>) {
    if let Some(reference) = element.reference.as_mut()
        && let Some(mapped_reference) = id_map.get(reference)
    {
        *reference = mapped_reference.clone();
    }
    for child in &mut element.children {
        rewrite_page_clone_references(child, id_map);
    }
}

/// parse_fills 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn parse_fills(v: &serde_json::Value) -> Vec<FillItem> {
    if let Ok(mut fills) = serde_json::from_value::<Vec<FillItem>>(v.clone()) {
        for item in &mut fills {
            if let FillItem::Object(FillObject::Mesh(mesh)) = item {
                mesh.normalize();
            }
        }
        return fills;
    }

    if let Ok(mut item) = serde_json::from_value::<FillItem>(v.clone()) {
        if let FillItem::Object(FillObject::Mesh(mesh)) = &mut item {
            mesh.normalize();
        }
        return vec![item];
    }

    if let Some(s) = v.as_str() {
        return vec![FillItem::Color(s.to_string())];
    }

    if let Some(obj) = v.as_object() {
        if obj.contains_key("color")
            && !obj.contains_key("type")
            && let Ok(solid) = serde_json::from_value::<FillObject>(serde_json::json!({
                "type": "solid",
                "color": obj.get("color").unwrap(),
                "enabled": obj.get("enabled").unwrap_or(&serde_json::json!(true))
            }))
        {
            return vec![FillItem::Object(solid)];
        }
    }

    vec![]
}

