#[test]
fn task_1181_test_module_is_wired() {}

use super::*;
use crate::app::message::DesignMessage;
use iced::widget::canvas::Program;

fn stops() -> Vec<GradientStop> {
    vec![
        GradientStop { color: "#000000".to_string(), position: 0.0 },
        GradientStop { color: "#ffffff".to_string(), position: 1.0 },
    ]
}

fn bar() -> GradientStopsBar {
    GradientStopsBar {
        stops: stops(),
        on_change: Box::new(|stops| {
            Message::Design(DesignMessage::PropertyUpdate(
                "shape".to_string(),
                "fill".to_string(),
                serde_json::json!(stops),
            ))
        }),
    }
}

fn bounds() -> Rectangle {
    Rectangle { x: 10.0, y: 20.0, width: 100.0, height: 24.0 }
}

#[test]
fn state_defaults_to_not_dragging() {
    let state = GradientStopsBarState::default();

    assert_eq!(state.dragging, None);
}

#[test]
fn update_ignores_events_without_cursor_inside_bounds() {
    let bar = bar();
    let mut state = GradientStopsBarState::default();
    let event = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));

    let action = <GradientStopsBar as Program<Message>>::update(
        &bar,
        &mut state,
        &event,
        bounds(),
        mouse::Cursor::Unavailable,
    );

    assert!(action.is_none());
    assert_eq!(state.dragging, None);
}

#[test]
fn clicking_existing_stop_starts_dragging_and_release_clears_it() {
    let bar = bar();
    let mut state = GradientStopsBarState::default();
    let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let release = Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));

    let action = <GradientStopsBar as Program<Message>>::update(
        &bar,
        &mut state,
        &press,
        bounds(),
        mouse::Cursor::Available(Point::new(10.0, 32.0)),
    );
    assert!(action.is_some());
    assert_eq!(state.dragging, Some(0));

    let action = <GradientStopsBar as Program<Message>>::update(
        &bar,
        &mut state,
        &release,
        bounds(),
        mouse::Cursor::Available(Point::new(10.0, 32.0)),
    );
    assert!(action.is_some());
    assert_eq!(state.dragging, None);
}

#[test]
fn clicking_empty_space_adds_stop_and_dragging_move_publishes_update() {
    let bar = bar();
    let mut state = GradientStopsBarState::default();
    let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let move_event = Event::Mouse(mouse::Event::CursorMoved { position: Point::new(85.0, 32.0) });

    let action = <GradientStopsBar as Program<Message>>::update(
        &bar,
        &mut state,
        &press,
        bounds(),
        mouse::Cursor::Available(Point::new(60.0, 32.0)),
    );
    assert!(action.is_some());
    assert_eq!(state.dragging, Some(2));

    let action = <GradientStopsBar as Program<Message>>::update(
        &bar,
        &mut state,
        &move_event,
        bounds(),
        mouse::Cursor::Available(Point::new(85.0, 32.0)),
    );
    assert!(action.is_some());
}
