#[test]
fn manager_tests_module_is_wired() {
    assert!(module_path!().ends_with("manager_tests"));
}

use iced_code_editor::{LspEvent, LspPosition, LspRange, LspTextChange};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};

#[test]
fn text_model_tracks_empty_text_and_utf16_columns() {
    let model = super::TextModel::from_text("");
    assert_eq!(model.lines, vec!["".to_string()]);

    let model = super::TextModel::from_text("a😀b\nsecond");
    assert_eq!(model.to_utf16_position(LspPosition { line: 0, character: 3 }).character, 4);
    assert_eq!(model.to_utf16_position(LspPosition { line: 8, character: 3 }).character, 3);
}

#[test]
fn text_model_applies_single_and_multiline_changes() {
    let mut model = super::TextModel::from_text("hello\nworld");
    model.apply_change(&LspTextChange {
        range: LspRange {
            start: LspPosition { line: 0, character: 1 },
            end: LspPosition { line: 0, character: 4 },
        },
        text: "i".to_string(),
    });
    assert_eq!(model.lines, vec!["hio".to_string(), "world".to_string()]);

    model.apply_change(&LspTextChange {
        range: LspRange {
            start: LspPosition { line: 0, character: 2 },
            end: LspPosition { line: 1, character: 3 },
        },
        text: "!\nnew".to_string(),
    });
    assert_eq!(model.lines, vec!["hi!".to_string(), "newld".to_string()]);
}

#[test]
fn char_to_byte_index_clamps_past_end_and_handles_multibyte() {
    assert_eq!(super::char_to_byte_index("a😀b", 2), 5);
    assert_eq!(super::char_to_byte_index("a😀b", 99), "a😀b".len());
}

#[test]
fn parse_hover_text_supports_strings_arrays_and_marked_strings() {
    assert_eq!(super::parse_hover_text(&json!({"contents": "plain"})), Some("plain".to_string()));
    assert_eq!(
        super::parse_hover_text(&json!({"contents": [{"value": "one"}, "two"]})),
        Some("one\ntwo".to_string())
    );
    assert_eq!(super::parse_hover_text(&json!({"contents": []})), None);
}

#[test]
fn parse_completion_items_accepts_array_or_list_shape() {
    assert_eq!(
        super::parse_completion_items(&json!([{"label": "alpha"}, {"label": 2}])),
        vec!["alpha".to_string()]
    );
    assert_eq!(
        super::parse_completion_items(&json!({"items": [{"label": "beta"}]})),
        vec!["beta".to_string()]
    );
}

#[test]
fn parse_definition_location_accepts_location_and_link_shapes() {
    let location = json!({
        "uri": "file:///a.rs",
        "range": {"start": {"line": 1, "character": 2}, "end": {"line": 3, "character": 4}}
    });
    let (uri, range) = super::parse_definition_location(&location).unwrap();
    assert_eq!(uri, "file:///a.rs");
    assert_eq!(range.start.line, 1);
    assert_eq!(range.end.character, 4);

    let link = json!([{
        "targetUri": "file:///b.rs",
        "targetSelectionRange": {
            "start": {"line": 5, "character": 6},
            "end": {"line": 7, "character": 8}
        }
    }]);
    let (uri, range) = super::parse_definition_location(&link).unwrap();
    assert_eq!(uri, "file:///b.rs");
    assert_eq!(range.start.character, 6);
}

#[test]
fn handle_server_request_acknowledges_progress_create_only() {
    let (tx, rx) = mpsc::channel();
    super::handle_server_request(7, "window/workDoneProgress/create", &tx);
    let payload = String::from_utf8(rx.try_recv().unwrap()).unwrap();
    assert!(payload.starts_with("Content-Length:"));
    assert!(payload.contains("\"id\":7"));

    super::handle_server_request(8, "workspace/configuration", &tx);
    assert!(rx.try_recv().is_err());
}

#[test]
fn handle_client_response_emits_matching_events_and_clears_pending() {
    let pending = Arc::new(Mutex::new(HashMap::from([
        (1, super::LspRequestKind::Hover),
        (2, super::LspRequestKind::Completion),
        (3, super::LspRequestKind::Definition),
    ])));
    let (tx, rx) = mpsc::channel();

    super::handle_client_response(1, &json!({"result": {"contents": "hover"}}), &pending, &tx);
    super::handle_client_response(2, &json!({"result": [{"label": "item"}]}), &pending, &tx);
    super::handle_client_response(
        3,
        &json!({"result": {"uri": "file:///main.rs", "range": {
            "start": {"line": 0, "character": 1},
            "end": {"line": 0, "character": 2}
        }}}),
        &pending,
        &tx,
    );

    assert!(matches!(rx.try_recv().unwrap(), LspEvent::Hover { text } if text == "hover"));
    assert!(
        matches!(rx.try_recv().unwrap(), LspEvent::Completion { items } if items == vec!["item"])
    );
    assert!(
        matches!(rx.try_recv().unwrap(), LspEvent::Definition { uri, .. } if uri == "file:///main.rs")
    );
    assert!(pending.lock().unwrap().is_empty());
}

#[test]
fn handle_server_notification_emits_progress_for_numeric_tokens() {
    let (tx, rx) = mpsc::channel();

    super::handle_server_notification(
        "$/progress",
        &json!({"token": 9, "value": {
            "kind": "end",
            "title": "index",
            "message": "done",
            "percentage": 100
        }}),
        &tx,
        "rust-analyzer",
    );

    assert!(matches!(
        rx.try_recv().unwrap(),
        LspEvent::Progress { token, server_key, title, message, percentage, done }
            if token == "9"
                && server_key == "rust-analyzer"
                && title == "index"
                && message.as_deref() == Some("done")
                && percentage == Some(100)
                && done
    ));

    super::handle_server_notification("other", &json!({"token": "x", "value": {}}), &tx, "server");
    assert!(rx.try_recv().is_err());
}

#[test]
fn detect_project_servers_returns_sorted_unique_server_keys() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname='x'\nversion='0.1.0'\n")
        .unwrap();
    std::fs::write(dir.path().join("src").join("main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.path().join("index.ts"), "const x = 1;\n").unwrap();

    let servers = super::detect_project_servers(dir.path().to_str().unwrap());

    assert!(servers.windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(servers.contains(&"rust-analyzer".to_string()));
}
