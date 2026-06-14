use iced::advanced::text::highlighter::Highlighter;

use super::input_mention_highlighter::{
    Highlight, MentionHighlighter, Settings, mention_display_format, mention_format,
    mention_hidden_format,
};

#[test]
fn highlighter_finds_mentions_with_supported_characters() {
    let mut highlighter = MentionHighlighter::new(&Settings);
    let ranges: Vec<_> =
        highlighter.highlight_line("hello @user/work.space-1:#tag and @second\\path!").collect();

    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0].0, 6..29);
    assert!(ranges[0].1.is_mention);
    assert_eq!(ranges[1].0, 34..46);
    assert_eq!(highlighter.current_line(), 1);
}

#[test]
fn highlighter_ignores_bare_at_and_stops_on_invalid_characters() {
    let mut highlighter = MentionHighlighter::new(&Settings);
    let ranges: Vec<_> = highlighter.highlight_line("@ @ok, email a@b.com").collect();

    assert_eq!(ranges.len(), 2);
    assert_eq!(ranges[0].0, 2..5);
    assert_eq!(ranges[1].0, 14..20);
}

#[test]
fn line_tracking_can_change_update_and_saturate() {
    let mut highlighter = MentionHighlighter::new(&Settings);
    highlighter.change_line(41);
    assert_eq!(highlighter.current_line(), 41);

    highlighter.update(&Settings);
    assert_eq!(highlighter.current_line(), 0);

    highlighter.change_line(usize::MAX);
    let _: Vec<_> = highlighter.highlight_line("@max").collect();
    assert_eq!(highlighter.current_line(), usize::MAX);
}

#[test]
fn mention_formats_choose_visible_hidden_or_default_colors() {
    let theme = iced::Theme::Light;
    let mention = Highlight { is_mention: true };
    let plain = Highlight { is_mention: false };

    assert_eq!(mention_format(&mention, &theme).color, Some(theme.palette().primary));
    assert_eq!(mention_format(&plain, &theme).color, None);
    assert_eq!(mention_hidden_format(&mention, &theme).color, Some(iced::Color::TRANSPARENT));
    assert_eq!(mention_hidden_format(&plain, &theme).color, None);

    assert_eq!(mention_display_format(true)(&mention, &theme).color, Some(theme.palette().primary));
    assert_eq!(
        mention_display_format(false)(&mention, &theme).color,
        Some(iced::Color::TRANSPARENT)
    );
}
