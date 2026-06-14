#[test]
fn fallback_char_width_matches_ascii_and_wide_chars() {
    assert_eq!(super::fallback_char_advance_px('a', 10.0), 6.0);
    assert_eq!(super::fallback_char_advance_px('中', 10.0), 10.0);
}

#[test]
fn parse_size_accepts_numbers_and_pixel_strings() {
    assert_eq!(
        super::parse_size(&Some(serde_json::json!(12)), &std::collections::HashMap::new(), None),
        Some(12.0)
    );
    assert_eq!(
        super::parse_size(&Some(serde_json::json!("14")), &std::collections::HashMap::new(), None),
        Some(14.0)
    );
    assert_eq!(
        super::parse_size(&Some(serde_json::json!("bad")), &std::collections::HashMap::new(), None),
        None
    );
}

fn assert_color_close(actual: iced::Color, expected: iced::Color) {
    assert!((actual.r - expected.r).abs() < 0.001, "r: {actual:?} != {expected:?}");
    assert!((actual.g - expected.g).abs() < 0.001, "g: {actual:?} != {expected:?}");
    assert!((actual.b - expected.b).abs() < 0.001, "b: {actual:?} != {expected:?}");
    assert!((actual.a - expected.a).abs() < 0.001, "a: {actual:?} != {expected:?}");
}

fn variables() -> std::collections::HashMap<String, crate::app::views::design::models::VariableDef>
{
    use crate::app::views::design::models::{ThemeCondition, VariableDef, VariableValue};

    std::collections::HashMap::from([
        (
            "-size".to_string(),
            VariableDef {
                kind: "number".to_string(),
                collection: None,
                value: vec![VariableValue { value: "32".to_string(), theme: None }],
            },
        ),
        (
            "-color".to_string(),
            VariableDef {
                kind: "color".to_string(),
                collection: None,
                value: vec![
                    VariableValue { value: "#112233".to_string(), theme: None },
                    VariableValue {
                        value: "#445566".to_string(),
                        theme: Some(ThemeCondition { mode: "Dark".to_string() }),
                    },
                ],
            },
        ),
        (
            "-font".to_string(),
            VariableDef {
                kind: "font".to_string(),
                collection: None,
                value: vec![VariableValue { value: "Geist".to_string(), theme: None }],
            },
        ),
    ])
}

#[test]
fn intern_font_family_returns_default_for_blank_and_reuses_names() {
    assert_eq!(super::intern_font_family_name("  "), "JetBrains Mono");
    let a = super::intern_font_family_name("Custom Font");
    let b = super::intern_font_family_name("Custom Font");
    assert!(std::ptr::eq(a.as_ptr(), b.as_ptr()));
}

#[test]
fn text_measurement_handles_invalid_sizes_and_spacing() {
    assert_eq!(super::measure_text_width_with_font("abc", "Missing Font", 0.0, 0.0), 0.0);
    assert_eq!(
        super::wrap_text_lines_with_font("abc", 0.0, "Missing Font", 12.0, 0.0),
        Vec::<String>::new()
    );

    let width = super::measure_text_width_with_font("ab", "Definitely Missing", 10.0, 2.0);
    assert!((width - 14.0).abs() < 0.001);

    let lines =
        super::wrap_text_lines_with_font("alpha beta", 24.0, "Definitely Missing", 10.0, 0.0);
    assert_eq!(lines, vec!["alph".to_string(), "a".to_string(), "beta".to_string()]);
}

#[test]
fn parse_size_resolves_variables_and_ignores_fill_container() {
    let vars = variables();
    assert_eq!(super::parse_size(&Some(serde_json::json!("$-size")), &vars, None), Some(32.0));
    assert_eq!(super::parse_size(&Some(serde_json::json!("fill_container")), &vars, None), None);
    assert_eq!(super::parse_size(&None, &vars, None), None);
}

#[test]
fn parse_color_supports_hex_rgb_named_variables_and_fallbacks() {
    let vars = variables();

    assert_color_close(
        super::parse_color("#abc", &vars, None),
        iced::Color::from_rgb8(0xaa, 0xbb, 0xcc),
    );
    assert_color_close(
        super::parse_color("#abcd", &vars, None),
        iced::Color::from_rgba8(0xaa, 0xbb, 0xcc, 0xdd as f32 / 255.0),
    );
    assert_color_close(
        super::parse_color("#11223344", &vars, None),
        iced::Color::from_rgba8(0x11, 0x22, 0x33, 0x44 as f32 / 255.0),
    );
    assert_color_close(
        super::parse_color("rgba(1, 2, 3, 0.5)", &vars, None),
        iced::Color::from_rgba8(1, 2, 3, 0.5),
    );
    assert_color_close(super::parse_color("navy", &vars, None), iced::Color::from_rgb8(0, 0, 128));
    assert_color_close(
        super::parse_color("$-color", &vars, Some("Dark")),
        iced::Color::from_rgb8(0x44, 0x55, 0x66),
    );
    assert_color_close(
        super::parse_color("$--primary", &vars, None),
        iced::Color::from_rgb8(255, 165, 0),
    );
    assert_color_close(
        super::parse_color("$-missing", &vars, None),
        iced::Color::from_rgb8(100, 100, 100),
    );
    assert_color_close(super::parse_color("???", &vars, None), iced::Color::TRANSPARENT);
    assert_color_close(
        super::parse_color("#badhex", &vars, None),
        iced::Color::from_rgb8(0xba, 0x00, 0x00),
    );
    assert_color_close(
        super::parse_color("#12", &vars, None),
        iced::Color::from_rgb(1.0, 0.0, 1.0),
    );
}

