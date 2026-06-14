use super::node_ops_history::{redo_node, undo_node};
use crate::app::App;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapTab;

fn app_with_doc(text: &str) -> App {
    let (mut app, _) = App::new();
    app.mindmap_tabs.clear();
    let tab = MindMapTab::new(
        "tab-1".to_string(),
        "Map".to_string(),
        None,
        MindNode { text: text.to_string(), children: Vec::new() },
    );
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn undo_and_redo_move_docs_between_stacks_and_close_menu() {
    let mut app = app_with_doc("current");
    {
        let tab = app.active_mindmap_tab_mut().unwrap();
        tab.undo_stack.push(MindNode { text: "previous".to_string(), children: Vec::new() });
        tab.show_context_menu = true;
    }

    let _ = undo_node(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.text, "previous");
    assert_eq!(tab.redo_stack.last().unwrap().text, "current");
    assert!(!tab.show_context_menu);

    let _ = redo_node(&mut app);
    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.text, "current");
    assert_eq!(tab.undo_stack.last().unwrap().text, "previous");
}

#[test]
fn undo_and_redo_with_empty_stack_only_close_menu() {
    let mut app = app_with_doc("current");
    app.active_mindmap_tab_mut().unwrap().show_context_menu = true;

    let _ = undo_node(&mut app);
    let _ = redo_node(&mut app);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.doc.text, "current");
    assert!(!tab.show_context_menu);
}
