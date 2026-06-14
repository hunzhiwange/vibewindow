use super::*;
use crate::app::state::ProjectEditTab;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

fn root_element() -> Element<'static, Message> {
    text("root").into()
}

#[test]
fn with_project_edit_returns_root_when_no_project_is_being_edited() {
    let app = test_app();

    let element = with_project_edit(&app, root_element());

    std::hint::black_box(element);
}

#[test]
fn with_project_edit_builds_all_tab_variants() {
    for tab in [
        ProjectEditTab::General,
        ProjectEditTab::Launch,
        ProjectEditTab::Refresh,
        ProjectEditTab::Scheduling,
    ] {
        let mut app = test_app();
        app.project_edit_path = Some("/repo/app".to_string());
        app.project_edit_tab = tab;
        app.project_edit_name = "Project".to_string();
        app.project_edit_icon_color = "#60a5fa".to_string();

        let element = with_project_edit(&app, root_element());

        std::hint::black_box(element);
    }
}

#[test]
fn with_project_edit_builds_color_picker_overlay_with_valid_and_invalid_colors() {
    for color in ["#34d399", "not-a-color"] {
        let mut app = test_app();
        app.project_edit_path = Some("/repo/app".to_string());
        app.project_edit_tab = ProjectEditTab::General;
        app.project_edit_icon_color_picker_open = true;
        app.project_edit_icon_color = color.to_string();

        let element = with_project_edit(&app, root_element());

        std::hint::black_box(element);
    }
}
