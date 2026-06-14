use crate::app::App;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;
use iced::widget::text_editor::{Action, Edit};

fn app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

fn text_action(value: &str) -> Action {
    Action::Edit(Edit::Paste(value.to_string().into()))
}

#[test]
fn context_editor_action_writes_selected_element_context() {
    let doc = DesignDoc {
        children: vec![DesignElement { id: "card".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    app.active_design_state_mut().unwrap().selected_element_id = Some("card".to_string());

    let _ = super::editors::context_editor_action(&mut app, text_action("usage note"));

    let element = app.active_design_state().unwrap().doc.find_element("card").unwrap();
    assert_eq!(element.context.as_deref(), Some("usage note"));
}

#[test]
fn content_editor_action_without_selection_only_updates_editor_text() {
    let doc = DesignDoc {
        children: vec![DesignElement { id: "label".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = super::editors::content_editor_action(&mut app, text_action("hello"));

    let state = app.active_design_state().unwrap();
    assert_eq!(state.content_editor.text(), "hello");
    assert_eq!(state.doc.find_element("label").unwrap().content.as_deref(), Some("hello"));
}

#[test]
fn tailwind_html_editor_action_updates_only_tailwind_elements() {
    let doc = DesignDoc {
        children: vec![
            DesignElement {
                id: "tw".to_string(),
                kind: "tailwind".to_string(),
                content: Some("<div>old</div>".to_string()),
                ..Default::default()
            },
            DesignElement {
                id: "text".to_string(),
                kind: "Text".to_string(),
                content: Some("old".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    app.active_design_state_mut().unwrap().selected_element_id = Some("tw".to_string());

    let _ = super::editors::tailwind_html_editor_action(&mut app, text_action("<p>new</p>"));
    assert_eq!(
        app.active_design_state().unwrap().doc.find_element("tw").unwrap().content.as_deref(),
        Some("<p>new</p>")
    );

    app.active_design_state_mut().unwrap().selected_element_id = Some("text".to_string());
    let _ = super::editors::tailwind_html_editor_action(&mut app, text_action(" ignored"));

    assert_eq!(
        app.active_design_state().unwrap().doc.find_element("text").unwrap().content.as_deref(),
        Some("old")
    );
}

#[test]
fn tailwind_node_text_editor_action_updates_nested_node_and_html_editor() {
    let doc = DesignDoc {
        children: vec![DesignElement {
            id: "tw".to_string(),
            kind: "tailwind".to_string(),
            content: Some("<div><span>old</span></div>".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    let state = app.active_design_state_mut().unwrap();
    state.selected_element_id = Some("tw".to_string());
    state.doc.tailwind_selection = Some(("tw".to_string(), vec![0, 0]));

    let _ = super::editors::tailwind_node_text_editor_action(&mut app, text_action("new"));

    let state = app.active_design_state().unwrap();
    let html = state.doc.find_element("tw").unwrap().content.as_deref().unwrap();
    assert!(html.contains("new"));
    assert_eq!(state.tailwind_html_editor.text(), html);
}

#[test]
fn toggle_context_editor_flips_expanded_state() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = super::editors::toggle_context_editor(&mut app);
    assert!(app.active_design_state().unwrap().context_expanded);

    let _ = super::editors::toggle_context_editor(&mut app);
    assert!(!app.active_design_state().unwrap().context_expanded);
}
