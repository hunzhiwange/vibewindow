use super::explore_summary_view::{right_aligned_slot_char, split_summary_segments, summary_animation_key};

#[test]
fn right_aligned_slot_char_pads_left_slots() {
    let chars = ['1', '2'];

    assert_eq!(right_aligned_slot_char(&chars, 0, 3), "");
    assert_eq!(right_aligned_slot_char(&chars, 1, 3), "1");
}

#[test]
fn summary_animation_key_packs_message_and_group() {
    assert_ne!(summary_animation_key(1, 2), summary_animation_key(1, 3));
}

#[test]
fn split_summary_segments_keeps_plain_text_and_numbers() {
    let segments = split_summary_segments("Read 12 files");

    assert!(segments.len() >= 2);
}
