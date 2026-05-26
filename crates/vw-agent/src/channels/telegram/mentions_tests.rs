use super::TelegramChannel;

#[test]
fn mention_spans_require_username_boundaries() {
    let spans = TelegramChannel::find_bot_mention_spans("hi @VibeBot and no@vibebot", "vibebot");

    assert_eq!(spans, vec![(3, 11)]);
    assert!(TelegramChannel::contains_bot_mention("hi @vibebot", "@VibeBot"));
}

#[test]
fn normalize_incoming_content_removes_mentions_and_collapses_space() {
    assert_eq!(
        TelegramChannel::normalize_incoming_content(" @bot   run  this ", "bot").as_deref(),
        Some("run this")
    );
    assert_eq!(TelegramChannel::normalize_incoming_content("@bot", "bot"), None);
}
