use super::*;
use crate::app::message::ViewMessage;
use crate::app::state::AppTab;
use crate::app::{App, Message, Screen};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

fn tab(id: &str, title: &str, screen: Screen) -> AppTab {
    AppTab { id: id.to_string(), title: title.to_string(), screen, project_path: None }
}

#[test]
fn view_builds_default_project_and_apps_states() {
    let app = test_app();
    keep_element(view(&app));

    let mut app = test_app();
    app.screen = Screen::Project;
    app.open_tabs = vec![
        tab("home", "Home", Screen::Home),
        tab("find:internal", "Find", Screen::Home),
        tab("project", "Project", Screen::Project),
        tab("apps", "Apps", Screen::Apps),
    ];
    app.active_tab_id = Some("project".to_string());
    app.hovered_tab_id = Some("project".to_string());
    keep_element(view(&app));
}

#[test]
fn tab_button_builds_home_regular_hovered_and_truncated_labels() {
    keep_element(tab_btn(
        "home".to_string(),
        "Home".to_string(),
        true,
        false,
        Message::View(ViewMessage::TabSelected("home".to_string())),
    ));
    keep_element(tab_btn(
        "project".to_string(),
        "abcdefghijklmnopqrstuvwxyz".to_string(),
        false,
        false,
        Message::View(ViewMessage::TabSelected("project".to_string())),
    ));
    keep_element(tab_btn(
        "project".to_string(),
        "Project".to_string(),
        false,
        true,
        Message::View(ViewMessage::TabSelected("project".to_string())),
    ));
}
