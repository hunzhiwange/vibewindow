#[test]
fn task_1182_test_module_is_wired() {}

use super::*;

fn stop(color: &str, position: f64) -> GradientStop {
    GradientStop { color: color.to_string(), position }
}

fn assert_color_close(actual: Color, expected: Color) {
    let eps = 0.01;
    assert!((actual.r - expected.r).abs() <= eps, "r: {} != {}", actual.r, expected.r);
    assert!((actual.g - expected.g).abs() <= eps, "g: {} != {}", actual.g, expected.g);
    assert!((actual.b - expected.b).abs() <= eps, "b: {} != {}", actual.b, expected.b);
    assert!((actual.a - expected.a).abs() <= eps, "a: {} != {}", actual.a, expected.a);
}

#[test]
fn normalize_hex_color_input_expands_valid_short_forms() {
    assert_eq!(normalize_hex_color_input(" #fff "), "#ffffff");
    assert_eq!(normalize_hex_color_input("abcd"), "#aabbccdd");
    assert_eq!(normalize_hex_color_input("#ABCDEF"), "#abcdef");
    assert_eq!(normalize_hex_color_input("12345678"), "#12345678");
}

#[test]
fn normalize_hex_color_input_preserves_invalid_or_unusual_values() {
    assert_eq!(normalize_hex_color_input(""), "");
    assert_eq!(normalize_hex_color_input("red"), "red");
    assert_eq!(normalize_hex_color_input("#12"), "#12");
    assert_eq!(normalize_hex_color_input("12345"), "12345");
}

#[test]
fn format_percent_removes_trailing_zeroes() {
    assert_eq!(format_percent(50.0), "50");
    assert_eq!(format_percent(33.333), "33.33");
    assert_eq!(format_percent(25.5), "25.5");
    assert_eq!(format_percent(-1.2), "-1.2");
}

#[test]
fn hit_test_stop_detects_square_hit_area() {
    let stops = vec![stop("#000000", 0.0), stop("#ffffff", 0.5)];
    let bounds = Rectangle { x: 10.0, y: 20.0, width: 200.0, height: 40.0 };

    assert_eq!(hit_test_stop(&stops, bounds, Point::new(2.0, 20.0)), Some(0));
    assert_eq!(hit_test_stop(&stops, bounds, Point::new(100.0, 20.0)), Some(1));
    assert_eq!(hit_test_stop(&stops, bounds, Point::new(130.0, 20.0)), None);
    assert_eq!(hit_test_stop(&stops, bounds, Point::new(100.0, 32.0)), None);
}

#[test]
fn gradient_color_at_handles_empty_bounds_sorting_and_interpolation() {
    assert_color_close(gradient_color_at(&[], 0.5), Color::from_rgb(0.8, 0.8, 0.8));

    let stops = vec![stop("#ffffff", 1.0), stop("#000000", 0.0)];
    assert_color_close(gradient_color_at(&stops, -1.0), Color::BLACK);
    assert_color_close(gradient_color_at(&stops, 2.0), Color::WHITE);
    assert_color_close(gradient_color_at(&stops, 0.5), Color::from_rgb(0.5, 0.5, 0.5));
}

#[test]
fn gradient_color_at_handles_duplicate_positions_and_alpha() {
    let stops = vec![stop("#00000080", 0.0), stop("#ffffff80", 0.0), stop("#ffffffff", 1.0)];

    assert_color_close(gradient_color_at(&stops, 0.0), Color::from_rgba(0.0, 0.0, 0.0, 0.5));
    assert_color_close(gradient_color_at(&stops, 0.5), Color::from_rgba(1.0, 1.0, 1.0, 0.75));
}
