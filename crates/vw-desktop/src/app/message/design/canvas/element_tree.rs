//! 维护设计画布元素树的选择、查找、变更与层级关系更新。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::Message;
use crate::app::views::design::canvas::geometry::get_element_screen_bounds;
use crate::app::views::design::canvas::layout::parse::parse_padding;
use crate::app::views::design::models::DesignElement;
use crate::app::views::design::state::DesignState;
use iced::Task;

/// insert_into_parent 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn insert_into_parent(
    children: &mut [DesignElement],
    parent_id: &str,
    element: DesignElement,
) -> Result<(), DesignElement> {
    let mut pending = Some(element);

    for child in children {
        if child.id == parent_id {
            if let Some(element) = pending.take() {
                child.children.push(element);
            }
            return Ok(());
        }

        if let Some(element) = pending.take() {
            match insert_into_parent(&mut child.children, parent_id, element) {
                Ok(()) => return Ok(()),
                Err(element) => pending = Some(element),
            }
        }
    }

    match pending {
        Some(element) => Err(element),
        None => Ok(()),
    }
}

fn remove_node(children: &mut Vec<DesignElement>, id: &str) -> Option<DesignElement> {
    if let Some(index) = children.iter().position(|child| child.id == id) {
        return Some(children.remove(index));
    }

    for child in children {
        if let Some(element) = remove_node(&mut child.children, id) {
            return Some(element);
        }
    }

    None
}

fn find_mut<'a>(children: &'a mut Vec<DesignElement>, id: &str) -> Option<&'a mut DesignElement> {
    for element in children {
        if element.id == id {
            return Some(element);
        }
        if let Some(found) = find_mut(&mut element.children, id) {
            return Some(found);
        }
    }
    None
}

fn find_path<'a>(
    elements: &'a [DesignElement],
    id: &str,
    path: &mut Vec<&'a DesignElement>,
) -> bool {
    for element in elements {
        path.push(element);
        if element.id == id {
            return true;
        }
        if find_path(&element.children, id, path) {
            return true;
        }
        path.pop();
    }
    false
}

fn is_ancestor_of(root: &[DesignElement], ancestor_id: &str, desc_id: &str) -> bool {
    let mut path = Vec::new();
    if find_path(root, desc_id, &mut path) {
        return path.iter().any(|element| element.id == ancestor_id);
    }
    false
}

/// reparent_elements 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn reparent_elements(
    state: &mut DesignState,
    ids: Vec<String>,
    parent_opt: Option<String>,
) -> Task<Message> {
    let pan = state.pan;
    let zoom = state.zoom;
    let doc = &mut state.doc;

    for id in ids {
        if let Some(element_rect) = get_element_screen_bounds(doc, &id, pan, zoom) {
            let element_doc_x = (element_rect.x - pan.x) / zoom;
            let element_doc_y = (element_rect.y - pan.y) / zoom;

            let mut element = match remove_node(&mut doc.children, &id) {
                Some(element) => element,
                None => continue,
            };

            if let Some(parent_id) = &parent_opt {
                if is_ancestor_of(&doc.children, &id, parent_id) {
                    element.x = element_doc_x;
                    element.y = element_doc_y;
                    doc.children.push(element);
                    continue;
                }

                if let Some(frame_rect) = get_element_screen_bounds(doc, parent_id, pan, zoom) {
                    let frame_doc_x = (frame_rect.x - pan.x) / zoom;
                    let frame_doc_y = (frame_rect.y - pan.y) / zoom;

                    if let Some(frame_element) = find_mut(&mut doc.children, parent_id) {
                        let theme_mode = doc.theme.as_ref().map(|theme| theme.mode.as_str());
                        let padding =
                            parse_padding(&frame_element.padding, &doc.variables, theme_mode);
                        element.x = element_doc_x - frame_doc_x - padding.left;
                        element.y = element_doc_y - frame_doc_y - padding.top;
                        frame_element.children.push(element);
                    } else {
                        element.x = element_doc_x;
                        element.y = element_doc_y;
                        doc.children.push(element);
                    }
                } else {
                    element.x = element_doc_x;
                    element.y = element_doc_y;
                    doc.children.push(element);
                }
            } else {
                element.x = element_doc_x;
                element.y = element_doc_y;
                doc.children.push(element);
            }
        }
    }

    state.canvas_cache.clear();
    Task::none()
}
