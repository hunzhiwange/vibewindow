use crate::app::App;
use crate::app::message::design::{DesignMessage, LayerAction, layer};
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;

fn element(id: &str) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        ..serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

fn text_element(id: &str) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "text".to_string(),
        content: Some("hello".to_string()),
        context: Some("context".to_string()),
        ..serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

fn doc_with_tree() -> DesignDoc {
    let mut parent = element("parent");
    parent.children.push(text_element("child"));
    DesignDoc {
        version: "1.0".to_string(),
        children: vec![parent, element("sibling")],
        ..Default::default()
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

    assert!(module.ends_with("layer_tests"));
}

#[test]
fn panel_drag_hover_and_menu_messages_update_app_state() {
    let mut app = app_with_doc(doc_with_tree());
    app.show_layer_panel = false;

    let _ = layer::update(&mut app, DesignMessage::ToggleLayerPanel);
    assert!(app.show_layer_panel);

    let _ = layer::update(&mut app, DesignMessage::LayerPanelResizing(99.0));
    assert_eq!(app.layer_panel_width, 150.0);
    let _ = layer::update(&mut app, DesignMessage::LayerPanelResizing(999.0));
    assert_eq!(app.layer_panel_width, 500.0);

    let _ = layer::update(&mut app, DesignMessage::LayerDragStart("parent".to_string()));
    let _ = layer::update(&mut app, DesignMessage::LayerDragOver("sibling".to_string()));
    assert_eq!(app.dragging_layer.as_deref(), Some("parent"));
    assert_eq!(app.drag_target_layer.as_deref(), Some("sibling"));

    let _ = layer::update(&mut app, DesignMessage::LayerHover("child".to_string()));
    assert_eq!(app.hovered_layer_id.as_deref(), Some("child"));
    assert_eq!(app.drag_target_layer.as_deref(), Some("child"));
    let _ = layer::update(&mut app, DesignMessage::LayerHoverLeave);
    assert!(app.hovered_layer_id.is_none());

    let _ =
        layer::update(&mut app, DesignMessage::LayerMenuToggle("parent".to_string(), 10.0, 20.0));
    assert_eq!(app.active_layer_menu.as_deref(), Some("parent"));
    assert_eq!(app.layer_menu_anchor.unwrap().x, 10.0);
    let _ = layer::update(&mut app, DesignMessage::LayerMenuToggle("parent".to_string(), 0.0, 0.0));
    assert!(app.active_layer_menu.is_none());
    assert!(app.layer_menu_anchor.is_none());

    let _ = layer::update(&mut app, DesignMessage::LayerMenuHover("child".to_string()));
    assert_eq!(app.active_layer_menu.as_deref(), Some("child"));
    let _ = layer::update(&mut app, DesignMessage::LayerMenuLeave);
    assert!(app.active_layer_menu.is_none());
}

#[test]
fn element_selection_saves_editing_content_and_expands_path() {
    let mut app = app_with_doc(doc_with_tree());
    {
        let state = app.active_design_state_mut().unwrap();
        state.editing_id = Some("child".to_string());
        state.editing_editor = iced::widget::text_editor::Content::with_text("edited");
    }

    let _ = layer::update(&mut app, DesignMessage::ElementSelected("child".to_string()));

    let state = app.active_design_state().unwrap();
    assert_eq!(state.selected_element_id.as_deref(), Some("child"));
    assert!(state.selected_element_ids.contains("child"));
    assert!(state.expanded_nodes.contains("parent"));
    assert_eq!(state.context_element_id.as_deref(), Some("child"));
    assert!(state.context_expanded);
    assert_eq!(state.doc.find_element("child").unwrap().content.as_deref(), Some("edited"));
}

#[test]
fn row_pressed_toggles_expand_only_when_node_has_children() {
    let mut app = app_with_doc(doc_with_tree());

    let _ = layer::update(&mut app, DesignMessage::LayerRowPressed("parent".to_string()));
    assert!(app.active_design_state().unwrap().expanded_nodes.contains("parent"));

    let _ = layer::update(&mut app, DesignMessage::LayerRowPressed("parent".to_string()));
    assert!(!app.active_design_state().unwrap().expanded_nodes.contains("parent"));

    let _ = layer::update(&mut app, DesignMessage::LayerRowPressed("sibling".to_string()));
    assert!(!app.active_design_state().unwrap().expanded_nodes.contains("sibling"));
}

#[test]
fn multi_select_sets_primary_or_clears_editors() {
    let mut app = app_with_doc(doc_with_tree());

    let _ = layer::update(
        &mut app,
        DesignMessage::MultiSelect(vec!["child".to_string(), "sibling".to_string()]),
    );
    let state = app.active_design_state().unwrap();
    assert_eq!(state.selected_element_id.as_deref(), Some("child"));
    assert!(state.selected_element_ids.contains("child"));
    assert!(state.selected_element_ids.contains("sibling"));
    assert_eq!(state.context_element_id.as_deref(), Some("child"));

    let _ = layer::update(&mut app, DesignMessage::MultiSelect(Vec::new()));
    let state = app.active_design_state().unwrap();
    assert!(state.selected_element_id.is_none());
    assert!(state.selected_element_ids.is_empty());
    assert!(state.context_element_id.is_none());
}

#[test]
fn toggle_node_visible_move_and_delete_update_doc() {
    let mut app = app_with_doc(doc_with_tree());

    let _ = layer::update(&mut app, DesignMessage::ToggleNode("parent".to_string()));
    assert!(app.active_design_state().unwrap().expanded_nodes.contains("parent"));

    let _ = layer::update(&mut app, DesignMessage::ToggleVisible("sibling".to_string()));
    assert_eq!(
        app.active_design_state().unwrap().doc.find_element("sibling").unwrap().visible,
        Some(false)
    );

    let _ = layer::update(&mut app, DesignMessage::MoveLayerItem("sibling".to_string(), -1));
    assert_eq!(app.active_design_state().unwrap().doc.children[0].id, "sibling");

    let _ = layer::update(
        &mut app,
        DesignMessage::LayerActionSelected("sibling".to_string(), LayerAction::Delete),
    );
    assert!(app.active_design_state().unwrap().doc.find_element("sibling").is_none());
}

#[test]
fn drop_moves_dragged_node_before_target_or_to_end_when_target_missing() {
    let mut app = app_with_doc(doc_with_tree());
    app.dragging_layer = Some("sibling".to_string());
    app.drag_target_layer = Some("parent".to_string());

    let _ = layer::update(&mut app, DesignMessage::LayerDrop);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.children[0].id, "sibling");
    assert!(app.dragging_layer.is_none());
    assert!(app.drag_target_layer.is_none());

    app.dragging_layer = Some("sibling".to_string());
    app.drag_target_layer = Some("missing".to_string());
    let _ = layer::update(&mut app, DesignMessage::LayerDrop);
    assert_eq!(app.active_design_state().unwrap().doc.children.last().unwrap().id, "sibling");
}
