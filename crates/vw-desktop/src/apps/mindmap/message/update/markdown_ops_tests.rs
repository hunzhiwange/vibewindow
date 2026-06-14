use super::markdown_ops::{
    apply_markdown_import, markdown_import_editor_action, toggle_markdown_import,
};
use crate::app::App;
use crate::app::components::mind_map;
use crate::apps::mindmap::state::{EdgeStyle, MindMapTab};
use iced::Point;
use iced::widget::text_editor;

fn test_app() -> App {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    let mut tab = MindMapTab::new(
        "tab-1".to_string(),
        "Map".to_string(),
        None,
        mind_map::parse("# Root\n\n- A\n  - A1\n- B\n"),
    );
    tab.selected_path = Some(vec![0]);
    tab.node_positions.insert(vec![0], Point::new(1.0, 2.0));
    tab.node_fills.insert(vec![0], 0x111111FF);
    tab.node_text_colors.insert(vec![0], 0x222222FF);
    tab.node_border_colors.insert(vec![0], 0x333333FF);
    tab.node_border_styles.insert(vec![0], EdgeStyle::Dashed);
    tab.node_priorities.insert(vec![0], 4);
    tab.node_urls.insert(vec![0], "https://a.test".to_string());
    tab.edge_styles.insert(vec![0], EdgeStyle::Dotted);
    tab.edge_colors.insert(vec![0], 0x444444FF);
    tab.collapsed_paths.insert(vec![0]);
    tab.show_diagram_type_picker = true;
    tab.show_zoom_menu = true;
    tab.show_priority_picker = true;
    tab.show_url_editor = true;
    tab.url_editor_value = "https://pending.test".to_string();
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn toggle_markdown_import_opens_with_current_markdown_and_closes_other_ui() {
    let mut app = test_app();

    let _ = toggle_markdown_import(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert!(tab.show_markdown_import);
    assert!(!tab.show_diagram_type_picker);
    assert!(!tab.show_zoom_menu);
    assert!(!tab.show_priority_picker);
    assert!(!tab.show_url_editor);
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://pending.test"));
    assert!(tab.markdown_import_editor.text().contains("# Root"));
    assert!(tab.markdown_import_editor.text().contains("- A"));

    let _ = toggle_markdown_import(&mut app);
    assert!(!app.active_mindmap_tab().unwrap().show_markdown_import);
}

#[test]
fn markdown_edit_action_updates_doc_and_remaps_matching_metadata_without_undo() {
    let mut app = test_app();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_url_editor = false;
    tab.markdown_import_editor =
        text_editor::Content::with_text("# Root\n\n- B\n- A\n  - A1\n- C\n");

    let _ = markdown_import_editor_action(
        &mut app,
        text_editor::Action::Edit(text_editor::Edit::Insert('\n')),
    );

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.children[0].text, "B");
    assert_eq!(tab.doc.children[1].text, "A");
    assert_eq!(tab.doc.children[2].text, "C");
    assert!(tab.node_positions.is_empty());
    assert_eq!(tab.selected_path, None);
    assert_eq!(tab.node_fills.get(&vec![1]), Some(&0x111111FF));
    assert_eq!(tab.node_text_colors.get(&vec![1]), Some(&0x222222FF));
    assert_eq!(tab.node_border_colors.get(&vec![1]), Some(&0x333333FF));
    assert_eq!(tab.node_border_styles.get(&vec![1]), Some(&EdgeStyle::Dashed));
    assert_eq!(tab.node_priorities.get(&vec![1]), Some(&4));
    assert_eq!(tab.node_urls.get(&vec![1]).map(String::as_str), Some("https://a.test"));
    assert_eq!(tab.edge_styles.get(&vec![1]), Some(&EdgeStyle::Dotted));
    assert_eq!(tab.edge_colors.get(&vec![1]), Some(&0x444444FF));
    assert!(tab.collapsed_paths.contains(&vec![1]));
    assert!(tab.undo_stack.is_empty());
}

#[test]
fn markdown_non_edit_action_only_updates_editor_content() {
    let mut app = test_app();
    let before = app.active_mindmap_tab().unwrap().doc.clone();

    let _ = markdown_import_editor_action(
        &mut app,
        text_editor::Action::Edit(text_editor::Edit::Insert('x')),
    );
    let after_edit = app.active_mindmap_tab().unwrap().doc.clone();

    let _ = markdown_import_editor_action(
        &mut app,
        text_editor::Action::Move(text_editor::Motion::Left),
    );

    let tab = app.active_mindmap_tab().unwrap();
    assert_ne!(after_edit, before);
    assert_eq!(tab.doc, after_edit);
}

#[test]
fn apply_markdown_import_commits_doc_pushes_undo_and_resets_ui() {
    let mut app = test_app();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.show_markdown_import = true;
    tab.markdown_import_editor = text_editor::Content::with_text("  ");
    tab.redo_stack.push(tab.doc.clone());

    let _ = apply_markdown_import(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.text, "中心主题");
    assert!(tab.doc.children.is_empty());
    assert_eq!(tab.undo_stack.len(), 1);
    assert!(tab.redo_stack.is_empty());
    assert!(!tab.show_markdown_import);
    assert!(!tab.show_diagram_type_picker);
    assert!(!tab.show_priority_picker);
    assert!(!tab.show_url_editor);
    assert!(!tab.show_text_editor);
    assert!(tab.url_editor_value.is_empty());
    assert!(tab.node_fills.is_empty());
    assert!(tab.collapsed_paths.is_empty());
}

#[test]
fn markdown_ops_do_nothing_without_active_tab() {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;

    let _ = toggle_markdown_import(&mut app);
    let _ = markdown_import_editor_action(
        &mut app,
        text_editor::Action::Edit(text_editor::Edit::Insert('x')),
    );
    let _ = apply_markdown_import(&mut app);
    assert!(app.mindmap_tabs.is_empty());
}
