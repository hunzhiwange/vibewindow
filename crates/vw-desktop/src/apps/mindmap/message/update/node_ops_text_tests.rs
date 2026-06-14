use super::node_ops_text::{
    commit_text_editor_if_needed, node_text_changed, node_text_editor_action,
    node_text_editor_enter, save_node_text, toggle_node_text_editor,
};
use crate::app::App;
use crate::app::components::mind_map;
use crate::apps::mindmap::state::MindMapTab;
use iced::widget::text_editor;

fn test_app() -> App {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    let mut tab = MindMapTab::new(
        "tab-1".to_string(),
        "Map".to_string(),
        None,
        mind_map::parse("# Root\n\n- A\n"),
    );
    tab.selected_path = Some(vec![0]);
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn toggle_text_editor_opens_with_selected_node_text_and_closes_other_ui() {
    let mut app = test_app();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_url_editor = true;
    tab.url_editor_value = " ` https://a.test ` ".to_string();
    tab.show_diagram_type_picker = true;
    tab.show_markdown_import = true;
    tab.show_zoom_menu = true;
    tab.show_priority_picker = true;
    tab.show_action_menu = true;
    tab.show_theme_panel = true;

    let _ = toggle_node_text_editor(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert!(tab.show_text_editor);
    assert_eq!(tab.node_text_editor.text(), "A");
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://a.test"));
    assert!(!tab.show_url_editor);
    assert!(!tab.show_diagram_type_picker);
    assert!(!tab.show_markdown_import);
    assert!(!tab.show_zoom_menu);
    assert!(!tab.show_priority_picker);
    assert!(!tab.show_action_menu);
    assert!(!tab.show_theme_panel);
}

#[test]
fn commit_and_save_text_editor_push_undo_only_when_text_changes() {
    let mut app = test_app();
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.show_text_editor = true;
        tab.node_text_editor = text_editor::Content::with_text("A");
        commit_text_editor_if_needed(tab);
        assert!(tab.undo_stack.is_empty());

        tab.node_text_editor = text_editor::Content::with_text("Renamed");
        commit_text_editor_if_needed(tab);
        assert_eq!(tab.doc.children[0].text, "Renamed");
        assert_eq!(tab.undo_stack.len(), 1);

        tab.selected_path = Some(vec![99]);
        tab.node_text_editor = text_editor::Content::with_text("Ignored");
        commit_text_editor_if_needed(tab);
        assert_eq!(tab.undo_stack.len(), 1);
    }

    let _ = save_node_text(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(!tab.show_text_editor);
    assert!(tab.node_text_editor.text().is_empty());
}

#[test]
fn text_change_action_enter_and_toggle_close_update_editor_state() {
    let mut app = test_app();

    let _ = node_text_changed(&mut app, "Draft".to_string());
    assert_eq!(app.active_mindmap_tab().unwrap().node_text_editor.text(), "Draft");

    let _ = node_text_editor_action(
        &mut app,
        text_editor::Action::Edit(text_editor::Edit::Insert('x')),
    );
    assert_eq!(app.active_mindmap_tab().unwrap().node_text_editor.text(), "Draft");

    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_text_editor = true;
    tab.node_text_editor = text_editor::Content::with_text("");
    let _ = node_text_editor_action(
        &mut app,
        text_editor::Action::Edit(text_editor::Edit::Insert('Q')),
    );
    assert_eq!(app.active_mindmap_tab().unwrap().node_text_editor.text(), "Q");

    let _ = node_text_editor_enter(&mut app, true);
    assert_eq!(app.active_mindmap_tab().unwrap().node_text_editor.text(), "Q\n");

    let _ = node_text_editor_enter(&mut app, false);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(!tab.show_text_editor);
    assert_eq!(tab.doc.children[0].text, "Q\n");

    let _ = toggle_node_text_editor(&mut app);
    let _ = node_text_changed(&mut app, "Closed by toggle".to_string());
    let _ = toggle_node_text_editor(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(!tab.show_text_editor);
    assert_eq!(tab.doc.children[0].text, "Closed by toggle");
}

#[test]
fn text_ops_do_nothing_without_active_tab_or_hidden_editor() {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;

    let _ = toggle_node_text_editor(&mut app);
    let _ = node_text_changed(&mut app, "x".to_string());
    let _ = node_text_editor_action(
        &mut app,
        text_editor::Action::Edit(text_editor::Edit::Insert('x')),
    );
    let _ = node_text_editor_enter(&mut app, false);
    let _ = save_node_text(&mut app);
    assert!(app.mindmap_tabs.is_empty());
}
