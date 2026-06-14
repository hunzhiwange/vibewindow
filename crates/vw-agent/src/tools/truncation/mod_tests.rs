use super::*;

#[test]
fn preview_and_output_cover_head_tail_byte_and_no_truncation() {
    let defaults = Options::default();
    assert_eq!(defaults.max_lines, MAX_LINES);
    assert_eq!(defaults.max_bytes, MAX_BYTES);
    assert_eq!(defaults.direction, Direction::Head);

    let head = compute_preview(
        "a\nb\nc",
        &Options { max_lines: 2, max_bytes: 99, direction: Direction::Head },
    );
    assert_eq!(head.text, "a\nb");
    assert!(!head.hit_bytes);

    let tail = compute_preview(
        "a\nb\nc",
        &Options { max_lines: 2, max_bytes: 99, direction: Direction::Tail },
    );
    assert_eq!(tail.text, "b\nc");

    let bytes = compute_preview(
        "abcd",
        &Options { max_lines: 9, max_bytes: 1, direction: Direction::Head },
    );
    assert!(bytes.hit_bytes);

    let original =
        output("short", Options { max_lines: 1, max_bytes: 99, direction: Direction::Head }, None);
    assert!(!original.truncated);
    assert_eq!(original.content, "short");

    let truncated = output(
        "a\nb\nc",
        Options { max_lines: 1, max_bytes: 99, direction: Direction::Head },
        None,
    );
    assert!(truncated.truncated);
    assert!(truncated.content.contains("已截断 2 lines"));
}

#[test]
fn cleanup_is_tolerant() {
    cleanup().unwrap();
}
