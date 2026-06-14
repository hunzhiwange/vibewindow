use super::node_ops_structure::{
    add_child, add_child_at, add_sibling, add_sibling_at, close_context_menu, open_context_menu,
    toggle_collapse_at,
};
use crate::app::App;
use crate::app::components::mind_map;
use crate::apps::mindmap::state::{MindMapDiagramType, MindMapTab};
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
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn add_child_adds_under_selected_node_and_respects_collapsed_parent() {
    let mut app = test_app();
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.show_url_editor = true;
        tab.url_editor_value = "https://a.test".to_string();
        tab.show_context_menu = true;
    }

    let _ = add_child(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.selected_path.as_deref(), Some(&[0, 1][..]));
    assert_eq!(tab.doc.children[0].children[1].text, "新节点");
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://a.test"));
    assert!(!tab.show_context_menu);
    assert_eq!(tab.undo_stack.len(), 1);

    let before_len = tab.doc.children[0].children.len();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.collapsed_paths.insert(vec![0]);
    tab.selected_path = Some(vec![0]);
    let _ = add_child(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().doc.children[0].children.len(), before_len);
}

#[test]
fn add_sibling_shifts_sibling_metadata_and_ignores_root_or_missing_selection() {
    let mut app = test_app();
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.selected_path = Some(vec![0]);
        tab.node_positions.insert(vec![1], Point::new(1.0, 2.0));
        tab.node_fills.insert(vec![1], 11);
        tab.node_text_colors.insert(vec![1], 12);
        tab.node_border_colors.insert(vec![1], 13);
        tab.node_priorities.insert(vec![1], 4);
        tab.node_urls.insert(vec![1], "https://b.test".to_string());
        tab.edge_styles.insert(vec![1], crate::apps::mindmap::state::EdgeStyle::Dashed);
        tab.edge_colors.insert(vec![1], 14);
        tab.collapsed_paths.insert(vec![1]);
        tab.diagram_type = MindMapDiagramType::MindMap;
    }

    let _ = add_sibling(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.children[1].text, "新节点");
    assert_eq!(tab.doc.children[2].text, "B");
    assert_eq!(tab.selected_path.as_deref(), Some(&[1][..]));
    assert!(tab.node_positions.contains_key(&vec![2]));
    assert_eq!(tab.node_fills.get(&vec![2]), Some(&11));
    assert_eq!(tab.node_text_colors.get(&vec![2]), Some(&12));
    assert_eq!(tab.node_border_colors.get(&vec![2]), Some(&13));
    assert_eq!(tab.node_priorities.get(&vec![2]), Some(&4));
    assert_eq!(tab.node_urls.get(&vec![2]).map(String::as_str), Some("https://b.test"));
    assert!(tab.collapsed_paths.contains(&vec![2]));

    let len = tab.doc.children.len();
    let tab = app.active_mindmap_tab_mut().unwrap();
    tab.selected_path = Some(Vec::new());
    let _ = add_sibling(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().doc.children.len(), len);

    app.active_mindmap_tab_mut().unwrap().selected_path = None;
    let _ = add_sibling(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().doc.children.len(), len);
}

#[test]
fn add_child_at_and_sibling_at_set_target_paths() {
    let mut app = test_app();

    let _ = add_child_at(&mut app, vec![1]);
    assert_eq!(app.active_mindmap_tab().unwrap().selected_path.as_deref(), Some(&[1, 0][..]));

    let _ = add_sibling_at(&mut app, vec![0, 0]);
    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.children[0].children[1].text, "新节点");
    assert_eq!(tab.selected_path.as_deref(), Some(&[0, 1][..]));
}

#[test]
fn toggle_collapse_ignores_leaf_and_moves_descendant_selection_to_parent() {
    let mut app = test_app();

    let _ = toggle_collapse_at(&mut app, vec![1]);
    assert!(app.active_mindmap_tab().unwrap().collapsed_paths.is_empty());

    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![0, 0]);
    let _ = toggle_collapse_at(&mut app, vec![0]);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(tab.collapsed_paths.contains(&vec![0]));
    assert_eq!(tab.selected_path.as_deref(), Some(&[0][..]));

    let _ = toggle_collapse_at(&mut app, vec![0]);
    assert!(!app.active_mindmap_tab().unwrap().collapsed_paths.contains(&vec![0]));
}

#[test]
fn context_menu_open_and_close_reset_editors_and_anchor() {
    let mut app = test_app();
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.show_url_editor = true;
        tab.url_editor_value = "https://a.test".to_string();
        tab.show_text_editor = true;
        tab.node_text_editor = text_editor::Content::with_text("Edited A");
        tab.show_markdown_import = true;
        tab.show_zoom_menu = true;
        tab.show_priority_picker = true;
    }

    let _ = open_context_menu(&mut app, vec![1], Point::new(4.0, 5.0));

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.selected_path.as_deref(), Some(&[1][..]));
    assert!(tab.show_context_menu);
    assert_eq!(tab.context_menu_anchor, Some(Point::new(4.0, 5.0)));
    assert_eq!(tab.doc.children[0].text, "Edited A");
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://a.test"));
    assert!(!tab.show_markdown_import);
    assert!(!tab.show_url_editor);
    assert!(!tab.show_text_editor);

    let _ = close_context_menu(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert!(!tab.show_context_menu);
    assert!(tab.context_menu_anchor.is_none());
}

#[test]
fn structure_ops_do_nothing_without_active_tab() {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;

    let _ = add_child(&mut app);
    let _ = add_sibling(&mut app);
    let _ = add_child_at(&mut app, vec![0]);
    let _ = add_sibling_at(&mut app, vec![0]);
    let _ = toggle_collapse_at(&mut app, vec![0]);
    let _ = open_context_menu(&mut app, vec![0], Point::new(0.0, 0.0));
    let _ = close_context_menu(&mut app);
    assert!(app.mindmap_tabs.is_empty());
}
