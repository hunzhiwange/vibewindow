use super::{matches, split_path};
use glob::Pattern;

#[test]
fn split_path_accepts_unix_and_windows_separators() {
    assert_eq!(
        split_path("src\\nested/file.rs").collect::<Vec<_>>(),
        vec!["src", "nested", "file.rs"]
    );
}

#[test]
fn matches_honors_whitelist_before_default_rules() {
    let whitelist = [Pattern::new("node_modules/kept.js").expect("valid glob")];

    assert!(!matches("node_modules/kept.js", None, Some(&whitelist)));
    assert!(matches("node_modules/other.js", None, Some(&whitelist)));
}
