#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("layout_tests"));
}

use super::layout::{LayoutInputs, build_layout};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::{MindMapColorTarget, MindMapTab};
use iced::{Point, Rectangle, Size};

fn inputs() -> LayoutInputs {
    LayoutInputs {
        action_menu_x: 10.0,
        action_menu_y: 20.0,
        action_bar_w: 300.0,
        action_bar_h: 40.0,
        action_menu_gap: 6.0,
        action_menu_w: 260.0,
        action_menu_h: 156.0,
        picker_left_gap: 18.0,
        picker_top_gap: 10.0,
        zoom_panel_margin: 14.0,
        zoom_control_w: 156.0,
        zoom_control_h: 32.0,
        zoom_menu_gap: 8.0,
        zoom_menu_padding: 6.0,
        zoom_menu_item_h: 28.0,
        zoom_menu_spacing: 2.0,
        zoom_menu_item_count: 4,
        bg_panel_w: 360.0,
        bg_panel_h: 250.0,
        bg_panel_margin: 14.0,
        diagram_panel_w: 620.0,
        diagram_panel_h: 260.0,
        diagram_panel_margin: 14.0,
        bg_color_panel_w: 280.0,
        bg_color_panel_h: 32.0,
        bg_color_panel_margin: 14.0,
    }
}

fn tab() -> MindMapTab {
    MindMapTab::new("id".to_string(), "title".to_string(), None, MindNode::default())
}

#[test]
fn build_layout_always_blocks_core_controls() {
    let tab = tab();
    let layout =
        build_layout(&tab, &inputs(), None, None, |_| Point::new(0.0, 0.0), |_, _| None, |_| None);

    assert_eq!(layout.default_side_anchor, Point::new(278.0, 60.0));
    assert_eq!(layout.priority_picker_anchor, layout.default_side_anchor);
    assert_eq!(layout.url_editor_anchor, layout.default_side_anchor);
    assert_eq!(layout.zoom_menu_anchor, Point::new(100_000.0, 46.0));
    assert_eq!(layout.ui_blocked_rects.len(), 3);
    assert_eq!(layout.ui_blocked_rects[0].x, 10.0);
    assert_eq!(layout.ui_blocked_rects[1].x, 100_000.0);
    assert_eq!(layout.ui_blocked_rects[2].x, 14.0);
}

#[test]
fn build_layout_adds_action_node_and_context_rects() {
    let mut tab = tab();
    tab.show_action_menu = true;
    tab.context_menu_anchor = Some(Point::new(40.0, 50.0));
    let toolbar = Rectangle::new(Point::new(80.0, 90.0), Size::new(200.0, 36.0));

    let layout = build_layout(
        &tab,
        &inputs(),
        Some(toolbar),
        None,
        |_| Point::new(0.0, 0.0),
        |_, _| None,
        |_| None,
    );

    assert!(layout.ui_blocked_rects.contains(&toolbar));
    assert!(
        layout
            .ui_blocked_rects
            .iter()
            .any(|rect| rect.x == 10.0 && rect.y == 66.0 && rect.width == 260.0)
    );
    assert!(
        layout
            .ui_blocked_rects
            .iter()
            .any(|rect| rect.x == 40.0 && rect.y == 56.0 && rect.width == 260.0)
    );
}

#[test]
fn build_layout_places_priority_picker_above_high_toolbar() {
    let mut tab = tab();
    tab.show_priority_picker = true;
    let toolbar = Rectangle::new(Point::new(100.0, 200.0), Size::new(240.0, 36.0));

    let layout = build_layout(
        &tab,
        &inputs(),
        Some(toolbar),
        None,
        |_| Point::new(0.0, 0.0),
        |_, _| Some(Point::new(110.0, 236.0)),
        |_| Some(Point::new(180.0, 200.0)),
    );

    assert_eq!(layout.priority_picker_anchor, Point::new(180.0, 200.0));
    assert!(layout.ui_blocked_rects.iter().any(|rect| rect.x == 110.0
        && rect.y == 40.0
        && rect.width == 140.0
        && rect.height == 150.0));
}

#[test]
fn build_layout_places_url_editor_below_low_toolbar() {
    let mut tab = tab();
    tab.show_url_editor = true;
    let toolbar = Rectangle::new(Point::new(100.0, 50.0), Size::new(240.0, 36.0));

    let layout = build_layout(
        &tab,
        &inputs(),
        Some(toolbar),
        None,
        |_| Point::new(0.0, 0.0),
        |_, _| Some(Point::new(60.0, 86.0)),
        |_| Some(Point::new(180.0, 50.0)),
    );

    assert!(layout.ui_blocked_rects.iter().any(|rect| rect.x == 60.0
        && rect.y == 96.0
        && rect.width == 350.0
        && rect.height == 84.0));
}

#[test]
fn build_layout_falls_back_to_top_anchor_when_bottom_anchor_missing() {
    let mut tab = tab();
    tab.show_priority_picker = true;

    let layout = build_layout(
        &tab,
        &inputs(),
        Some(Rectangle::new(Point::new(10.0, 20.0), Size::new(100.0, 30.0))),
        None,
        |_| Point::new(0.0, 0.0),
        |_, _| None,
        |_| Some(Point::new(180.0, 80.0)),
    );

    assert!(
        layout
            .ui_blocked_rects
            .iter()
            .any(|rect| rect.x == 110.0 && rect.y == -80.0 && rect.width == 140.0)
    );
}

#[test]
fn build_layout_blocks_fullscreen_for_text_editor_and_markdown_import() {
    for show_markdown_import in [false, true] {
        let mut tab = tab();
        if show_markdown_import {
            tab.show_markdown_import = true;
        } else {
            tab.show_text_editor = true;
        }

        let layout = build_layout(
            &tab,
            &inputs(),
            None,
            None,
            |_| Point::new(0.0, 0.0),
            |_, _| None,
            |_| None,
        );

        assert!(
            layout
                .ui_blocked_rects
                .iter()
                .any(|rect| rect.x == 0.0 && rect.y == 0.0 && rect.width == 100_000.0)
        );
    }
}

#[test]
fn build_layout_sizes_color_picker_by_target() {
    for (target, expected_h) in [
        (MindMapColorTarget::NodeText, 360.0),
        (MindMapColorTarget::NodeFill, 360.0),
        (MindMapColorTarget::NodeBorder, 400.0),
        (MindMapColorTarget::EdgeStroke, 400.0),
        (MindMapColorTarget::Background, 360.0),
    ] {
        let tab = tab();
        let layout = build_layout(
            &tab,
            &inputs(),
            None,
            Some(target),
            |_| Point::new(500.0, 300.0),
            |_, _| None,
            |_| None,
        );

        assert!(layout.ui_blocked_rects.iter().any(|rect| rect.x == 262.0
            && rect.y == 300.0 - expected_h / 2.0
            && rect.height == expected_h));
    }
}

#[test]
fn build_layout_adds_zoom_theme_and_diagram_panels_when_open() {
    let mut tab = tab();
    tab.show_zoom_menu = true;
    tab.show_theme_panel = true;
    tab.show_diagram_type_picker = true;

    let layout =
        build_layout(&tab, &inputs(), None, None, |_| Point::new(0.0, 0.0), |_, _| None, |_| None);

    assert!(
        layout
            .ui_blocked_rects
            .iter()
            .any(|rect| rect.x == 100_000.0)
    );
    assert!(layout.ui_blocked_rects.iter().any(|rect| rect.width == 374.0 && rect.height == 264.0));
    assert!(layout.ui_blocked_rects.iter().any(|rect| rect.width == 634.0 && rect.height == 274.0));
}
