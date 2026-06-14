use crate::app::App;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;

fn app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

fn tailwind_doc() -> DesignDoc {
    DesignDoc {
        children: vec![DesignElement {
            id: "tw".to_string(),
            kind: "tailwind".to_string(),
            content: Some(
                "<div class=\"old\"><span>hello</span><button>bye</button></div>".to_string(),
            ),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn update_tailwind_node_class_changes_nested_html_and_editors() {
    let mut app = app_with_doc(tailwind_doc());
    let state = app.active_design_state_mut().unwrap();
    state.selected_element_id = Some("tw".to_string());
    state.doc.tailwind_selection = Some(("tw".to_string(), vec![0, 0]));

    let task = super::tailwind::update(
        &mut app,
        DesignMessage::UpdateTailwindNodeClass(
            "tw".to_string(),
            vec![0, 0],
            "text-lg font-bold".to_string(),
        ),
    );

    assert!(task.is_some());
    let state = app.active_design_state().unwrap();
    let html = state.doc.find_element("tw").unwrap().content.as_deref().unwrap();
    assert!(html.contains("text-lg font-bold"));
    assert_eq!(state.tailwind_html_editor.text(), html);
    assert_eq!(state.tailwind_node_class_editor.text(), "text-lg font-bold");
}

#[test]
fn commit_messages_normalize_editor_text_and_delegate_to_update() {
    let mut app = app_with_doc(tailwind_doc());
    let state = app.active_design_state_mut().unwrap();
    state.selected_element_id = Some("tw".to_string());
    state.doc.tailwind_selection = Some(("tw".to_string(), vec![0]));
    state.tailwind_node_class_editor =
        iced::widget::text_editor::Content::with_text("  flex   items-center ");
    state.tailwind_node_text_editor = iced::widget::text_editor::Content::with_text("updated text");

    let _ = super::tailwind::update(
        &mut app,
        DesignMessage::TailwindNodeClassCommit("tw".to_string(), vec![0]),
    );
    let _ = super::tailwind::update(
        &mut app,
        DesignMessage::TailwindNodeTextCommit("tw".to_string(), vec![0, 0]),
    );

    let html = app
        .active_design_state()
        .unwrap()
        .doc
        .find_element("tw")
        .unwrap()
        .content
        .as_deref()
        .unwrap()
        .to_string();
    assert!(html.contains("flex items-center"));
    assert!(html.contains("updated text"));
}

#[test]
fn update_tailwind_html_replaces_content_and_syncs_editor() {
    let mut app = app_with_doc(tailwind_doc());
    app.active_design_state_mut().unwrap().selected_element_id = Some("tw".to_string());

    let task = super::tailwind::update(
        &mut app,
        DesignMessage::UpdateTailwindHtml("tw".to_string(), "<section>fresh</section>".to_string()),
    );

    assert!(task.is_some());
    let state = app.active_design_state().unwrap();
    assert_eq!(
        state.doc.find_element("tw").unwrap().content.as_deref(),
        Some("<section>fresh</section>")
    );
    assert_eq!(state.tailwind_html_editor.text(), "<section>fresh</section>");
}

#[test]
fn delete_tailwind_node_removes_selected_node_and_clears_selection() {
    let mut app = app_with_doc(tailwind_doc());
    let state = app.active_design_state_mut().unwrap();
    state.selected_element_id = Some("tw".to_string());
    state.doc.tailwind_selection = Some(("tw".to_string(), vec![0, 1]));
    state.tailwind_node_class_editor = iced::widget::text_editor::Content::with_text("old");
    state.tailwind_node_text_editor = iced::widget::text_editor::Content::with_text("bye");
    state.tailwind_node_class_input = "p-2".to_string();
    state.tailwind_node_class_dropdown_open = true;

    let task = super::tailwind::update(
        &mut app,
        DesignMessage::DeleteTailwindNode("tw".to_string(), vec![0, 1]),
    );

    assert!(task.is_some());
    let state = app.active_design_state().unwrap();
    let html = state.doc.find_element("tw").unwrap().content.as_deref().unwrap();
    assert!(!html.contains("button"));
    assert_eq!(state.doc.tailwind_selection, None);
    assert!(state.tailwind_node_class_editor.text().is_empty());
    assert!(state.tailwind_node_text_editor.text().is_empty());
    assert!(state.tailwind_node_class_input.is_empty());
    assert!(!state.tailwind_node_class_dropdown_open);
}

#[test]
fn unhandled_tailwind_message_returns_none() {
    let mut app = app_with_doc(tailwind_doc());

    assert!(super::tailwind::update(&mut app, DesignMessage::Snapshot).is_none());
}
