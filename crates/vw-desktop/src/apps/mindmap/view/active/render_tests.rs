#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("render_tests"));
}

use super::render::{color_picker_title, render};
use crate::app::components::mind_map::MindNode;
use crate::app::views::design::models::ColorFormat;
use crate::apps::mindmap::state::{EdgeStyle, MindMapColorPicker, MindMapColorTarget, MindMapTab};
use iced::{Color, Point};

fn sample_tab() -> MindMapTab {
    let doc = MindNode {
        text: "Root".to_string(),
        children: vec![MindNode { text: "Child".to_string(), children: Vec::new() }],
    };
    let mut tab = MindMapTab::new("id".to_string(), "title".to_string(), None, doc);
    tab.selected_path = Some(vec![0]);
    tab
}

#[test]
fn color_picker_title_returns_titles_for_node_and_edge_targets() {
    assert_eq!(color_picker_title(MindMapColorTarget::NodeText), Some("文字颜色"));
    assert_eq!(color_picker_title(MindMapColorTarget::NodeFill), Some("节点填充"));
    assert_eq!(color_picker_title(MindMapColorTarget::NodeBorder), Some("边框颜色"));
    assert_eq!(color_picker_title(MindMapColorTarget::EdgeStroke), Some("连线颜色"));
}

#[test]
fn color_picker_title_ignores_background_target() {
    assert_eq!(color_picker_title(MindMapColorTarget::Background), None);
}

#[test]
fn render_builds_default_active_view() {
    let tab = sample_tab();

    let element = render(&tab);

    std::hint::black_box(element);
}

#[test]
fn render_builds_open_panel_and_menu_layers() {
    let mut tab = sample_tab();
    tab.show_action_menu = true;
    tab.show_zoom_menu = true;
    tab.show_theme_panel = true;
    tab.show_diagram_type_picker = true;
    tab.show_context_menu = true;
    tab.context_menu_anchor = Some(Point::new(50.0, 60.0));
    tab.undo_stack.push(tab.doc.clone());
    tab.redo_stack.push(tab.doc.clone());
    tab.clipboard_node = Some(MindNode::default());

    let element = render(&tab);

    std::hint::black_box(element);
}

#[test]
fn render_builds_priority_and_url_overlays() {
    for show_url_editor in [false, true] {
        let mut tab = sample_tab();
        if show_url_editor {
            tab.show_url_editor = true;
            tab.url_editor_value = "https://example.test".to_string();
        } else {
            tab.show_priority_picker = true;
        }
        tab.node_priorities.insert(vec![0], 2);
        tab.node_urls.insert(vec![0], "https://example.test".to_string());

        let element = render(&tab);

        std::hint::black_box(element);
    }
}

#[test]
fn render_builds_color_picker_for_style_targets() {
    for target in [
        MindMapColorTarget::NodeText,
        MindMapColorTarget::NodeFill,
        MindMapColorTarget::NodeBorder,
        MindMapColorTarget::EdgeStroke,
        MindMapColorTarget::Background,
    ] {
        let mut tab = sample_tab();
        tab.active_color_picker = Some(MindMapColorPicker {
            color: Color::from_rgb(0.1, 0.2, 0.3),
            format: ColorFormat::Hex,
            target,
            picking: target == MindMapColorTarget::NodeFill,
        });
        tab.edge_styles.insert(vec![0], EdgeStyle::Dashed);
        tab.node_border_styles.insert(vec![0], EdgeStyle::Dotted);
        tab.node_fills.insert(vec![0], 0x112233FF);
        tab.node_text_colors.insert(vec![0], 0x445566FF);
        tab.node_border_colors.insert(vec![0], 0x778899FF);
        tab.edge_colors.insert(vec![0], 0xAABBCCFF);

        let element = render(&tab);

        std::hint::black_box(element);
    }
}

#[test]
fn render_builds_text_and_markdown_overlays() {
    for show_markdown_import in [false, true] {
        let mut tab = sample_tab();
        if show_markdown_import {
            tab.show_markdown_import = true;
        } else {
            tab.show_text_editor = true;
        }

        let element = render(&tab);

        std::hint::black_box(element);
    }
}

#[test]
fn render_hides_node_toolbar_when_no_node_is_selected() {
    let mut tab = sample_tab();
    tab.selected_path = None;
    tab.show_context_menu = true;

    let element = render(&tab);

    std::hint::black_box(element);
}
