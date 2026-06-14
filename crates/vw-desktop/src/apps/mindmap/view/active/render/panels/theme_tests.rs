use crate::apps::mindmap::canvas::theme::{CUSTOM_THEME_GROUP_ID, THEME_GROUPS};
use crate::apps::mindmap::model::default_doc;
use crate::apps::mindmap::state::MindMapTab;

fn base_tab() -> MindMapTab {
    MindMapTab::new("tab".to_string(), "Theme".to_string(), None, default_doc())
}

#[test]
fn theme_panel_builds_for_each_builtin_group() {
    for group in THEME_GROUPS {
        let mut tab = base_tab();
        tab.theme_group = group.id.to_string();
        tab.theme_variant = group.variants.len().saturating_sub(1);

        let panel = super::theme_panel(&tab, 360.0, 280.0);
        std::hint::black_box(panel);
    }
}

#[test]
fn theme_panel_falls_back_when_group_is_unknown() {
    let mut tab = base_tab();
    tab.theme_group = "missing-group".to_string();
    tab.theme_variant = 99;

    let panel = super::theme_panel(&tab, 320.0, 240.0);
    std::hint::black_box(panel);
}

#[test]
fn theme_panel_includes_conditional_cancel_and_delete_actions() {
    let mut tab = base_tab();
    tab.background = None;
    tab.follow_theme_background = true;

    let with_cancel = super::theme_panel(&tab, 360.0, 280.0);
    std::hint::black_box(with_cancel);

    tab.theme_group = CUSTOM_THEME_GROUP_ID.to_string();
    tab.theme_variant = 0;
    let with_delete = super::theme_panel(&tab, 360.0, 280.0);
    std::hint::black_box(with_delete);
}

#[test]
fn theme_panel_handles_custom_group_without_saved_themes() {
    let mut tab = base_tab();
    tab.theme_group = CUSTOM_THEME_GROUP_ID.to_string();
    tab.custom_themes.clear();
    tab.follow_theme_background = false;
    tab.background = Some(0xFFFFFFFF);

    let panel = super::theme_panel(&tab, 200.0, 160.0);
    std::hint::black_box(panel);
}
