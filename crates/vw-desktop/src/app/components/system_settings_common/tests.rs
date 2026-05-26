use super::{bool_support_label, format_context_limit, url_encode};

#[test]
fn common_exports_keep_formatting_helpers_available() {
    assert_eq!(bool_support_label(true), "支持");
    assert_eq!(format_context_limit(8192), "8K");
    assert_eq!(url_encode("a b"), "a%20b");
}
