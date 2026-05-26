//! 思维导图 Markdown 操作更新逻辑，处理 Markdown 导入和节点文本同步。

use crate::app::components::mind_map;
use crate::app::{App, Message};
use crate::apps::mindmap::model;
use iced::Task;
use iced::widget::text_editor;
use std::collections::HashMap;
use std::collections::VecDeque;

use super::super::persist::persist;

/// 构建或更新 toggle markdown import 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn toggle_markdown_import(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        super::node_ops::commit_text_editor_if_needed(tab);
        tab.show_markdown_import = !tab.show_markdown_import;
        if tab.show_markdown_import {
            tab.active_color_picker = None;
            tab.show_diagram_type_picker = false;
            tab.show_zoom_menu = false;
            tab.show_priority_picker = false;
            tab.show_url_editor = false;
            tab.show_text_editor = false;
            tab.url_editor_value.clear();
            tab.node_text_editor = text_editor::Content::new();
            tab.show_action_menu = false;
            tab.show_context_menu = false;
            let md = mind_map::to_markdown(&tab.doc);
            tab.markdown_import_editor = text_editor::Content::with_text(&md);
        }
    }
    Task::none()
}

/// 构建或更新 markdown import editor action 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn markdown_import_editor_action(
    app: &mut App,
    action: text_editor::Action,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        match action {
            text_editor::Action::Edit(edit) => {
                tab.markdown_import_editor.perform(text_editor::Action::Edit(edit));

                let md = tab.markdown_import_editor.text();
                let old_doc = tab.doc.clone();
                let old_node_fills = tab.node_fills.clone();
                let old_node_text_colors = tab.node_text_colors.clone();
                let old_node_border_colors = tab.node_border_colors.clone();
                let old_node_border_styles = tab.node_border_styles.clone();
                let old_node_priorities = tab.node_priorities.clone();
                let old_node_urls = tab.node_urls.clone();
                let old_edge_styles = tab.edge_styles.clone();
                let old_edge_colors = tab.edge_colors.clone();
                let old_collapsed_paths = tab.collapsed_paths.clone();

                let new_doc =
                    if md.trim().is_empty() { model::default_doc() } else { mind_map::parse(&md) };

                fn match_paths(
                    old: &mind_map::MindNode,
                    new: &mind_map::MindNode,
                ) -> HashMap<Vec<usize>, Vec<usize>> {
                    fn go(
                        old: &mind_map::MindNode,
                        old_path: &Vec<usize>,
                        new: &mind_map::MindNode,
                        new_path: &Vec<usize>,
                        out: &mut HashMap<Vec<usize>, Vec<usize>>,
                    ) {
                        let mut old_by_text: HashMap<String, VecDeque<usize>> = HashMap::new();
                        for (i, c) in old.children.iter().enumerate() {
                            old_by_text.entry(c.text.trim().to_string()).or_default().push_back(i);
                        }

                        for (j, c_new) in new.children.iter().enumerate() {
                            let key = c_new.text.trim().to_string();
                            let Some(q) = old_by_text.get_mut(&key) else {
                                continue;
                            };
                            let Some(i) = q.pop_front() else {
                                continue;
                            };

                            let mut op = old_path.clone();
                            op.push(i);
                            let mut np = new_path.clone();
                            np.push(j);

                            out.insert(op.clone(), np.clone());
                            go(&old.children[i], &op, c_new, &np, out);
                        }
                    }

                    let mut out = HashMap::new();
                    out.insert(Vec::new(), Vec::new());
                    go(old, &Vec::new(), new, &Vec::new(), &mut out);
                    out
                }

                let mapping = match_paths(&old_doc, &new_doc);

                let remap_hash = |src: HashMap<Vec<usize>, u32>| -> HashMap<Vec<usize>, u32> {
                    let mut out = HashMap::new();
                    for (k, v) in src {
                        if let Some(nk) = mapping.get(&k) {
                            out.insert(nk.clone(), v);
                        }
                    }
                    out
                };
                let remap_hash_u8 = |src: HashMap<Vec<usize>, u8>| -> HashMap<Vec<usize>, u8> {
                    let mut out = HashMap::new();
                    for (k, v) in src {
                        if let Some(nk) = mapping.get(&k) {
                            out.insert(nk.clone(), v);
                        }
                    }
                    out
                };
                let remap_hash_string =
                    |src: HashMap<Vec<usize>, String>| -> HashMap<Vec<usize>, String> {
                        let mut out = HashMap::new();
                        for (k, v) in src {
                            if let Some(nk) = mapping.get(&k) {
                                out.insert(nk.clone(), v);
                            }
                        }
                        out
                    };
                let remap_hash_edge_style = |src: HashMap<
                    Vec<usize>,
                    crate::apps::mindmap::state::EdgeStyle,
                >|
                 -> HashMap<
                    Vec<usize>,
                    crate::apps::mindmap::state::EdgeStyle,
                > {
                    let mut out = HashMap::new();
                    for (k, v) in src {
                        if let Some(nk) = mapping.get(&k) {
                            out.insert(nk.clone(), v);
                        }
                    }
                    out
                };

                let mut new_collapsed = std::collections::HashSet::new();
                for p in old_collapsed_paths {
                    if let Some(np) = mapping.get(&p) {
                        new_collapsed.insert(np.clone());
                    }
                }

                tab.doc = new_doc;
                tab.selected_path = None;
                tab.node_positions.clear();
                tab.node_fills = remap_hash(old_node_fills);
                tab.node_text_colors = remap_hash(old_node_text_colors);
                tab.node_border_colors = remap_hash(old_node_border_colors);
                tab.node_border_styles = remap_hash_edge_style(old_node_border_styles);
                tab.node_priorities = remap_hash_u8(old_node_priorities);
                tab.node_urls = remap_hash_string(old_node_urls);
                tab.edge_styles = remap_hash_edge_style(old_edge_styles);
                tab.edge_colors = remap_hash(old_edge_colors);
                tab.collapsed_paths = new_collapsed;
                tab.canvas_cache.clear();
                let _ = persist(app);
            }
            other => {
                tab.markdown_import_editor.perform(other);
            }
        }
    }
    Task::none()
}

