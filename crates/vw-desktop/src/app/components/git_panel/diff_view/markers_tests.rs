use iced::Color;

#[test]
fn marker_alpha_covers_all_marker_kinds_and_theme_modes() {
    assert_eq!(
        super::markers::emphasized_marker_alpha(super::markers::LineMarkerKind::None, false, true),
        0.0
    );
    assert_eq!(
        super::markers::emphasized_marker_alpha(super::markers::LineMarkerKind::None, true, true),
        0.34
    );
    assert_eq!(
        super::markers::emphasized_marker_alpha(super::markers::LineMarkerKind::Add, true, false),
        0.76
    );
    assert_eq!(
        super::markers::emphasized_marker_alpha(
            super::markers::LineMarkerKind::Delete,
            false,
            true
        ),
        0.66
    );
    assert_eq!(
        super::markers::emphasized_marker_alpha(
            super::markers::LineMarkerKind::Delete,
            false,
            false
        ),
        0.58
    );
}

#[test]
fn marker_mix_color_clamps_and_interpolates() {
    let a = Color::from_rgba(0.0, 0.0, 0.0, 0.2);
    let b = Color::from_rgba(1.0, 1.0, 1.0, 0.8);

    assert_eq!(super::markers::mix_color(a, b, -0.5), a);
    assert_eq!(super::markers::mix_color(a, b, 1.5), b);
    assert_eq!(super::markers::mix_color(a, b, 0.5), Color::from_rgba(0.5, 0.5, 0.5, 0.5));
}

#[test]
fn line_number_padding_and_cells_cover_tones() {
    assert_eq!(
        super::markers::line_number_right_padding(super::markers::LineNumberTone::Neutral),
        6.0
    );
    assert_eq!(super::markers::line_number_right_padding(super::markers::LineNumberTone::Add), 7.0);
    assert_eq!(
        super::markers::line_number_right_padding(super::markers::LineNumberTone::Delete),
        7.0
    );

    let _neutral = super::markers::line_number_cell("1".to_string());
    let _add = super::markers::line_number_cell_with_tone(
        "2".to_string(),
        super::markers::LineNumberTone::Add,
    );
    let _delete = super::markers::line_number_cell_with_tone_offset(
        "3".to_string(),
        super::markers::LineNumberTone::Delete,
        3.0,
    );
    let _empty = super::markers::empty_line_number_cell();
}

#[test]
fn line_marker_cells_cover_add_delete_none_and_emphasis() {
    for kind in [
        super::markers::LineMarkerKind::None,
        super::markers::LineMarkerKind::Add,
        super::markers::LineMarkerKind::Delete,
    ] {
        let _plain = super::markers::line_marker_cell_emphasis(kind, false);
        let _emphasized = super::markers::line_marker_cell_emphasis(kind, true);
    }
}
