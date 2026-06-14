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

#[test]
fn class_picker_open_close_and_filter_update_app_state() {
    let mut app = app_with_doc(DesignDoc::default());
    app.cursor_position = iced::Point::new(7.0, 8.0);

    let _ = super::tailwind::set_filter(&mut app, "grid".to_string());
    assert_eq!(app.tailwind_filter_query, "grid");

    let _ = super::tailwind::open_class_picker(&mut app, "el".to_string(), None);
    let picker = app.active_tailwind_class_picker.as_ref().unwrap();
    assert_eq!(picker.element_id, "el");
    assert_eq!(picker.position, iced::Point::new(7.0, 8.0));
    assert!(app.tailwind_filter_query.is_empty());

    let _ = super::tailwind::close_class_picker(&mut app);
    assert!(app.active_tailwind_class_picker.is_none());
}

#[test]
fn class_input_changed_accepts_only_selected_element_and_normalizes_newlines() {
    let doc = DesignDoc {
        children: vec![DesignElement { id: "el".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    app.active_design_state_mut().unwrap().selected_element_id = Some("el".to_string());

    let _ = super::tailwind::class_input_changed(&mut app, "other".to_string(), "p-2".to_string());
    assert!(app.active_design_state().unwrap().tailwind_class_input.is_empty());

    let _ =
        super::tailwind::class_input_changed(&mut app, "el".to_string(), "p-2\nmt-4".to_string());
    assert_eq!(app.active_design_state().unwrap().tailwind_class_input, "p-2 mt-4");
}

#[test]
fn class_input_submit_appends_unique_tokens_and_clears_input() {
    let doc = DesignDoc {
        children: vec![DesignElement {
            id: "el".to_string(),
            class: Some("p-2".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    let state = app.active_design_state_mut().unwrap();
    state.selected_element_id = Some("el".to_string());
    state.tailwind_class_input = "p-2 mt-4".to_string();

    let _ = super::tailwind::class_input_submit(&mut app, "el".to_string());

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.find_element("el").unwrap().class.as_deref(), Some("p-2 mt-4"));
    assert!(state.tailwind_class_input.is_empty());
}

#[test]
fn node_class_input_tracks_selection_and_submit_returns_update_message() {
    let mut app = app_with_doc(DesignDoc::default());
    let state = app.active_design_state_mut().unwrap();
    state.doc.tailwind_selection = Some(("tw".to_string(), vec![0]));
    state.tailwind_node_class_editor = iced::widget::text_editor::Content::with_text("text-sm");

    let _ = super::tailwind::node_class_input_changed(
        &mut app,
        "tw".to_string(),
        vec![0],
        "font-bold\ntext-sm".to_string(),
    );
    let state = app.active_design_state().unwrap();
    assert_eq!(state.tailwind_node_class_input, "font-bold text-sm");
    assert!(state.tailwind_node_class_dropdown_open);

    let _ = super::tailwind::close_node_class_dropdown(&mut app, "tw".to_string(), vec![0]);
    assert!(!app.active_design_state().unwrap().tailwind_node_class_dropdown_open);

    let _ = super::tailwind::node_class_input_submit(&mut app, "tw".to_string(), vec![0]);
    let state = app.active_design_state().unwrap();
    assert!(state.tailwind_node_class_input.is_empty());
    assert!(!state.tailwind_node_class_dropdown_open);
}

#[test]
fn add_class_token_trims_empty_tokens_and_appends_unique_token() {
    let doc = DesignDoc {
        children: vec![DesignElement {
            id: "el".to_string(),
            class: Some("p-2".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = super::tailwind::add_class_token(&mut app, "el".to_string(), "  ".to_string());
    assert_eq!(
        app.active_design_state().unwrap().doc.find_element("el").unwrap().class.as_deref(),
        Some("p-2")
    );

    let _ = super::tailwind::add_class_token(&mut app, "el".to_string(), " mt-4 ".to_string());
    assert_eq!(
        app.active_design_state().unwrap().doc.find_element("el").unwrap().class.as_deref(),
        Some("p-2 mt-4")
    );
}

#[test]
fn inspector_hover_and_node_editor_actions_update_state() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = super::tailwind::set_inspector_hover(&mut app, true);
    assert!(app.active_design_state().unwrap().tailwind_inspector_hovered);

    let _ = super::tailwind::node_class_input_changed(
        &mut app,
        "tw".to_string(),
        vec![9],
        "ignored".to_string(),
    );
    assert!(app.active_design_state().unwrap().tailwind_node_class_input.is_empty());

    let _ = super::tailwind::add_class_token(&mut app, "missing".to_string(), "p-2".to_string());
    let _ = super::tailwind::node_class_input_submit(&mut app, "missing".to_string(), vec![]);

    let action = Action::Edit(Edit::Paste("typed".to_string().into()));
    let _ = super::editors::tailwind_node_class_editor_action(&mut app, action);
    assert_eq!(app.active_design_state().unwrap().tailwind_node_class_editor.text(), "typed");
}