/// 构建或更新 apply markdown import 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn apply_markdown_import(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let md = tab.markdown_import_editor.text();
        let old_doc = tab.doc.clone();
        let old_node_fills = tab.node_fills.clone();
        let old_node_text_colors = tab.node_text_colors.clone();
        let old_node_border_colors = tab.node_border_colors.clone();
        let old_node_border_styles = tab.node_border_styles.clone();
        let old_node_priorities = tab.node_priorities.clone();
        let old_node_urls = tab.node_urls.clone();
        let old_edge_styles = tab.edge_styles.clone();
        let old_edge_colors = tab.edge_colors.clone();
        let old_collapsed_paths = tab.collapsed_paths.clone();

        tab.undo_stack.push(tab.doc.clone());
        tab.redo_stack.clear();
        if tab.undo_stack.len() > 50 {
            tab.undo_stack.remove(0);
        }

        let new_doc =
            if md.trim().is_empty() { model::default_doc() } else { mind_map::parse(&md) };

        fn match_paths(
            old: &mind_map::MindNode,
            new: &mind_map::MindNode,
        ) -> HashMap<Vec<usize>, Vec<usize>> {
            fn go(
                old: &mind_map::MindNode,
                old_path: &Vec<usize>,
                new: &mind_map::MindNode,
                new_path: &Vec<usize>,
                out: &mut HashMap<Vec<usize>, Vec<usize>>,
            ) {
                let mut old_by_text: HashMap<String, VecDeque<usize>> = HashMap::new();
                for (i, c) in old.children.iter().enumerate() {
                    old_by_text.entry(c.text.trim().to_string()).or_default().push_back(i);
                }

                for (j, c_new) in new.children.iter().enumerate() {
                    let key = c_new.text.trim().to_string();
                    let Some(q) = old_by_text.get_mut(&key) else {
                        continue;
                    };
                    let Some(i) = q.pop_front() else {
                        continue;
                    };

                    let mut op = old_path.clone();
                    op.push(i);
                    let mut np = new_path.clone();
                    np.push(j);

                    out.insert(op.clone(), np.clone());
                    go(&old.children[i], &op, c_new, &np, out);
                }
            }

            let mut out = HashMap::new();
            out.insert(Vec::new(), Vec::new());
            go(old, &Vec::new(), new, &Vec::new(), &mut out);
            out
        }

        let mapping = match_paths(&old_doc, &new_doc);

        let remap_hash = |src: HashMap<Vec<usize>, u32>| -> HashMap<Vec<usize>, u32> {
            let mut out = HashMap::new();
            for (k, v) in src {
                if let Some(nk) = mapping.get(&k) {
                    out.insert(nk.clone(), v);
                }
            }
            out
        };
        let remap_hash_u8 = |src: HashMap<Vec<usize>, u8>| -> HashMap<Vec<usize>, u8> {
            let mut out = HashMap::new();
            for (k, v) in src {
                if let Some(nk) = mapping.get(&k) {
                    out.insert(nk.clone(), v);
                }
            }
            out
        };
        let remap_hash_string = |src: HashMap<Vec<usize>, String>| -> HashMap<Vec<usize>, String> {
            let mut out = HashMap::new();
            for (k, v) in src {
                if let Some(nk) = mapping.get(&k) {
                    out.insert(nk.clone(), v);
                }
            }
            out
        };
        let remap_hash_edge_style = |src: HashMap<
            Vec<usize>,
            crate::apps::mindmap::state::EdgeStyle,
        >|
         -> HashMap<
            Vec<usize>,
            crate::apps::mindmap::state::EdgeStyle,
        > {
            let mut out = HashMap::new();
            for (k, v) in src {
                if let Some(nk) = mapping.get(&k) {
                    out.insert(nk.clone(), v);
                }
            }
            out
        };

        let mut new_collapsed = std::collections::HashSet::new();
        for p in old_collapsed_paths {
            if let Some(np) = mapping.get(&p) {
                new_collapsed.insert(np.clone());
            }
        }

        tab.doc = new_doc;
        tab.selected_path = None;
        tab.node_positions.clear();
        tab.node_fills = remap_hash(old_node_fills);
        tab.node_text_colors = remap_hash(old_node_text_colors);
        tab.node_border_colors = remap_hash(old_node_border_colors);
        tab.node_border_styles = remap_hash_edge_style(old_node_border_styles);
        tab.node_priorities = remap_hash_u8(old_node_priorities);
        tab.node_urls = remap_hash_string(old_node_urls);
        tab.edge_styles = remap_hash_edge_style(old_edge_styles);
        tab.edge_colors = remap_hash(old_edge_colors);
        tab.collapsed_paths = new_collapsed;
        tab.show_diagram_type_picker = false;
        tab.show_markdown_import = false;
        tab.show_zoom_menu = false;
        tab.show_priority_picker = false;
        tab.show_url_editor = false;
        tab.show_text_editor = false;
        tab.url_editor_value.clear();
        tab.node_text_editor = text_editor::Content::new();
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}
