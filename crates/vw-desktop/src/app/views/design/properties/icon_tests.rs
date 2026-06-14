#[test]
fn task_1190_test_module_is_wired() {}

#[test]
fn icon_display_and_weight_helpers_normalize_values() {
    assert_eq!(super::icon_display_name("arrow-left_circle"), "Arrow Left Circle");
    assert_eq!(super::icon_display_name("__"), "");

    assert_eq!(super::icon_weight_label(&None), "Regular");
    assert_eq!(super::icon_weight_label(&Some(serde_json::json!(100))), "Thin");
    assert_eq!(super::icon_weight_label(&Some(serde_json::json!("300"))), "Light");
    assert_eq!(super::icon_weight_label(&Some(serde_json::json!(500))), "Regular");
    assert_eq!(super::icon_weight_label(&Some(serde_json::json!(700))), "Bold");

    assert_eq!(super::icon_weight_value_from_label("Thin"), serde_json::json!(100));
    assert_eq!(super::icon_weight_value_from_label("Light"), serde_json::json!(300));
    assert_eq!(super::icon_weight_value_from_label("Regular"), serde_json::json!(400));
    assert_eq!(super::icon_weight_value_from_label("Bold"), serde_json::json!(700));
}

#[test]
fn icon_weight_options_are_family_specific_and_render_accepts_defaults() {
    assert!(super::icon_weight_options_for_family("lucide").is_empty());
    assert_eq!(
        super::icon_weight_options_for_family("phosphor"),
        vec!["Thin", "Light", "Regular", "Bold"]
    );

    let element = crate::app::views::design::models::DesignElement {
        id: "icon".to_string(),
        icon_font_family: Some("phosphor".to_string()),
        icon_font_name: Some("arrow-left".to_string()),
        weight: Some(serde_json::json!(700)),
        ..Default::default()
    };
    let _ = super::render(&element);
}
