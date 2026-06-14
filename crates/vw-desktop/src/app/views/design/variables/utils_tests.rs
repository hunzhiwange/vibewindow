#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("utils_tests"));
}

fn variable(
    kind: &str,
    collection: Option<&str>,
    values: Vec<(&str, Option<&str>)>,
) -> crate::app::views::design::models::VariableDef {
    crate::app::views::design::models::VariableDef {
        kind: kind.to_string(),
        collection: collection.map(str::to_string),
        value: values
            .into_iter()
            .map(|(value, theme)| crate::app::views::design::models::VariableValue {
                value: value.to_string(),
                theme: theme.map(|mode| crate::app::views::design::models::ThemeCondition {
                    mode: mode.to_string(),
                }),
            })
            .collect(),
    }
}

#[test]
fn variable_collection_and_value_helpers_match_case_insensitively() {
    let def = variable("color", Some("Brand"), vec![("#fff", None), ("#000", Some("Dark"))]);

    assert!(super::variable_belongs_to_collection(&def, "brand"));
    assert!(!super::variable_belongs_to_collection(&def, "other"));
    assert_eq!(super::direct_variable_value(&def, None), "#fff");
    assert_eq!(super::direct_variable_value(&def, Some("dark")), "#000");
    assert_eq!(super::direct_variable_value(&def, Some("missing")), "");
}

#[test]
fn color_input_helpers_preserve_alpha_and_normalize_hex() {
    let parsed = super::parse_hex_color("33669980").expect("hex with alpha should parse");
    assert!((parsed.a - 128.0 / 255.0).abs() < 0.001);
    assert!(super::parse_hex_color("bad").is_none());

    assert_eq!(super::color_hex_input_value("#33669980"), "#336699");
    assert_eq!(super::color_hex_input_value(""), "#000000");
    assert_eq!(super::color_hex_input_value("oops"), "oops");
    assert_eq!(super::color_alpha_input_value("#33669980"), "50");
    assert_eq!(super::color_alpha_input_value("oops"), "100");

    assert_eq!(super::update_color_hex_value("#11223380", "abcdef"), "#ABCDEF80");
    assert_eq!(super::update_color_hex_value("#11223380", "bad"), "#bad");
    assert_eq!(super::update_color_alpha_value("#112233", "25"), "#11223340");
    assert_eq!(super::update_color_alpha_value("#112233", "999"), "#112233FF");
    assert_eq!(super::update_color_alpha_value("oops", "25"), "oops");
}

#[test]
fn swatch_border_uses_dark_outline_for_light_colors() {
    let light = super::swatch_border_color(iced::Color::WHITE);
    let dark = super::swatch_border_color(iced::Color::BLACK);

    assert!(light.r < dark.r);
    assert!(light.a < dark.a);
}
