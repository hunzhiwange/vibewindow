use iced::Length;

use super::project_sessions_panel_container;
use crate::app::App;

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("sessions_panel_tests"));
}

#[test]
fn project_sessions_panel_container_uses_scaled_available_width() {
    let (app, _task) = App::new();
    let element = project_sessions_panel_container(&app, 320.0, 80.0, 0.5, 12.0, None);
    let size = element.as_widget().size();

    assert_eq!(size.width, Length::Fixed(120.0));
    assert_eq!(size.height, Length::Fill);
}

#[test]
fn project_sessions_panel_container_clamps_negative_width_to_zero() {
    let (app, _task) = App::new();
    let element = project_sessions_panel_container(
        &app,
        80.0,
        120.0,
        1.25,
        8.0,
        Some("/tmp/project".to_string()),
    );
    let size = element.as_widget().size();

    assert_eq!(size.width, Length::Fixed(0.0));
    assert_eq!(size.height, Length::Fill);
}
