use crate::app::App;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;
use serde_json::json;

fn app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

#[test]
fn property_update_writes_value_and_recomputes_metrics_for_name() {
    let doc = DesignDoc {
        children: vec![DesignElement {
            id: "el".to_string(),
            kind: "Frame".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = super::updates::property_update(
        &mut app,
        "el".to_string(),
        "name".to_string(),
        json!("Long Layer Name"),
    );

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.find_element("el").unwrap().name.as_deref(), Some("Long Layer Name"));
    assert!(state.layer_tree_metrics.0 >= "Long Layer Name".len());
}

#[test]
fn property_update_reapplies_tailwind_class_to_element() {
    let doc = DesignDoc {
        children: vec![DesignElement { id: "el".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = super::updates::property_update(
        &mut app,
        "el".to_string(),
        "class".to_string(),
        json!("w-10 h-12 opacity-50"),
    );

    let element = app.active_design_state().unwrap().doc.find_element("el").unwrap();
    assert_eq!(element.class.as_deref(), Some("w-10 h-12 opacity-50"));
    assert_eq!(element.opacity, None);
}

#[test]
fn properties_update_applies_multiple_values_and_class_rules() {
    let doc = DesignDoc {
        children: vec![DesignElement { id: "el".to_string(), ..Default::default() }],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = super::updates::properties_update(
        &mut app,
        "el".to_string(),
        vec![
            ("x".to_string(), json!(12.0)),
            ("class".to_string(), json!("opacity-25")),
            ("name".to_string(), json!("Text Layer")),
        ],
    );

    let state = app.active_design_state().unwrap();
    let element = state.doc.find_element("el").unwrap();
    assert_eq!(element.x, 12.0);
    assert_eq!(element.opacity, None);
    assert!(state.layer_tree_metrics.0 >= "Text Layer".len());
}

#[test]
fn batch_properties_update_updates_each_target() {
    let doc = DesignDoc {
        children: vec![
            DesignElement { id: "a".to_string(), ..Default::default() },
            DesignElement { id: "b".to_string(), ..Default::default() },
        ],
        ..Default::default()
    };
    let mut app = app_with_doc(doc);

    let _ = super::updates::batch_properties_update(
        &mut app,
        vec![
            ("a".to_string(), vec![("name".to_string(), json!("Alpha"))]),
            ("b".to_string(), vec![("color".to_string(), json!("#fff"))]),
        ],
    );

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.find_element("a").unwrap().name.as_deref(), Some("Alpha"));
    assert_eq!(state.doc.find_element("b").unwrap().color.as_deref(), Some("#fff"));
    assert!(state.layer_tree_metrics.0 >= "Alpha".len());
}
