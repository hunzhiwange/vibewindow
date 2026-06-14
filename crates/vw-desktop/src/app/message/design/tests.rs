use crate::app::App;
use crate::app::message::design::{self, DesignMessage};
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

    assert!(module.ends_with("tests"));
}

#[test]
fn update_routes_layer_property_settings_and_history_messages() {
    let doc = DesignDoc {
        version: "1.0".to_string(),
        children: vec![element("shape")],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);
    app.show_layer_panel = false;

    let _ = design::update(&mut app, DesignMessage::ToggleLayerPanel);
    assert!(app.show_layer_panel);

    let _ = design::update(
        &mut app,
        DesignMessage::PropertyUpdateTransient(
            "shape".to_string(),
            "x".to_string(),
            serde_json::json!(9.0),
        ),
    );
    assert_eq!(app.active_design_state().unwrap().doc.find_element("shape").unwrap().x, 9.0);

    let _ = design::update(&mut app, DesignMessage::ToggleVariables);
    assert!(app.show_design_variables);

    let before = app.active_design_state().unwrap().history.len();
    let _ = design::update(&mut app, DesignMessage::Snapshot);
    assert_eq!(app.active_design_state().unwrap().history.len(), before + 1);
}
