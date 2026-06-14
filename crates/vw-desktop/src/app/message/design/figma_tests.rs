#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("figma_tests"));
}

use super::figma;
use crate::app::message::design::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement, DesignGroup, DesignTool};
use crate::app::views::design::state::{DesignState, FigmaProgressStage, FigmaProgressState};
use crate::app::{App, Message};
use std::sync::mpsc;

fn app_with_design_state(state: DesignState) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, state);
    app
}

fn doc_with_group(group_id: u32, element_id: &str) -> DesignDoc {
    DesignDoc {
        groups: vec![DesignGroup { id: group_id, name: format!("Group {group_id}") }],
        children: vec![DesignElement {
            kind: "frame".to_string(),
            id: element_id.to_string(),
            group_id,
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn update_ignores_non_figma_messages() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));

    assert!(figma::update(&mut app, DesignMessage::CloseHelpModal).is_none());
}

#[test]
fn file_picked_none_returns_noop_task() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));

    let task = figma::update(&mut app, DesignMessage::FigmaImportFilePicked(None));

    assert!(task.is_some());
    assert!(app.active_design_state().unwrap().figma_progress.is_none());
}

#[test]
fn progress_tick_drains_latest_progress_and_keeps_receiver() {
    let (tx, rx) = mpsc::channel();
    tx.send(FigmaProgressState::new(FigmaProgressStage::Importing, 1, 3, "one")).unwrap();
    tx.send(FigmaProgressState::new(FigmaProgressStage::Importing, 2, 3, "two")).unwrap();
    let mut state = DesignState::new(DesignDoc::default());
    state.figma_progress_rx = Some(rx);
    let mut app = app_with_design_state(state);

    let _ = figma::update(&mut app, DesignMessage::FigmaProgressTick);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.figma_progress.as_ref().map(|progress| progress.current), Some(2));
    assert!(state.figma_progress_rx.is_some());
}

#[test]
fn progress_tick_drops_disconnected_receiver() {
    let (_tx, rx) = mpsc::channel::<FigmaProgressState>();
    let mut state = DesignState::new(DesignDoc::default());
    state.figma_progress_rx = Some(rx);
    let mut app = app_with_design_state(state);

    let _ = figma::update(&mut app, DesignMessage::FigmaProgressTick);

    assert!(app.active_design_state().unwrap().figma_progress_rx.is_some());
}

#[test]
fn imported_figma_replaces_single_empty_page() {
    let mut app = app_with_design_state(DesignState::new(DesignDoc::default()));
    let imported = doc_with_group(7, "imported");

    let task = figma::update(&mut app, DesignMessage::FigmaFileImported(Ok(Some(imported))));

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.children[0].id, "imported");
    assert_eq!(state.active_tool, DesignTool::Move);
    assert_eq!(state.selected_element_id.as_deref(), Some("imported"));
    let _ = task.unwrap().map(|message| {
        assert!(matches!(message, Message::Design(DesignMessage::Snapshot)));
    });
}

#[test]
fn imported_figma_merges_groups_without_colliding_ids() {
    let base = doc_with_group(0, "base");
    let mut state = DesignState::new(base);
    state.figma_progress = Some(FigmaProgressState::new(FigmaProgressStage::Importing, 1, 1, "x"));
    let mut app = app_with_design_state(state);
    let imported = doc_with_group(0, "imported");

    let _ = figma::update(&mut app, DesignMessage::FigmaFileImported(Ok(Some(imported))));

    let state = app.active_design_state().unwrap();
    assert!(state.figma_progress.is_none());
    assert_eq!(state.doc.children.len(), 2);
    assert_eq!(state.doc.children[1].id, "imported");
    assert_ne!(state.doc.children[0].group_id, state.doc.children[1].group_id);
    assert_eq!(state.active_tool, DesignTool::Move);
}

#[test]
fn imported_figma_error_clears_progress_without_modifying_doc() {
    let mut state = DesignState::new(doc_with_group(0, "base"));
    state.figma_progress = Some(FigmaProgressState::new(FigmaProgressStage::Importing, 1, 1, "x"));
    let mut app = app_with_design_state(state);

    let _ = figma::update(&mut app, DesignMessage::FigmaFileImported(Err("bad fig".to_string())));

    let state = app.active_design_state().unwrap();
    assert!(state.figma_progress.is_none());
    assert_eq!(state.doc.children.len(), 1);
    assert_eq!(state.doc.children[0].id, "base");
}
