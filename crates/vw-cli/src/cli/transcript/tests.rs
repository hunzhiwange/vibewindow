//! 验证 CLI 转录渲染的回归行为。
//! 测试覆盖消息片段、工具卡片和文本边界，确保终端输出保持可读。

use super::{
    ThinkBlockMeta, TranscriptEntry, TranscriptRole, build_streaming_transcript_view,
    transcript_to_lines,
};
use std::collections::BTreeSet;

fn first_think_meta(think_map: &[Option<ThinkBlockMeta>]) -> ThinkBlockMeta {
    think_map.iter().find_map(|meta| *meta).expect("expected at least one think block")
}

#[test]
fn open_think_block_defaults_expanded_and_can_collapse() {
    let transcript = vec![TranscriptEntry::new(TranscriptRole::Assistant, "<think>\nalpha")];

    let (expanded_lines, think_map) =
        transcript_to_lines(&transcript, false, false, &BTreeSet::new(), "");
    assert_eq!(expanded_lines.len(), 3);

    let think_meta = first_think_meta(&think_map);
    let overrides = BTreeSet::from([think_meta.id]);
    let (collapsed_lines, _) = transcript_to_lines(&transcript, false, false, &overrides, "");
    assert_eq!(collapsed_lines.len(), 2);
}

#[test]
fn closed_think_block_defaults_collapsed_and_can_expand() {
    let transcript =
        vec![TranscriptEntry::new(TranscriptRole::Assistant, "<think>\nalpha\n</think>")];

    let (collapsed_lines, think_map) =
        transcript_to_lines(&transcript, false, false, &BTreeSet::new(), "");
    assert_eq!(collapsed_lines.len(), 2);

    let think_meta = first_think_meta(&think_map);
    let overrides = BTreeSet::from([think_meta.id]);
    let (expanded_lines, _) = transcript_to_lines(&transcript, false, false, &overrides, "");
    assert_eq!(expanded_lines.len(), 3);
}

#[test]
fn streaming_think_block_id_stays_stable_across_rebuilds() {
    let transcript = vec![TranscriptEntry::new(TranscriptRole::User, "hi")];
    let draft = "<think>\nalpha";

    let (streaming_view_a, remaining_a) =
        build_streaming_transcript_view(&transcript, draft, false);
    assert!(remaining_a.is_empty());
    let (_, think_map_a) =
        transcript_to_lines(&streaming_view_a, false, false, &BTreeSet::new(), "");
    let think_id_a = first_think_meta(&think_map_a).id;

    let (streaming_view_b, remaining_b) =
        build_streaming_transcript_view(&transcript, draft, false);
    assert!(remaining_b.is_empty());
    let (_, think_map_b) =
        transcript_to_lines(&streaming_view_b, false, false, &BTreeSet::new(), "");
    let think_id_b = first_think_meta(&think_map_b).id;

    assert_eq!(think_id_a, think_id_b);
}
