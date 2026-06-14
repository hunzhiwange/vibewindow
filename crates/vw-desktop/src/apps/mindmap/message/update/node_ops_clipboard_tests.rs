use super::node_ops_clipboard::{copy_node, cut_node, delete_node, duplicate_node, paste_node};
use crate::app::App;
use crate::app::components::mind_map;
use crate::apps::mindmap::state::{EdgeStyle, MindMapTab};
use iced::Point;

fn test_app() -> App {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    let mut tab = MindMapTab::new(
        "tab-1".to_string(),
        "Map".to_string(),
        None,
        mind_map::parse("# Root\n\n- A\n  - A1\n- B\n- C\n"),
    );
    tab.selected_path = Some(vec![0]);
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

fn seed_metadata(tab: &mut MindMapTab) {
    tab.node_positions.insert(vec![0], Point::new(1.0, 2.0));
    tab.node_positions.insert(vec![0, 0], Point::new(3.0, 4.0));
    tab.node_positions.insert(vec![1], Point::new(5.0, 6.0));
    tab.node_fills.insert(vec![0], 10);
    tab.node_text_colors.insert(vec![0], 11);
    tab.node_border_colors.insert(vec![0], 12);
    tab.node_priorities.insert(vec![0], 3);
    tab.node_urls.insert(vec![0], "https://a.test".to_string());
    tab.edge_styles.insert(vec![0], EdgeStyle::Dashed);
    tab.edge_colors.insert(vec![0], 13);
    tab.collapsed_paths.insert(vec![0]);
    tab.node_fills.insert(vec![1], 20);
    tab.node_urls.insert(vec![1], "https://b.test".to_string());
}

#[test]
fn copy_and_paste_selected_node_append_clone_under_parent() {
    let mut app = test_app();

    let _ = copy_node(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().clipboard_node.as_ref().unwrap().text, "A");

    app.active_mindmap_tab_mut().unwrap().selected_path = Some(vec![1]);
    let _ = paste_node(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.children[1].children[0].text, "A");
    assert_eq!(tab.selected_path.as_deref(), Some(&[1, 0][..]));
    assert_eq!(tab.undo_stack.len(), 1);
}

#[test]
fn cut_node_removes_subtree_metadata_shifts_siblings_and_fills_clipboard() {
    let mut app = test_app();
    seed_metadata(app.active_mindmap_tab_mut().unwrap());

    let _ = cut_node(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.clipboard_node.as_ref().unwrap().text, "A");
    assert_eq!(tab.selected_path.as_deref(), Some(&[][..]));
    assert_eq!(tab.doc.children[0].text, "B");
    assert_eq!(tab.doc.children[1].text, "C");
    assert!(!tab.node_positions.contains_key(&vec![0, 0]));
    assert_eq!(tab.node_fills.get(&vec![0]), Some(&20));
    assert_eq!(tab.node_urls.get(&vec![0]).map(String::as_str), Some("https://b.test"));
    assert_eq!(tab.undo_stack.len(), 1);
}

#[test]
fn delete_node_removes_without_touching_clipboard_and_protects_root() {
    let mut app = test_app();
    seed_metadata(app.active_mindmap_tab_mut().unwrap());
    app.active_mindmap_tab_mut().unwrap().clipboard_node =
        Some(mind_map::parse("# Existing Clipboard\n"));

    let _ = delete_node(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.clipboard_node.as_ref().unwrap().text, "Existing Clipboard");
    assert_eq!(tab.selected_path.as_deref(), Some(&[][..]));
    assert_eq!(tab.doc.children[0].text, "B");
    assert_eq!(tab.node_fills.get(&vec![0]), Some(&20));

    let len = tab.doc.children.len();
    app.active_mindmap_tab_mut().unwrap().selected_path = Some(Vec::new());
    let _ = delete_node(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().doc.children.len(), len);
}

#[test]
fn duplicate_node_copies_subtree_metadata_and_offsets_positions() {
    let mut app = test_app();
    seed_metadata(app.active_mindmap_tab_mut().unwrap());

    let _ = duplicate_node(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.children[1].text, "A");
    assert_eq!(tab.doc.children[1].children[0].text, "A1");
    assert_eq!(tab.doc.children[2].text, "B");
    assert_eq!(tab.selected_path.as_deref(), Some(&[1][..]));
    assert_eq!(tab.node_fills.get(&vec![1]), Some(&10));
    assert_eq!(tab.node_text_colors.get(&vec![1]), Some(&11));
    assert_eq!(tab.node_border_colors.get(&vec![1]), Some(&12));
    assert_eq!(tab.node_priorities.get(&vec![1]), Some(&3));
    assert_eq!(tab.node_urls.get(&vec![1]).map(String::as_str), Some("https://a.test"));
    assert_eq!(tab.edge_styles.get(&vec![1]), Some(&EdgeStyle::Dashed));
    assert_eq!(tab.edge_colors.get(&vec![1]), Some(&13));
    assert!(tab.collapsed_paths.contains(&vec![1]));
    assert!(tab.node_positions.contains_key(&vec![1]));
}

#[test]
fn clipboard_ops_ignore_missing_selection_clipboard_or_root() {
    let mut app = test_app();
    let len = app.active_mindmap_tab().unwrap().doc.children.len();

    app.active_mindmap_tab_mut().unwrap().selected_path = None;
    let _ = copy_node(&mut app);
    let _ = cut_node(&mut app);
    let _ = delete_node(&mut app);
    let _ = paste_node(&mut app);
    let _ = duplicate_node(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().doc.children.len(), len);

    app.active_mindmap_tab_mut().unwrap().selected_path = Some(Vec::new());
    let _ = cut_node(&mut app);
    let _ = delete_node(&mut app);
    let _ = duplicate_node(&mut app);
    assert_eq!(app.active_mindmap_tab().unwrap().doc.children.len(), len);
}

#[test]
fn clipboard_ops_do_nothing_without_active_tab() {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;

    let _ = copy_node(&mut app);
    let _ = cut_node(&mut app);
    let _ = delete_node(&mut app);
    let _ = paste_node(&mut app);
    let _ = duplicate_node(&mut app);
    assert!(app.mindmap_tabs.is_empty());
}
