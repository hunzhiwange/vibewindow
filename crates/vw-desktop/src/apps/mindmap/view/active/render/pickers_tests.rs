use crate::app::views::design::models::ColorFormat;
use crate::apps::mindmap::model::default_doc;
use crate::apps::mindmap::state::{EdgeStyle, MindMapColorTarget, MindMapTab};
use iced::Color;

fn base_tab() -> MindMapTab {
    MindMapTab::new("tab".to_string(), "Pickers".to_string(), None, default_doc())
}

#[test]
fn color_picker_overlay_builds_without_style_row_for_plain_targets() {
    for target in
        [MindMapColorTarget::NodeFill, MindMapColorTarget::NodeText, MindMapColorTarget::Background]
    {
        let overlay = super::pickers::color_picker_overlay(
            "颜色",
            target,
            EdgeStyle::Solid,
            EdgeStyle::Dashed,
            Color::from_rgb(0.2, 0.4, 0.6),
            ColorFormat::Hex,
            false,
        );

        std::hint::black_box(overlay);
    }
}

#[test]
fn color_picker_overlay_builds_edge_style_row_for_all_active_styles() {
    for active_style in [EdgeStyle::Solid, EdgeStyle::Dashed, EdgeStyle::Dotted] {
        let overlay = super::pickers::color_picker_overlay(
            "线条",
            MindMapColorTarget::EdgeStroke,
            active_style,
            EdgeStyle::Solid,
            Color::from_rgb(0.1, 0.2, 0.3),
            ColorFormat::Rgba,
            true,
        );

        std::hint::black_box(overlay);
    }
}

#[test]
fn color_picker_overlay_builds_node_border_style_row_for_all_active_styles() {
    for active_style in [EdgeStyle::Solid, EdgeStyle::Dashed, EdgeStyle::Dotted] {
        let overlay = super::pickers::color_picker_overlay(
            "边框",
            MindMapColorTarget::NodeBorder,
            EdgeStyle::Solid,
            active_style,
            Color::from_rgb(0.8, 0.7, 0.6),
            ColorFormat::Hsl,
            false,
        );

        std::hint::black_box(overlay);
    }
}

#[test]
fn priority_picker_overlay_builds_empty_active_and_completed_states() {
    for priority in [None, Some(1), Some(5), Some(10)] {
        let overlay = super::pickers::priority_picker_overlay(priority);
        std::hint::black_box(overlay);
    }
}

#[test]
fn url_editor_overlay_builds_for_empty_and_existing_values() {
    let mut tab = base_tab();
    let empty = super::pickers::url_editor_overlay(&tab);
    std::hint::black_box(empty);

    tab.url_editor_value = "https://example.com/path?q=1".to_string();
    let existing = super::pickers::url_editor_overlay(&tab);
    std::hint::black_box(existing);
}

#[test]
fn text_editor_overlay_builds_editor_shell() {
    let tab = base_tab();
    let overlay = super::pickers::text_editor_overlay(&tab);

    std::hint::black_box(overlay);
}
