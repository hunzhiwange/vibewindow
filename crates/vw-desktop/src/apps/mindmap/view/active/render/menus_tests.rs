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

    assert!(module.ends_with("menus_tests"));
}

#[test]
fn action_menu_overlay_builds_file_actions() {
    let tab = tab();
    let element = super::menus::action_menu_overlay(&tab, 260.0);

    assert!(child_count(&element) > 0);
}

#[test]
fn zoom_control_builds_for_common_and_fractional_zoom_values() {
    for zoom in [0.333, 1.0, 1.255, 10.0] {
        let mut tab = tab();
        tab.zoom = zoom;

        let element = super::menus::zoom_control(&tab, 122.0, 30.0);

        assert!(child_count(&element) > 0);
    }
}

#[test]
fn zoom_menu_overlay_builds_fit_item_and_presets() {
    let mut tab = tab();
    tab.zoom = 1.25;

    let element =
        super::menus::zoom_menu_overlay(&tab, 122.0, 28.0, 2.0, 8.0, &[50, 75, 100, 125, 200]);

    assert!(child_count(&element) > 0);
}

#[test]
fn zoom_menu_overlay_handles_empty_and_clamped_presets() {
    for zoom in [-2.0, 0.0, 1000.0] {
        let mut tab = tab();
        tab.zoom = zoom;

        let element = super::menus::zoom_menu_overlay(&tab, 90.0, 20.0, 0.0, 0.0, &[]);

        assert!(child_count(&element) > 0);
    }
}
