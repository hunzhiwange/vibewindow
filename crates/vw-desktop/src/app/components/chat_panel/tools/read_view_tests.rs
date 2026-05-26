use super::read_view::{parse_read_input, read_range_text};

#[test]
fn parse_read_input_accepts_json_and_plain_paths() {
    assert_eq!(
        parse_read_input(r#"{"filePath":"src/main.rs","offset":2,"limit":5}"#),
        Some(("src/main.rs".to_string(), 2, 5))
    );
    assert_eq!(parse_read_input("src/lib.rs"), Some(("src/lib.rs".to_string(), 0, 0)));
}

#[test]
fn read_range_text_formats_offset_and_limit() {
    assert_eq!(read_range_text(2, 5), Some("offset=2 limit=5 (line 3-7)".to_string()));
    assert_eq!(read_range_text(0, 0), None);
}
