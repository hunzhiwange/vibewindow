use super::*;

#[test]
fn read_like_tool_detects_explicit_kind_case_insensitively() {
    assert!(is_read_like_tool(&ReadLikeToolDescriptor {
        title: None,
        kind: Some(" READ ".to_string()),
    }));
}

#[test]
fn read_like_tool_infers_from_title_head_only() {
    assert!(is_read_like_tool(&ReadLikeToolDescriptor {
        title: Some("open: file".to_string()),
        kind: None,
    }));
    assert!(!is_read_like_tool(&ReadLikeToolDescriptor {
        title: Some("edit: then read".to_string()),
        kind: None,
    }));
}

#[test]
fn read_like_tool_ignores_missing_or_blank_title() {
    assert!(!is_read_like_tool(&ReadLikeToolDescriptor { title: None, kind: None }));
    assert!(!is_read_like_tool(&ReadLikeToolDescriptor {
        title: Some("   ".to_string()),
        kind: None,
    }));
}

#[test]
fn read_like_tool_ignores_empty_title_head() {
    assert!(!is_read_like_tool(&ReadLikeToolDescriptor {
        title: Some(": read file".to_string()),
        kind: None,
    }));
}
