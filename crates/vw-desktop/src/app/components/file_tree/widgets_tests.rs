use super::widgets::RightClickArea;
use crate::app::Message;
use iced::widget::text;

#[test]
fn right_click_area_new_wraps_content() {
    let area: RightClickArea<'_, Message> =
        RightClickArea::new(text("file").into(), Box::new(|_| Message::PreviewLspTick), None, None);

    let _ = area;
}

#[test]
fn right_click_area_accepts_drag_messages() {
    let area: RightClickArea<'_, Message> = RightClickArea::new(
        text("file").into(),
        Box::new(|_| Message::PreviewLspTick),
        Some(Message::PreviewLspTick),
        Some(Message::PreviewLspTick),
    );

    let _ = area;
}
