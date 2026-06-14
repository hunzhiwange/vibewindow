use super::*;

fn test_app() -> App {
    App::new().0
}

#[test]
fn view_window_routes_task_pet_window_to_pet_view() {
    let mut app = test_app();
    let pet_window = iced::window::Id::unique();
    app.task_pet_window_id = Some(pet_window);

    let _pet: iced::Element<'_, Message> = app.view_window(pet_window);
    let _main: iced::Element<'_, Message> = app.view_window(iced::window::Id::unique());
}

#[test]
fn view_builds_for_primary_screens_without_modal_state() {
    let mut app = test_app();

    for screen in [
        Screen::Home,
        Screen::Project,
        Screen::Apps,
        Screen::Usage,
        Screen::JsonTool,
        Screen::JsonYamlTool,
        Screen::Knowledge,
        Screen::SqlTool,
        Screen::RedisTool,
        Screen::HtmlTool,
        Screen::JsonDiffTool,
        Screen::MarkdownTool,
        Screen::PasswordTool,
        Screen::BaseTool,
        Screen::TimestampTool,
        Screen::QrTool,
        Screen::ColorTool,
        Screen::CleanerTool,
        Screen::LargeFileTool,
        Screen::TaskBoard,
    ] {
        app.screen = screen;
        let _element: iced::Element<'_, Message> = app.view();
    }
}

#[test]
fn view_builds_with_settings_notifications_error_and_toast_overlays() {
    let mut app = test_app();
    app.show_system_settings = true;
    app.notifications_expanded = true;
    app.error_message = Some("failure".to_string());
    let _ = app.show_success_toast("saved");

    let _element: iced::Element<'_, Message> = app.view();
}

#[test]
fn view_builds_with_about_cli_search_and_rename_overlays() {
    let mut app = test_app();
    app.show_about_modal = true;
    app.show_cli_install_modal = true;
    app.cli_install_modal_show_update_action = true;
    app.cli_install_modal_title = "CLI".to_string();
    app.cli_install_modal_message = "Install".to_string();
    app.cli_install_modal_current_version = "1.0.0".to_string();
    app.cli_install_modal_server_version = "1.1.0".to_string();
    app.cli_install_modal_show_install_action = true;
    app.show_search_overlay = true;
    app.file_tree_rename_path = Some("src/lib.rs".to_string());
    app.file_tree_rename_value = "main.rs".to_string();
    app.session_rename_id = Some("session-1".to_string());
    app.session_rename_value = "Session".to_string();

    let _element: iced::Element<'_, Message> = app.view();
}
