#[test]
fn task_1196_test_module_is_wired() {}

#[test]
fn font_weight_helpers_cover_known_and_custom_labels() {
    assert_eq!(super::font_weight_label(&None), "Regular");
    assert_eq!(super::font_weight_label(&Some(serde_json::json!("300"))), "Light");
    assert_eq!(super::font_weight_label(&Some(serde_json::json!("400"))), "Regular");
    assert_eq!(super::font_weight_label(&Some(serde_json::json!("500"))), "Medium");
    assert_eq!(super::font_weight_label(&Some(serde_json::json!("600"))), "Semi Bold");
    assert_eq!(super::font_weight_label(&Some(serde_json::json!("700"))), "Bold");
    assert_eq!(super::font_weight_label(&Some(serde_json::json!("800"))), "Extra Bold");
    assert_eq!(super::font_weight_label(&Some(serde_json::json!("950"))), "950");

    assert_eq!(super::font_weight_value_from_label("Light"), "300");
    assert_eq!(super::font_weight_value_from_label("Regular"), "400");
    assert_eq!(super::font_weight_value_from_label("Medium"), "500");
    assert_eq!(super::font_weight_value_from_label("Semi Bold"), "600");
    assert_eq!(super::font_weight_value_from_label("Bold"), "700");
    assert_eq!(super::font_weight_value_from_label("Extra Bold"), "800");
    assert_eq!(super::font_weight_value_from_label("950"), "950");
}

#[test]
fn available_weight_options_match_common_font_families() {
    assert!(super::available_weights_for_font("").is_empty());
    assert_eq!(
        super::available_weights_for_font("Inter").first().map(String::as_str),
        Some("Light")
    );
    assert!(super::available_weights_for_font("PingFang SC").contains(&"Medium".to_string()));
    assert_eq!(super::available_weights_for_font("Noto Sans"), vec!["Regular", "Bold"]);
    assert_eq!(super::available_weights_for_font("Microsoft YaHei"), vec!["Regular", "Bold"]);
    assert_eq!(super::available_weights_for_font("Arial"), vec!["Regular", "Bold"]);
    assert!(super::available_weights_for_font("Unknown Family").is_empty());
}

#[test]
fn system_font_scan_returns_sorted_unique_names() {
    let fonts = super::available_system_fonts();
    let mut sorted = fonts.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(fonts, sorted);
}
