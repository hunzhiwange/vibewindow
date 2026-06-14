use crate::app::Message;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::MindMapTab;

fn tab() -> MindMapTab {
    MindMapTab::new("tab-1".to_string(), "Mind map".to_string(), None, model::default_doc())
}

fn child_count(element: &iced::Element<'_, Message>) -> usize {
    element.as_widget().children().len()
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("background_tests"));
}

#[test]
fn background_panel_builds_follow_theme_state() {
    let mut tab = tab();
    tab.background = None;
    tab.follow_theme_background = true;

    let element = super::background::background_panel(&tab, 260.0, 36.0);

    assert!(child_count(&element) > 0);
}

#[test]
fn background_panel_builds_fixed_light_and_dark_states() {
    for background in [0xFFFFFFFF, 0x111827FF, 0x0B1220FF] {
        let mut tab = tab();
        tab.background = Some(background);
        tab.follow_theme_background = false;

        let element = super::background::background_panel(&tab, 260.0, 36.0);

        assert!(child_count(&element) > 0);
    }
}
