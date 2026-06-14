#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("geometry_tests"));
}

use super::geometry::{
    node_action_btn_anchor, node_action_btn_bottom_left_anchor, node_action_btn_top_anchor,
    node_toolbar_layout, selected_node_rect,
};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapTab;
use iced::{Point, Rectangle, Size};

fn tab_with_selection(path: Option<Vec<usize>>) -> MindMapTab {
    let doc = MindNode {
        text: "Root".to_string(),
        children: vec![MindNode { text: "Child".to_string(), children: Vec::new() }],
    };
    let mut tab = MindMapTab::new("id".to_string(), "title".to_string(), None, doc);
    tab.selected_path = path;
    tab
}

#[test]
fn selected_node_rect_returns_none_without_selection() {
    let tab = tab_with_selection(None);

    assert!(selected_node_rect(&tab).is_none());
}

#[test]
fn selected_node_rect_returns_screen_rect_for_selected_root() {
    let tab = tab_with_selection(Some(Vec::new()));

    let rect = selected_node_rect(&tab).expect("selected root should have a rect");

    assert!(rect.width > 0.0);
    assert!(rect.height > 0.0);
    assert!(rect.x.is_finite());
    assert!(rect.y.is_finite());
}

#[test]
fn node_toolbar_layout_places_above_when_there_is_room() {
    let mut tab = tab_with_selection(Some(Vec::new()));
    tab.pan = iced::Vector::new(300.0, 300.0);

    let layout = node_toolbar_layout(&tab, 14.0, 42.0, 6.0, 240.0, 36.0, 10.0)
        .expect("selected node should produce toolbar layout");

    assert!(layout.place_above);
    assert_eq!(layout.rect.width, 240.0);
    assert_eq!(layout.rect.height, 36.0);
    assert_eq!(layout.anchor.y, layout.rect.y + layout.rect.height + 10.0);
}

#[test]
fn node_toolbar_layout_places_below_when_top_would_hit_action_bar() {
    let mut tab = tab_with_selection(Some(Vec::new()));
    tab.pan = iced::Vector::new(300.0, 40.0);

    let layout = node_toolbar_layout(&tab, 14.0, 42.0, 6.0, 240.0, 36.0, 10.0)
        .expect("selected node should produce toolbar layout");

    assert!(!layout.place_above);
    assert_eq!(layout.rect.y, layout.anchor.y + 10.0);
}

#[test]
fn node_toolbar_layout_returns_none_when_selection_is_missing() {
    let tab = tab_with_selection(None);

    assert!(node_toolbar_layout(&tab, 14.0, 42.0, 6.0, 240.0, 36.0, 10.0).is_none());
}

#[test]
fn node_action_btn_anchor_accumulates_widths_and_spacing() {
    let toolbar = Rectangle::new(Point::new(10.0, 20.0), Size::new(300.0, 40.0));
    let widths = [20.0, 30.0, 40.0];

    let anchor = node_action_btn_anchor(toolbar, &widths, 6.0, 100.0, 10.0, 1.0, 5.0, 2)
        .expect("third button exists");

    assert_eq!(anchor.x, 10.0 + 6.0 + 100.0 + 10.0 + 1.0 + 10.0 + 20.0 + 5.0 + 30.0 + 5.0 + 20.0);
    assert_eq!(anchor.y, 40.0);
}

#[test]
fn node_action_btn_anchor_rejects_out_of_range_index() {
    let toolbar = Rectangle::new(Point::new(10.0, 20.0), Size::new(300.0, 40.0));

    assert!(node_action_btn_anchor(toolbar, &[20.0], 6.0, 100.0, 10.0, 1.0, 5.0, 1).is_none());
}

#[test]
fn node_action_btn_top_anchor_uses_toolbar_top_y() {
    let toolbar = Rectangle::new(Point::new(10.0, 20.0), Size::new(300.0, 40.0));

    let anchor = node_action_btn_top_anchor(toolbar, &[20.0], 6.0, 100.0, 10.0, 1.0, 5.0, 0)
        .expect("first button exists");

    assert_eq!(anchor.y, 20.0);
}

#[test]
fn node_action_btn_bottom_left_anchor_offsets_by_overlay_half_width() {
    let toolbar = Rectangle::new(Point::new(10.0, 20.0), Size::new(300.0, 40.0));

    let center = node_action_btn_anchor(toolbar, &[20.0], 6.0, 100.0, 10.0, 1.0, 5.0, 0)
        .expect("first button exists");
    let anchor =
        node_action_btn_bottom_left_anchor(toolbar, &[20.0], 6.0, 100.0, 10.0, 1.0, 5.0, 0, 140.0)
            .expect("first button exists");

    assert_eq!(anchor.x, center.x - 70.0);
    assert_eq!(anchor.y, 60.0);
}
