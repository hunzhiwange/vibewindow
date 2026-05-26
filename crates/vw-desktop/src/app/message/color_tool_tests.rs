#[test]
fn parse_hsl_accepts_hsl_and_hsla_with_clamps() {
    assert_eq!(super::parse_hsl("hsl(180, 50%, 75%)"), Some((180.0, 0.5, 0.75, 1.0)));
    assert_eq!(super::parse_hsl("hsla(-90, 125%, -5%, 1.5)"), Some((270.0, 1.0, 0.0, 1.0)));
}

#[test]
fn parse_hsl_rejects_malformed_input() {
    assert_eq!(super::parse_hsl("rgb(1, 2, 3)"), None);
    assert_eq!(super::parse_hsl("hsl(1, 2%)"), None);
    assert_eq!(super::parse_hsl("hsla(1, 2%, 3%, nope)"), None);
}

#[test]
fn parse_hsv_accepts_hsv_and_hsva_with_clamps() {
    assert_eq!(super::parse_hsv("hsv(360, 50%, 25%)"), Some((0.0, 0.5, 0.25, 1.0)));
    assert_eq!(super::parse_hsv("hsva(720, -10%, 150%, -1)"), Some((0.0, 0.0, 1.0, 0.0)));
}

#[test]
fn normalize_h_only_wraps_one_negative_turn_and_mods_positive_values() {
    assert_eq!(super::normalize_h(-90.0), 270.0);
    assert_eq!(super::normalize_h(720.0), 0.0);
    assert_eq!(super::normalize_h(42.0), 42.0);
}

#[test]
fn format_hsl_and_hsv_round_percent_components() {
    assert_eq!(super::format_hsla(12.4, 0.456, 0.789, 0.5), "hsla(12, 46%, 79%, 0.50)");
    assert_eq!(super::format_hsva(12.6, 0.454, 0.781, 1.0), "hsva(13, 45%, 78%, 1.00)");
}
