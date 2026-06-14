use super::{has_priority, has_url, node_size};
use std::collections::HashMap;

#[test]
fn node_size_uses_root_metrics_and_decorations() {
    let plain = node_size("root", false, false, true);
    let decorated = node_size("root", true, true, true);

    assert_eq!(plain.width, 104.0);
    assert_eq!(plain.height, 50.0);
    assert_eq!(decorated.width, plain.width + 64.0);
}

#[test]
fn node_size_counts_lines_and_clamps_width() {
    let multiline = node_size("a\nsecond line", false, false, false);
    let wide = node_size(&"x".repeat(120), false, false, false);

    assert_eq!(multiline.width, 26.0 + 11.0 * 12.0);
    assert_eq!(multiline.height, 34.0 + 18.0);
    assert_eq!(wide.width, 600.0);
}

#[test]
fn node_size_handles_empty_text_as_one_character() {
    let size = node_size("", false, false, false);

    assert_eq!(size.width, 80.0);
    assert_eq!(size.height, 34.0);
}

#[test]
fn priority_and_url_helpers_filter_invalid_values() {
    let mut priorities = HashMap::new();
    priorities.insert(vec![0], 1);
    priorities.insert(vec![1], 10);
    priorities.insert(vec![2], 0);
    priorities.insert(vec![3], 11);

    let mut urls = HashMap::new();
    urls.insert(vec![0], " https://example.com ".to_string());
    urls.insert(vec![1], "   ".to_string());

    assert!(has_priority(&priorities, &[0]));
    assert!(has_priority(&priorities, &[1]));
    assert!(!has_priority(&priorities, &[2]));
    assert!(!has_priority(&priorities, &[3]));
    assert!(!has_priority(&priorities, &[4]));
    assert!(has_url(&urls, &[0]));
    assert!(!has_url(&urls, &[1]));
    assert!(!has_url(&urls, &[2]));
}
