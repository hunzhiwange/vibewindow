use super::*;

#[test]
fn chunk_text_returns_no_chunks_for_blank_input() {
    assert!(chunk_text("").is_empty());
    assert!(chunk_text(" \n\t\n ").is_empty());
}

#[test]
fn chunk_text_normalizes_line_whitespace() {
    let chunks = chunk_text_with_limits("  alpha  \n\n beta \n", 50, 0);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].ordinal, 0);
    assert_eq!(chunks[0].content, "alpha\n\nbeta");
}

#[test]
fn chunking_prefers_english_sentence_boundaries() {
    let chunks = chunk_text_with_limits("alpha beta. gamma delta. epsilon.", 18, 0);

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].content, "alpha beta.");
    assert_eq!(chunks[1].ordinal, 1);
}

#[test]
fn chunking_prefers_cjk_sentence_boundaries() {
    let chunks = chunk_text_with_limits("你好世界。后续内容继续。", 6, 0);

    assert_eq!(chunks[0].content, "你好世界。");
    assert!(chunks[1].content.starts_with("后续"));
}

#[test]
fn chunking_uses_overlap_between_adjacent_chunks() {
    let chunks = chunk_text_with_limits("abcdef", 3, 1);

    assert_eq!(
        chunks.iter().map(|chunk| chunk.content.as_str()).collect::<Vec<_>>(),
        vec!["abc", "cdef"]
    );
}

#[test]
fn chunk_and_overlap_limits_are_clamped_to_progress() {
    let chunks = chunk_text_with_limits("abc", 0, usize::MAX);

    assert_eq!(
        chunks.iter().map(|chunk| chunk.content.as_str()).collect::<Vec<_>>(),
        vec!["a", "bc"]
    );
}

#[test]
fn small_tail_is_folded_into_previous_chunk() {
    let chunks = chunk_text_with_limits("abcd", 3, 0);

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "abcd");
}

#[test]
fn private_helpers_cover_empty_and_boundary_cases() {
    assert!(remaining_fits_tail(&['a', 'b', 'c'], 0, 3));
    assert!(!remaining_fits_tail(&['a', 'b', 'c', 'd', 'e'], 0, 3));

    let chars = "one\ntwo".chars().collect::<Vec<_>>();
    assert_eq!(prefer_boundary(&chars, 0, 5), 4);
    assert_eq!(prefer_boundary(&['a', 'b', 'c'], 0, 3), 3);
    assert_eq!(normalize_text("  a \n b  "), "a\nb");
}
