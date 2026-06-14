use super::menus::{edit_menu, file_menu, help_menu, view_menu};
use crate::app::message::view::MenuType;
use crate::app::{App, Screen};
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::MindMapTab;

fn test_app(screen: Screen) -> App {
    let (mut app, _task) = App::new();
    app.screen = screen;
    app.active_menu = None;
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;
    app
}

fn mindmap_tab() -> MindMapTab {
    MindMapTab::new("tab-1".to_string(), "Mind map".to_string(), None, model::default_doc())
}

#[test]
fn file_menu_renders_screen_specific_items() {
    for screen in [Screen::Design, Screen::Project, Screen::Home] {
        let mut app = test_app(screen);
        app.active_menu = Some(MenuType::File);
        let _element = file_menu(&app);
    }
}

#[test]
fn file_menu_renders_when_file_menu_is_closed() {
    let mut app = test_app(Screen::Project);
    app.active_menu = None;

    let _element = file_menu(&app);
}

#[test]
fn file_menu_renders_mindmap_save_states() {
    let app = test_app(Screen::MindMapTool);
    let _element = file_menu(&app);

    let mut app = test_app(Screen::MindMapTool);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app.mindmap_tabs.push(mindmap_tab());
    let _element = file_menu(&app);
}

#[test]
fn edit_menu_renders_design_project_and_empty_fallback() {
    for screen in [Screen::Design, Screen::Project, Screen::Home] {
        let mut app = test_app(screen);
        app.active_menu = Some(MenuType::Edit);
        let _element = edit_menu(&app);
    }
}

#[test]
fn edit_menu_renders_mindmap_disabled_and_enabled_actions() {
    let app = test_app(Screen::MindMapTool);
    let _element = edit_menu(&app);

    let mut app = test_app(Screen::MindMapTool);
    let mut tab = mindmap_tab();
    tab.undo_stack.push(model::default_doc());
    tab.redo_stack.push(model::default_doc());
    tab.selected_path = Some(vec![0]);
    tab.clipboard_node = Some(model::default_doc());
    app.mindmap_active_tab_id = Some(tab.id.clone());
    app.mindmap_tabs.push(tab);
    let _element = edit_menu(&app);

    let mut app = test_app(Screen::MindMapTool);
    let mut tab = mindmap_tab();
    tab.selected_path = Some(Vec::new());
    app.mindmap_active_tab_id = Some(tab.id.clone());
    app.mindmap_tabs.push(tab);
    let _element = edit_menu(&app);
}

#[test]
fn edit_menu_renders_project_when_menu_is_closed() {
    let mut app = test_app(Screen::Project);
    app.active_menu = None;

    let _element = edit_menu(&app);
}

#[test]
fn view_menu_renders_panel_and_mode_toggle_states() {
    let mut app = test_app(Screen::Project);
    app.active_menu = Some(MenuType::View);
    app.show_file_manager = false;
    app.file_manager_show_changes = false;
    let _element = view_menu(&app);
    drop(_element);

    app.show_file_manager = true;
    app.file_manager_show_changes = true;
    let _element = view_menu(&app);
}

#[test]
fn help_menu_renders_available_actions() {
    let mut app = test_app(Screen::Home);
    app.active_menu = Some(MenuType::Help);
    let _element = help_menu(&app);
}

#[test]
fn help_menu_renders_closed_state() {
    let app = test_app(Screen::Home);

    let _element = help_menu(&app);
}
