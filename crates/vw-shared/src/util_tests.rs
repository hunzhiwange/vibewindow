#[test]
fn format_duration_uses_compact_boundaries() {
    assert_eq!(super::format_duration(0), "");
    assert_eq!(super::format_duration(59), "59s");
    assert_eq!(super::format_duration(61), "1m 1s");
    assert_eq!(super::format_duration(3600), "1h");
    assert_eq!(super::format_duration(604800), "~1 week");
}

#[test]
fn truncate_respects_character_boundaries() {
    assert_eq!(super::truncate("workflow", 4), "wor…");
    assert_eq!(super::truncate("节点编辑", 3), "节点…");
    assert_eq!(super::truncate("abc", 0), "");
    assert_eq!(super::truncate("abc", 1), "…");
}

#[test]
fn truncate_middle_preserves_edges() {
    assert_eq!(super::truncate_middle("abcdefghij", 5), "ab…ij");
    assert_eq!(super::truncate_middle("short", 5), "short");
}

#[test]
fn text_and_number_helpers_keep_expected_output() {
    assert_eq!(super::titlecase("hello workflow-agent"), "Hello Workflow-Agent");
    assert_eq!(super::pluralize(1, "{} item", "{} items"), "1 item");
    assert_eq!(super::pluralize(2, "{} item", "{} items"), "2 items");
    assert_eq!(super::number(1500.0), "1.5K");
    assert_eq!(super::number(2_000_000.0), "2.0M");
    assert_eq!(super::duration(65_000), "1m 5s");
}
