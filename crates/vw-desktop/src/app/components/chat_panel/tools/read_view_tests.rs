use super::read_view::{parse_read_input, read_range_text};
use super::{tool_read_compact_view, tool_read_view};
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn parse_read_input_accepts_json_and_plain_paths() {
    assert_eq!(
        parse_read_input(r#"{"filePath":"src/main.rs","offset":2,"limit":5}"#),
        Some(("src/main.rs".to_string(), 2, 5))
    );
    assert_eq!(
        parse_read_input(r#"{"file_path":"docs/readme.md"}"#),
        Some(("docs/readme.md".to_string(), 0, 0))
    );
    assert_eq!(
        parse_read_input(r#"{"path":"file:///tmp/demo.txt","offset":1}"#),
        Some(("tmp/demo.txt".to_string(), 1, 0))
    );
    assert_eq!(parse_read_input("src/lib.rs"), Some(("src/lib.rs".to_string(), 0, 0)));
    assert_eq!(parse_read_input(r#"{"offset":2}"#), None);
    assert_eq!(parse_read_input(r#"{"path":""}"#), None);
}

#[test]
fn read_range_text_formats_offset_and_limit() {
    assert_eq!(read_range_text(2, 5), Some("offset=2 limit=5 (line 3-7)".to_string()));
    assert_eq!(read_range_text(2, 0), Some("offset=2 (from line 3)".to_string()));
    assert_eq!(read_range_text(0, 5), Some("limit=5 (line 1-5)".to_string()));
    assert_eq!(read_range_text(0, 0), None);
}

#[test]
fn read_views_accept_supported_tools_and_reject_bad_input() {
    let app = app();
    let visible = r#"tool read
{"input":"{\"filePath\":\"src/main.rs\",\"offset\":1,\"limit\":2}"}"#;
    let pdf_visible = r#"tool pdf_read
{"input":"docs/manual.pdf"}"#;

    assert!(tool_read_compact_view(&app, visible).is_some());
    assert!(tool_read_view(&app, 1, 2, visible).is_some());
    assert!(tool_read_view(&app, 1, 3, pdf_visible).is_some());
    assert!(tool_read_view(&app, 1, 4, "tool bash\n{}").is_none());
    assert!(tool_read_view(&app, 1, 5, "tool read\nnot-json").is_none());
    assert!(tool_read_view(&app, 1, 6, "read\n{}").is_none());
}
