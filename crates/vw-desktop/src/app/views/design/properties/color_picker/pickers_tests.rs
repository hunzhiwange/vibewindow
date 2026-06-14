#[test]
fn task_1174_test_module_is_wired() {}

use iced::mouse;
use iced::widget::canvas::{Event, Program};
use iced::{Color, Point, Rectangle};

use crate::app::Message;

fn bounds() -> Rectangle {
    Rectangle { x: 10.0, y: 20.0, width: 100.0, height: 50.0 }
}

fn cursor(x: f32, y: f32) -> mouse::Cursor {
    mouse::Cursor::Available(Point::new(x, y))
}

#[test]
fn saturation_value_picker_updates_drag_state_and_ignores_outside_cursor() {
    let picker = super::SaturationValuePicker {
        hsv: super::Hsv { h: 20.0, s: 0.1, v: 0.2 },
        on_change: Box::new(|_| Message::None),
    };
    let mut state = super::SaturationValueState::default();

    let pressed = picker.update(
        &mut state,
        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        bounds(),
        cursor(60.0, 45.0),
    );
    assert!(pressed.is_some());
    assert!(state.is_dragging);

    let moved = picker.update(
        &mut state,
        &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(110.0, 70.0) }),
        bounds(),
        cursor(110.0, 70.0),
    );
    assert!(moved.is_some());

    let released = picker.update(
        &mut state,
        &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        bounds(),
        cursor(60.0, 45.0),
    );
    assert!(released.is_some());
    assert!(!state.is_dragging);

    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds(),
                mouse::Cursor::Unavailable,
            )
            .is_none()
    );
}

#[test]
fn saturation_value_picker_ignores_move_and_release_when_not_dragging() {
    let picker = super::SaturationValuePicker {
        hsv: super::Hsv { h: 20.0, s: 0.1, v: 0.2 },
        on_change: Box::new(|_| Message::None),
    };
    let mut state = super::SaturationValueState::default();

    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(60.0, 45.0) }),
                bounds(),
                cursor(60.0, 45.0),
            )
            .is_none()
    );
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds(),
                cursor(60.0, 45.0),
            )
            .is_none()
    );
}

#[test]
fn hue_picker_updates_while_dragging_and_resets_on_release() {
    let picker = super::HuePicker {
        hsv: super::Hsv { h: 20.0, s: 0.1, v: 0.2 },
        on_change: Box::new(|_| Message::None),
    };
    let mut state = super::HueState::default();

    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds(),
                cursor(10.0, 20.0),
            )
            .is_some()
    );
    assert!(state.is_dragging);
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(110.0, 20.0) }),
                bounds(),
                cursor(110.0, 20.0),
            )
            .is_some()
    );
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds(),
                cursor(110.0, 20.0),
            )
            .is_some()
    );
    assert!(!state.is_dragging);
}

#[test]
fn alpha_picker_updates_and_ignores_non_dragging_events() {
    let picker = super::AlphaPicker {
        rgb: Color::from_rgb(1.0, 0.0, 0.0),
        alpha: 0.4,
        on_change: Box::new(|_| Message::None),
    };
    let mut state = super::AlphaState::default();

    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(20.0, 20.0) }),
                bounds(),
                cursor(20.0, 20.0),
            )
            .is_none()
    );
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds(),
                cursor(20.0, 20.0),
            )
            .is_none()
    );
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds(),
                cursor(160.0, 20.0),
            )
            .is_none(),
        "cursor outside bounds should be ignored"
    );
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                bounds(),
                cursor(60.0, 20.0),
            )
            .is_some()
    );
    assert!(state.is_dragging);
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::CursorMoved { position: Point::new(80.0, 20.0) }),
                bounds(),
                cursor(80.0, 20.0),
            )
            .is_some()
    );
    assert!(
        picker
            .update(
                &mut state,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                bounds(),
                cursor(80.0, 20.0),
            )
            .is_some()
    );
    assert!(!state.is_dragging);
}
