//! 封装设计属性更新逻辑，统一写入元素并触发必要的刷新。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::views::design::canvas::utils::apply_tailwind_classes;
use crate::app::views::design::models::{DesignElement, compute_tree_metrics};
use crate::app::{App, Message};
use iced::Task;

use super::super::load_image_tasks_from_fill_value;

fn find_mut<'a>(elements: &'a mut Vec<DesignElement>, id: &str) -> Option<&'a mut DesignElement> {
    for el in elements {
        if el.id == id {
            return Some(el);
        }
        if let Some(found) = find_mut(&mut el.children, id) {
            return Some(found);
        }
    }
    None
}

fn reapply_tailwind_if_needed(
    doc: &mut crate::app::views::design::models::DesignDoc,
    id: &str,
    needs_class_apply: bool,
) {
    if !needs_class_apply {
        return;
    }
    if let Some(el) = find_mut(&mut doc.children, id) {
        apply_tailwind_classes(el);
    }
}

/// property_update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn property_update(
    app: &mut App,
    id: String,
    key: String,
    value: serde_json::Value,
) -> Task<Message> {
    let image_tasks =
        if key == "fill" { load_image_tasks_from_fill_value(&value) } else { Vec::new() };
    if let Some(state) = app.active_design_state_mut() {
        let doc = &mut state.doc;
        doc.update_property(&id, &key, value);
        reapply_tailwind_if_needed(doc, &id, key == "class");
        if key == "name" || key == "kind" || key == "reference" {
            state.layer_tree_metrics = compute_tree_metrics(&state.doc);
        }
        state.canvas_cache.clear();
    }
    Task::batch(image_tasks)
}

/// properties_update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn properties_update(
    app: &mut App,
    id: String,
    props: Vec<(String, serde_json::Value)>,
) -> Task<Message> {
    let mut image_tasks = Vec::new();
    for (key, value) in &props {
        if key == "fill" {
            image_tasks.extend(load_image_tasks_from_fill_value(value));
        }
    }
    if let Some(state) = app.active_design_state_mut() {
        let doc = &mut state.doc;
        for (key, value) in &props {
            doc.update_property(&id, key, value.clone());
        }
        reapply_tailwind_if_needed(doc, &id, props.iter().any(|(key, _)| key == "class"));
        if props.iter().any(|(key, _)| key == "name" || key == "kind" || key == "reference") {
            state.layer_tree_metrics = compute_tree_metrics(&state.doc);
        }
        state.canvas_cache.clear();
    }
    Task::batch(image_tasks)
}

/// batch_properties_update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn batch_properties_update(
    app: &mut App,
    updates: Vec<(String, Vec<(String, serde_json::Value)>)>,
) -> Task<Message> {
    let mut image_tasks = Vec::new();
    for (_, props) in &updates {
        for (key, value) in props {
            if key == "fill" {
                image_tasks.extend(load_image_tasks_from_fill_value(value));
            }
        }
    }
    if let Some(state) = app.active_design_state_mut() {
        let doc = &mut state.doc;
        let mut affects_metrics = false;
        for (id, props) in updates {
            for (key, value) in props {
                if key == "name" || key == "kind" || key == "reference" {
                    affects_metrics = true;
                }
                doc.update_property(&id, &key, value);
            }
        }
        if affects_metrics {
            state.layer_tree_metrics = compute_tree_metrics(&state.doc);
        }
        state.canvas_cache.clear();
    }
    Task::batch(image_tasks)
}
