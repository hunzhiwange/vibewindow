use crate::app::App;
use crate::app::message::design::{DesignMessage, history};
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;

fn element(id: &str) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        ..serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

fn doc_with(id: &str) -> DesignDoc {
    DesignDoc { version: "1.0".to_string(), children: vec![element(id)], ..Default::default() }
}

fn app_with_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("history_tests"));
}

#[test]
fn undo_and_redo_restore_docs_and_clear_selection() {
    let mut state = DesignState::new(doc_with("one"));
    state.history.push(doc_with("two"));
    state.history_index = 1;
    state.doc = doc_with("two");
    state.selected_element_id = Some("two".to_string());
    state.selected_element_ids.insert("two".to_string());

    let mut app = app_with_state(state);

    let _ = history::update(&mut app, DesignMessage::Undo);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.history_index, 0);
    assert_eq!(state.doc.children[0].id, "one");
    assert!(state.selected_element_id.is_none());
    assert!(state.selected_element_ids.is_empty());

    let _ = history::update(&mut app, DesignMessage::Redo);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.history_index, 1);
    assert_eq!(state.doc.children[0].id, "two");
}

#[test]
fn snapshot_truncates_redo_branch_and_keeps_recent_limit() {
    let mut state = DesignState::new(doc_with("base"));
    state.history = vec![doc_with("old"), doc_with("current"), doc_with("future")];
    state.history_index = 1;
    state.doc = doc_with("new");
    let mut app = app_with_state(state);

    let _ = history::update(&mut app, DesignMessage::Snapshot);
    let state = app.active_design_state_mut().unwrap();
    assert_eq!(state.history.len(), 3);
    assert_eq!(state.history_index, 2);
    assert_eq!(state.history[2].children[0].id, "new");

    for idx in 0..60 {
        app.active_design_state_mut().unwrap().doc = doc_with(&format!("doc-{idx}"));
        let _ = history::update(&mut app, DesignMessage::Snapshot);
    }

    let state = app.active_design_state().unwrap();
    assert_eq!(state.history.len(), 50);
    assert_eq!(state.history_index, 49);
    assert_eq!(state.history.last().unwrap().children[0].id, "doc-59");
}
