use crate::app::App;

fn render_project_view_with_settings_width(settings_panel_width: f32) {
    let (mut app, _task) = App::new();
    app.settings_panel_width = settings_panel_width;

    let element = super::view(&app);

    std::hint::black_box(element);
}

#[test]
fn view_builds_with_regular_settings_width() {
    render_project_view_with_settings_width(476.0);
}

#[test]
fn view_builds_with_too_small_settings_width() {
    render_project_view_with_settings_width(120.0);
}

#[test]
fn view_builds_with_too_large_settings_width() {
    render_project_view_with_settings_width(1_200.0);
}

#[test]
fn view_builds_with_non_finite_settings_width() {
    render_project_view_with_settings_width(f32::INFINITY);
}
