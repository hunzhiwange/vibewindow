use super::{centered_overlay_rect, home_layout, main_layout};
use ratatui::layout::Rect;

#[test]
fn main_layout_splits_expected_regions() {
    let layout = main_layout(Rect::new(0, 0, 100, 30));

    assert_eq!(layout.header_area().height, 4);
    assert_eq!(layout.subheader_area().height, 0);
    assert_eq!(layout.footer_area().height, 1);
    assert!(layout.body_area().height >= 5);
}

#[test]
fn input_area_comes_from_left_lower_body() {
    let layout = main_layout(Rect::new(0, 0, 80, 24));
    let input = layout.input_area();

    assert!(input.width > 0);
    assert!(input.height > 0);
    assert!(input.x < layout.body_area().x + layout.body_area().width);
    assert!(input.y >= layout.body_area().y);
}

#[test]
fn home_layout_centers_logo_and_input_stack() {
    let (chunks, center) = home_layout(Rect::new(0, 0, 80, 24), 5, 3);

    assert_eq!(chunks.len(), 4);
    assert_eq!(center.len(), 6);
    assert_eq!(center[0].height, 5);
    assert!(center[2].height > 0);
    assert!(center[2].height <= 3);
    assert_eq!(chunks[3].height, 1);
}

#[test]
fn centered_overlay_respects_max_height_and_centering() {
    let overlay = centered_overlay_rect(Rect::new(10, 20, 90, 40), 2, 5, 10);

    assert_eq!(overlay.height, 10);
    assert!(overlay.x >= 10);
    assert!(overlay.y >= 20);
    assert!(overlay.width < 90);
}
