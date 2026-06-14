#[test]
fn task_1178_test_module_is_wired() {}

use crate::app::views::design::models::DesignElement;

#[test]
fn render_export_panel_constructs_for_element_with_id() {
    let element =
        DesignElement { id: "shape-1".to_string(), kind: "rect".to_string(), ..Default::default() };

    let _panel = super::render(&element);
}

#[test]
fn render_export_panel_constructs_for_empty_id() {
    let element = DesignElement::default();

    let _panel = super::render(&element);
}
