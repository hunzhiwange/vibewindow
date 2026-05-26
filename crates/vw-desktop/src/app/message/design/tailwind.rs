//! 处理设计工具中的 Tailwind 导入、解析和类名生成任务。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::DesignMessage;
use crate::app::views::design::models::DesignElement;
use crate::app::{App, Message};
use iced::Task;

fn find_element_mut<'a>(
    elements: &'a mut Vec<DesignElement>,
    id: &str,
) -> Option<&'a mut DesignElement> {
    for element in elements {
        if element.id == id {
            return Some(element);
        }
        if let Some(found) = find_element_mut(&mut element.children, id) {
            return Some(found);
        }
    }
    None
}

fn sync_tailwind_html_editor(state: &mut crate::app::views::design::state::DesignState, id: &str) {
    if let Some(element) = find_element_mut(&mut state.doc.children, id)
        && state.selected_element_id.as_deref() == Some(id)
        && let Some(html) = element.content.as_deref()
    {
        state.tailwind_html_editor = iced::widget::text_editor::Content::with_text(html);
    }
}

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn update(app: &mut App, message: DesignMessage) -> Option<Task<Message>> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        DesignMessage::ConvertHtmlToLayers(id) => {
            let mut tasks = Vec::new();
            if let Some(state) = app.active_design_state_mut() {
                use crate::app::views::design::canvas::layout::resolve_element_size;
                use crate::app::views::design::import::import_html_as_positioned_elements;

                fn replace_in_elements(
                    elements: &mut Vec<DesignElement>,
                    target_id: &str,
                    new_elements: &[DesignElement],
                ) -> bool {
                    for element in elements {
                        if element.id == target_id {
                            element.kind = "Frame".to_string();
                            element.children = new_elements.to_vec();
                            element.content = None;
                            return true;
                        }
                        if replace_in_elements(&mut element.children, target_id, new_elements) {
                            return true;
                        }
                    }
                    false
                }

                #[cfg(not(target_arch = "wasm32"))]
                fn collect_images(elements: &[DesignElement], tasks: &mut Vec<Task<Message>>) {
                    for element in elements {
                        if element.kind == "image"
                            && let Some(serde_json::Value::Object(map)) = &element.fill
                            && let Some(serde_json::Value::String(url)) = map.get("url")
                            && url.starts_with("http")
                        {
                            let url_clone = url.clone();
                            tasks.push(Task::perform(
                                async move {
                                    use iced::widget::image::Handle;
                                    use image::GenericImageView;

                                    let client = reqwest::Client::new();
                                    match client.get(&url_clone).send().await {
                                        Ok(response) => {
                                            if response.status().is_success() {
                                                match response.bytes().await {
                                                    Ok(bytes) => {
                                                        let bytes = bytes.to_vec();
                                                        let size_opt = image::load_from_memory(&bytes)
                                                            .ok()
                                                            .map(|img| img.dimensions());
                                                        (
                                                            url_clone,
                                                            Ok((Handle::from_bytes(bytes), size_opt)),
                                                        )
                                                    }
                                                    Err(error) => (url_clone, Err(error.to_string())),
                                                }
                                            } else {
                                                (
                                                    url_clone,
                                                    Err(format!("HTTP {}", response.status())),
                                                )
                                            }
                                        }
                                        Err(error) => (url_clone, Err(error.to_string())),
                                    }
                                },
                                |(url, result)| {
                                    Message::Design(DesignMessage::ImageLoaded(url, result))
                                },
                            ));
                        }
                        collect_images(&element.children, tasks);
                    }
                }

                let converted = state.doc.find_element(&id).and_then(|element| {
                    let html = element.content.clone()?;
                    let size = resolve_element_size(element, None, &state.doc, None);
                    Some(import_html_as_positioned_elements(&html, size))
                });

                if let Some(new_elements) = converted {
                    #[cfg(not(target_arch = "wasm32"))]
                    collect_images(&new_elements, &mut tasks);

                    replace_in_elements(&mut state.doc.children, &id, &new_elements);

                    if state
                        .doc
                        .tailwind_selection
                        .as_ref()
                        .is_some_and(|(selected_id, _)| selected_id == &id)
                    {
                        state.doc.tailwind_selection = None;
                        state.tailwind_html_editor = iced::widget::text_editor::Content::new();
                        state.tailwind_node_class_editor =
                            iced::widget::text_editor::Content::new();
                        state.tailwind_node_text_editor =
                            iced::widget::text_editor::Content::new();
                        state.tailwind_node_class_input.clear();
                        state.tailwind_node_class_dropdown_open = false;
                    }

                    state.canvas_cache.clear();
                    tasks.push(Task::done(Message::Design(DesignMessage::Snapshot)));
                }
            }
            Some(Task::batch(tasks))
        }
        DesignMessage::UpdateTailwindNodeClass(id, path, class) => {
            if let Some(state) = app.active_design_state_mut() {
                if let Some(element) = find_element_mut(&mut state.doc.children, &id)
                    && let Some(content) = &element.content
                {
                    let mut nodes =
                        crate::app::views::design::canvas::tailwind::dom::parse_html(content);

                    if !path.is_empty()
                        && let Some(root_idx) = path.first()
                        && let Some(current_node) = nodes.get_mut(*root_idx)
                    {
                        fn update_node(
                            node: &mut crate::app::views::design::canvas::tailwind::dom::TailwindNode,
                            path: &[usize],
                            class: &str,
                        ) {
                            if path.is_empty() {
                                node.attributes
                                    .insert("class".to_string(), class.to_string());
                                return;
                            }
                            let idx = path[0];
                            if let Some(child) = node.children.get_mut(idx) {
                                update_node(child, &path[1..], class);
                            }
                        }

                        update_node(current_node, &path[1..], &class);
                    }

                    element.content = Some(
                        crate::app::views::design::canvas::tailwind::dom::nodes_to_html(&nodes),
                    );
                }
                sync_tailwind_html_editor(state, &id);
                if let Some((selected_id, selected_path)) = state.doc.tailwind_selection.as_ref()
                    && selected_id == &id
                    && selected_path.as_slice() == path.as_slice()
                {
                    state.tailwind_node_class_editor =
                        iced::widget::text_editor::Content::with_text(&class);
                }
                state.canvas_cache.clear();
            }
            Some(Task::done(Message::Design(DesignMessage::Snapshot)))
        }
        DesignMessage::TailwindNodeClassCommit(id, path) => {
            let normalized = app
                .active_design_state()
                .map(|state| {
                    state
                        .tailwind_node_class_editor
                        .text()
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            Some(super::update(
                app,
                DesignMessage::UpdateTailwindNodeClass(id, path, normalized),
            ))
        }
        DesignMessage::TailwindNodeTextCommit(id, path) => {
            let text = app
                .active_design_state()
                .map(|state| state.tailwind_node_text_editor.text().to_string())
                .unwrap_or_default();
            Some(super::update(
                app,
                DesignMessage::UpdateTailwindNodeText(id, path, text),
            ))
        }
        DesignMessage::UpdateTailwindNodeText(id, path, text) => {
            if let Some(state) = app.active_design_state_mut() {
                if let Some(element) = find_element_mut(&mut state.doc.children, &id)
                    && let Some(content) = &element.content
                {
                    let mut nodes =
                        crate::app::views::design::canvas::tailwind::dom::parse_html(content);

                    if !path.is_empty()
                        && let Some(root_idx) = path.first()
                        && let Some(current_node) = nodes.get_mut(*root_idx)
                    {
                        fn update_node(
                            node: &mut crate::app::views::design::canvas::tailwind::dom::TailwindNode,
                            path: &[usize],
                            text: &str,
                        ) {
                            if path.is_empty() {
                                node.text = Some(text.to_string());
                                return;
                            }
                            let idx = path[0];
                            if let Some(child) = node.children.get_mut(idx) {
                                update_node(child, &path[1..], text);
                            }
                        }

                        update_node(current_node, &path[1..], &text);
                    }

                    element.content = Some(
                        crate::app::views::design::canvas::tailwind::dom::nodes_to_html(&nodes),
                    );
                }
                sync_tailwind_html_editor(state, &id);
                if let Some((selected_id, selected_path)) = state.doc.tailwind_selection.as_ref()
                    && selected_id == &id
                    && selected_path.as_slice() == path.as_slice()
                {
                    state.tailwind_node_text_editor =
                        iced::widget::text_editor::Content::with_text(&text);
                }
                state.canvas_cache.clear();
            }
            Some(Task::done(Message::Design(DesignMessage::Snapshot)))
        }
        DesignMessage::UpdateTailwindHtml(id, html) => {
            if let Some(state) = app.active_design_state_mut() {
                if let Some(element) = find_element_mut(&mut state.doc.children, &id) {
                    element.content = Some(html);
                }
                sync_tailwind_html_editor(state, &id);
                state.canvas_cache.clear();
            }
            Some(Task::done(Message::Design(DesignMessage::Snapshot)))
        }
        DesignMessage::DeleteTailwindNode(id, path) => {
            if let Some(state) = app.active_design_state_mut() {
                if let Some(element) = find_element_mut(&mut state.doc.children, &id)
                    && let Some(content) = &element.content
                {
                    let mut nodes =
                        crate::app::views::design::canvas::tailwind::dom::parse_html(content);
                    if crate::app::views::design::canvas::tailwind::dom::remove_node_by_path(
                        &mut nodes, &path,
                    ) {
                        let html =
                            crate::app::views::design::canvas::tailwind::dom::nodes_to_html(&nodes);
                        element.content = Some(html.clone());
                        if state.selected_element_id.as_deref() == Some(id.as_str()) {
                            state.tailwind_html_editor =
                                iced::widget::text_editor::Content::with_text(&html);
                        }
                        if state
                            .doc
                            .tailwind_selection
                            .as_ref()
                            .is_some_and(|(selected_id, selected_path)| {
                                selected_id == &id && selected_path.as_slice() == path.as_slice()
                            })
                        {
                            state.doc.tailwind_selection = None;
                            state.tailwind_node_class_editor =
                                iced::widget::text_editor::Content::new();
                            state.tailwind_node_text_editor =
                                iced::widget::text_editor::Content::new();
                            state.tailwind_node_class_input.clear();
                            state.tailwind_node_class_dropdown_open = false;
                        }
                    }
                }
                state.canvas_cache.clear();
            }
            Some(Task::done(Message::Design(DesignMessage::Snapshot)))
        }
        _ => None,
    }
}

