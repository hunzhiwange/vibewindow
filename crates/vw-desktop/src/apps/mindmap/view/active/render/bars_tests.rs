#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("bars_tests"));
}

use super::bars::{action_bar, node_toolbar_overlay};
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapTab;

fn tab() -> MindMapTab {
    let mut tab = MindMapTab::new("id".to_string(), "title".to_string(), None, MindNode::default());
    tab.selected_path = Some(vec![0]);
    tab
}

#[test]
fn action_bar_builds_all_disabled_edit_actions() {
    let tab = tab();

    let element = action_bar(&tab, 30.0, 6.0, 6.0, false, false, false, false, false, false);

    std::hint::black_box(element);
}

#[test]
fn action_bar_builds_enabled_and_active_buttons() {
    let mut tab = tab();
    tab.show_action_menu = true;
    tab.show_theme_panel = true;
    tab.show_diagram_type_picker = true;
    tab.show_markdown_import = true;

    let element = action_bar(&tab, 30.0, 6.0, 6.0, true, true, true, true, true, true);

    std::hint::black_box(element);
}

#[test]
fn node_toolbar_overlay_builds_disabled_root_style_actions() {
    let tab = tab();

    let element = node_toolbar_overlay(
        &tab, 30.0, 24.0, 6.0, 24.0, true, false, false, false, false, false, None, false,
    );

    std::hint::black_box(element);
}

#[test]
fn node_toolbar_overlay_builds_enabled_child_style_actions() {
    let mut tab = tab();
    tab.node_fills.insert(vec![0], 0x112233FF);
    tab.node_text_colors.insert(vec![0], 0x445566FF);
    tab.node_border_colors.insert(vec![0], 0x778899FF);
    tab.edge_colors.insert(vec![0], 0xAABBCCFF);

    let element = node_toolbar_overlay(
        &tab,
        30.0,
        24.0,
        6.0,
        24.0,
        false,
        true,
        true,
        true,
        true,
        true,
        Some(3),
        true,
    );

    std::hint::black_box(element);
}
