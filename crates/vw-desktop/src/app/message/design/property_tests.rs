use crate::app::App;
use crate::app::message::design::{DesignMessage, property};
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;

fn element(id: &str) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        ..serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

fn app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("property_tests"));
}

#[test]
fn update_routes_selection_and_help_messages() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = property::update(&mut app, DesignMessage::SelectFill(Some(2)));
    assert_eq!(app.active_design_state().unwrap().selected_fill_index, Some(2));

    let _ = property::update(&mut app, DesignMessage::SelectEffect(Some(1)));
    assert_eq!(app.active_design_state().unwrap().selected_effect_index, Some(1));

    let _ = property::update(&mut app, DesignMessage::ShowHelpModal("help".to_string()));
    assert_eq!(app.active_design_state().unwrap().design_help_text.as_deref(), None);

    let _ = property::update(&mut app, DesignMessage::CloseHelpModal);
    assert!(app.active_design_state().unwrap().design_help_text.is_none());
}

#[test]
fn update_routes_property_and_transient_updates() {
    let doc = DesignDoc {
        version: "1.0".to_string(),
        children: vec![element("shape")],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = property::update(
        &mut app,
        DesignMessage::PropertyUpdate(
            "shape".to_string(),
            "x".to_string(),
            serde_json::json!(42.0),
        ),
    );
    assert_eq!(app.active_design_state().unwrap().doc.find_element("shape").unwrap().x, 42.0);

    let _ = property::update(
        &mut app,
        DesignMessage::PropertyUpdateTransient(
            "shape".to_string(),
            "y".to_string(),
            serde_json::json!(7.0),
        ),
    );
    assert_eq!(app.active_design_state().unwrap().doc.find_element("shape").unwrap().y, 7.0);
}

#[test]
fn update_ignores_unhandled_messages() {
    let mut app = app_with_doc(DesignDoc::default());

    let _ = property::update(&mut app, DesignMessage::ToggleVariables);

    assert!(app.active_design_state().is_some());
}
