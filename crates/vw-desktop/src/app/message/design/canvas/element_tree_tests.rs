#![allow(unused_must_use)]
use super::element_tree::{insert_into_parent, reparent_elements};
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::DesignState;
use iced::{Point, Vector};

fn element(id: &str, x: f32, y: f32, width: f32, height: f32) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        x,
        y,
        width: Some(serde_json::json!(width)),
        height: Some(serde_json::json!(height)),
        ..Default::default()
    }
}

#[test]
fn insert_into_parent_adds_to_direct_or_nested_parent() {
    let mut children = vec![DesignElement {
        id: "parent".to_string(),
        children: vec![DesignElement { id: "nested".to_string(), ..Default::default() }],
        ..Default::default()
    }];

    assert!(
        insert_into_parent(&mut children, "parent", element("child", 0.0, 0.0, 1.0, 1.0)).is_ok()
    );
    assert_eq!(children[0].children.last().map(|child| child.id.as_str()), Some("child"));

    assert!(
        insert_into_parent(&mut children, "nested", element("grand", 0.0, 0.0, 1.0, 1.0)).is_ok()
    );
    assert_eq!(children[0].children[0].children[0].id, "grand");
}

#[test]
fn insert_into_parent_returns_element_when_parent_is_missing() {
    let mut children = vec![DesignElement { id: "root".to_string(), ..Default::default() }];
    let missing = element("orphan", 0.0, 0.0, 1.0, 1.0);

    let result = insert_into_parent(&mut children, "missing", missing).unwrap_err();

    assert_eq!(result.id, "orphan");
    assert!(children[0].children.is_empty());
}

#[test]
fn reparent_elements_moves_root_element_into_parent_with_relative_position() {
    let mut doc = DesignDoc::default();
    doc.children = vec![
        DesignElement {
            id: "frame".to_string(),
            kind: "frame".to_string(),
            x: 100.0,
            y: 50.0,
            width: Some(serde_json::json!(200.0)),
            height: Some(serde_json::json!(100.0)),
            padding: Some(serde_json::json!(10.0)),
            ..Default::default()
        },
        element("child", 150.0, 90.0, 20.0, 10.0),
    ];
    let mut state = DesignState::new(doc);

    reparent_elements(&mut state, vec!["child".to_string()], Some("frame".to_string()));

    let frame = state.doc.find_element("frame").unwrap();
    let child = frame.children.first().unwrap();
    assert_eq!(child.id, "child");
    assert_eq!(child.x, 40.0);
    assert_eq!(child.y, 30.0);
}

#[test]
fn reparent_elements_moves_child_to_root_when_parent_is_none() {
    let mut doc = DesignDoc::default();
    doc.children = vec![DesignElement {
        id: "frame".to_string(),
        x: 100.0,
        y: 100.0,
        width: Some(serde_json::json!(200.0)),
        height: Some(serde_json::json!(200.0)),
        children: vec![element("child", 5.0, 6.0, 10.0, 10.0)],
        ..Default::default()
    }];
    let mut state = DesignState::new(doc);
    state.pan = Vector::new(10.0, 20.0);
    state.zoom = 2.0;

    reparent_elements(&mut state, vec!["child".to_string()], None);

    assert!(state.doc.find_element("frame").unwrap().children.is_empty());
    let moved = state.doc.children.iter().find(|element| element.id == "child").unwrap();
    assert_eq!(Point::new(moved.x, moved.y), Point::new(105.0, 106.0));
}

#[test]
fn reparent_elements_prevents_moving_ancestor_into_descendant() {
    let mut doc = DesignDoc::default();
    doc.children = vec![DesignElement {
        id: "parent".to_string(),
        x: 10.0,
        y: 20.0,
        width: Some(serde_json::json!(100.0)),
        height: Some(serde_json::json!(100.0)),
        children: vec![element("child", 5.0, 6.0, 10.0, 10.0)],
        ..Default::default()
    }];
    let mut state = DesignState::new(doc);

    reparent_elements(&mut state, vec!["parent".to_string()], Some("child".to_string()));

    assert!(state.doc.children.iter().any(|element| element.id == "parent"));
    assert!(state.doc.find_element("child").is_some());
}