#[test]
fn parse_fill_and_fills_handle_arrays_objects_and_disabled_entries() {
    let vars = variables();

    assert_color_close(
        super::parse_fill(&Some(serde_json::json!(["not-a-color", "#010203"])), &vars, None),
        iced::Color::TRANSPARENT,
    );
    assert_color_close(
        super::parse_fill(
            &Some(serde_json::json!({ "color": "#102030", "opacity": 0.5 })),
            &vars,
            None,
        ),
        iced::Color::from_rgba8(0x10, 0x20, 0x30, 0.5),
    );
    assert_color_close(
        super::parse_fill(
            &Some(serde_json::json!({ "enabled": false, "color": "#ffffff" })),
            &vars,
            None,
        ),
        iced::Color::TRANSPARENT,
    );

    let fills = super::parse_fills(
        &Some(serde_json::json!([
            { "enabled": false, "color": "#ffffff" },
            { "colors": [{ "color": "#010203" }] },
            "red",
            4
        ])),
        &vars,
        None,
    );
    assert_eq!(fills.len(), 2);
    assert_color_close(fills[0], iced::Color::from_rgb8(1, 2, 3));
    assert_color_close(fills[1], iced::Color::from_rgb8(255, 0, 0));
    assert!(super::parse_fills(&None, &vars, None).is_empty());
}

#[test]
fn radii_parse_percent_arrays_variables_and_clamp_to_half_size() {
    let vars = variables();

    assert_eq!(super::parse_radius(&Some(serde_json::json!("50%")), 80.0, &vars, None), 40.0);
    assert_eq!(super::parse_radius(&Some(serde_json::json!("12px")), 80.0, &vars, None), 12.0);
    assert_eq!(super::parse_radius(&Some(serde_json::json!("$-size")), 80.0, &vars, None), 32.0);
    assert_eq!(super::parse_radius(&Some(serde_json::json!([7])), 80.0, &vars, None), 7.0);

    let radius = super::parse_corner_radii(
        &Some(serde_json::json!([100, 10, 20, 30])),
        40.0,
        80.0,
        &vars,
        None,
    );
    assert_eq!(radius.top_left, 20.0);
    assert_eq!(radius.top_right, 10.0);
    assert_eq!(radius.bottom_right, 20.0);
    assert_eq!(radius.bottom_left, 20.0);
}

#[test]
fn thickness_font_size_family_and_line_height_parse_common_shapes() {
    let vars = variables();
    let object = serde_json::json!({ "left": 4 });
    assert_eq!(super::parse_thickness(&Some(&object), &vars, None), 4.0);
    assert_eq!(super::parse_thickness(&Some(&serde_json::json!("$-size")), &vars, None), 1.0);
    assert_eq!(super::parse_thickness(&None, &vars, None), 1.0);

    assert_eq!(super::parse_font_size(&Some(serde_json::json!("$-size")), &vars, None), 32.0);
    assert_eq!(super::parse_font_size(&Some(serde_json::json!("bad")), &vars, None), 16.0);
    assert_eq!(
        super::resolve_font_family(&Some("$-font".to_string()), &vars, None),
        "Noto Sans CJK SC"
    );
    assert_eq!(super::resolve_font_family(&Some("Inter".to_string()), &vars, None), "Inter");

    assert_eq!(super::parse_line_height(&Some(serde_json::json!(1.5)), 20.0, &vars, None), 30.0);
    assert_eq!(super::parse_line_height(&Some(serde_json::json!(24)), 20.0, &vars, None), 24.0);
    assert_eq!(super::parse_line_height(&Some(serde_json::json!("150%")), 20.0, &vars, None), 30.0);
    assert_eq!(
        super::parse_line_height(&Some(serde_json::json!("$-size")), 20.0, &vars, None),
        32.0
    );
}

#[test]
fn stroke_color_and_shadow_parse_defaults_and_values() {
    let vars = variables();

    assert_color_close(
        super::resolve_stroke_color(Some("#010203"), &None, &vars, None),
        iced::Color::from_rgb8(1, 2, 3),
    );
    assert_color_close(
        super::resolve_stroke_color(None, &None, &vars, None),
        iced::Color::TRANSPARENT,
    );

    let shadow = super::parse_shadow(
        &Some(serde_json::json!({
            "color": "#010203",
            "offset_x": 2,
            "offset_y": 3,
            "blur": 4,
            "spread": 5
        })),
        &vars,
        None,
    )
    .expect("shadow");
    assert_color_close(shadow.color, iced::Color::from_rgb8(1, 2, 3));
    assert_eq!(
        (shadow.offset.x, shadow.offset.y, shadow.blur, shadow.spread),
        (2.0, 3.0, 4.0, 5.0)
    );
    assert!(super::parse_shadow(&Some(serde_json::json!(true)), &vars, None).is_none());
}
