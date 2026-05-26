use super::{all, get, provide_with, remove, set};
use std::collections::HashMap;

#[test]
fn injected_environment_is_isolated_from_default_snapshot() {
    let mut vars = HashMap::new();
    vars.insert("VIBEWINDOW_TEST_KEY".to_string(), "one".to_string());

    provide_with(vars, || {
        assert_eq!(get("VIBEWINDOW_TEST_KEY").as_deref(), Some("one"));
        set("VIBEWINDOW_TEST_KEY", "two");
        assert_eq!(get("VIBEWINDOW_TEST_KEY").as_deref(), Some("two"));
        remove("VIBEWINDOW_TEST_KEY");
        assert_eq!(get("VIBEWINDOW_TEST_KEY"), None);
        assert!(!all().contains_key("VIBEWINDOW_TEST_KEY"));
    });
}
