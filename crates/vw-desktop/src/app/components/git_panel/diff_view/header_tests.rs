use iced::Color;

use crate::app::Message;
use crate::app::assets::Icon;
use crate::app::message;

#[test]
fn mix_color_clamps_ratio_and_interpolates_alpha() {
    let red = Color::from_rgba(1.0, 0.0, 0.0, 0.25);
    let blue = Color::from_rgba(0.0, 0.0, 1.0, 0.75);

    assert_eq!(super::header::mix_color(red, blue, -1.0), red);
    assert_eq!(super::header::mix_color(red, blue, 2.0), blue);

    let mixed = super::header::mix_color(red, blue, 0.5);
    assert_eq!(mixed, Color::from_rgba(0.5, 0.0, 0.5, 0.5));
}

#[test]
fn build_diff_header_covers_stats_close_and_fullscreen_combinations() {
    let close = Message::Git(message::GitMessage::CloseDiffFileMenu);
    let fullscreen = Message::Git(message::GitMessage::ToggleFullscreen);

    let _with_all_controls = super::header::build_diff_header(
        "src/lib.rs (+2 -1)".to_string(),
        2,
        1,
        Some(close.clone()),
        Some(fullscreen.clone()),
        Some("全屏".to_string()),
        Some(Icon::ArrowsFullscreen),
    );

    let _without_stats_or_close =
        super::header::build_diff_header("src/lib.rs".to_string(), 0, 0, None, None, None, None);

    let _partial_fullscreen_args_are_ignored = super::header::build_diff_header(
        "src/lib.rs （+3）".to_string(),
        3,
        0,
        None,
        Some(fullscreen),
        None,
        Some(Icon::ArrowsFullscreen),
    );
}

#[test]
fn wrap_diff_header_covers_neutral_added_deleted_and_mixed_styles() {
    for (insertions, deletions) in [(0, 0), (1, 0), (0, 1), (2, 3)] {
        let header = super::header::build_diff_header(
            "src/lib.rs".to_string(),
            insertions,
            deletions,
            None,
            None,
            None,
            None,
        );
        let _wrapped = super::header::wrap_diff_header(header, insertions, deletions);
    }
}
