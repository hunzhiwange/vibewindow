use super::node_ops_helpers::{
    close_context_menu_tab, dir_for_node, push_undo, relayout_keep_root, remove_prefix,
    remove_prefix_set, shift_on_delete, shift_on_insert, shift_positions_on_insert,
    shift_set_on_delete, shift_set_on_insert,
};
use crate::app::components::mind_map;
use crate::apps::mindmap::state::{MindMapLayoutFormat, MindMapTab};
use iced::{Point, Vector};
use std::collections::{HashMap, HashSet};

#[test]
fn remove_prefix_drops_nested_paths_only() {
    let mut map = HashMap::from([(vec![0], "a"), (vec![0, 1], "b"), (vec![1], "c")]);
    let mut set = HashSet::from([vec![0], vec![0, 1], vec![1]]);

    remove_prefix(&mut map, &[0]);
    remove_prefix_set(&mut set, &[0]);

    assert_eq!(map.keys().cloned().collect::<Vec<_>>(), vec![vec![1]]);
    assert_eq!(set, HashSet::from([vec![1]]));
}

#[test]
fn insert_and_delete_shift_paths_under_parent() {
    let mut map = HashMap::from([(vec![2, 0], "a"), (vec![2, 1], "b"), (vec![3, 1], "c")]);
    let mut set = HashSet::from([vec![2, 0], vec![2, 1], vec![3, 1]]);

    shift_on_insert(&mut map, &[2], 1);
    shift_set_on_insert(&mut set, &[2], 1);
    assert!(map.contains_key(&vec![2, 2]));
    assert!(set.contains(&vec![2, 2]));

    shift_on_delete(&mut map, &[2], 0);
    shift_set_on_delete(&mut set, &[2], 0);
    assert!(map.contains_key(&vec![2, 1]));
    assert!(set.contains(&vec![2, 1]));
}

#[test]
fn dir_for_node_matches_layout_format() {
    assert_eq!(dir_for_node(MindMapLayoutFormat::RightAligned, &[1]), 1.0);
    assert_eq!(dir_for_node(MindMapLayoutFormat::LeftAligned, &[0]), -1.0);
    assert_eq!(dir_for_node(MindMapLayoutFormat::Bidirectional, &[0]), 1.0);
    assert_eq!(dir_for_node(MindMapLayoutFormat::Bidirectional, &[1]), -1.0);
    assert_eq!(dir_for_node(MindMapLayoutFormat::Bidirectional, &[]), 1.0);
}

#[test]
fn shift_positions_offsets_matching_siblings_only() {
    let mut map = HashMap::from([
        (vec![1, 0], Point::new(1.0, 1.0)),
        (vec![1, 2], Point::new(2.0, 2.0)),
        (vec![2, 0], Point::new(3.0, 3.0)),
    ]);

    shift_positions_on_insert(&mut map, &[1], 1, Vector::new(10.0, -3.0));

    assert_eq!(map.get(&vec![1, 0]).unwrap().x, 1.0);
    assert_eq!(map.get(&vec![1, 2]).unwrap().x, 12.0);
    assert_eq!(map.get(&vec![1, 2]).unwrap().y, -1.0);
    assert_eq!(map.get(&vec![2, 0]).unwrap().x, 3.0);
}

#[test]
fn close_context_menu_and_push_undo_update_tab_state() {
    let mut tab = MindMapTab::new(
        "tab".to_string(),
        "Map".to_string(),
        None,
        mind_map::parse("# Root\n\n- A\n"),
    );
    tab.show_context_menu = true;
    tab.context_menu_anchor = Some(Point::new(1.0, 2.0));

    close_context_menu_tab(&mut tab);
    assert!(!tab.show_context_menu);
    assert!(tab.context_menu_anchor.is_none());

    for i in 0..55 {
        tab.doc.text = format!("Root {i}");
        push_undo(&mut tab);
        tab.redo_stack.push(mind_map::parse("# Redo\n"));
    }
    assert_eq!(tab.undo_stack.len(), 50);
    assert!(tab.undo_stack.first().unwrap().text.starts_with("Root 5"));
    assert!(!tab.redo_stack.is_empty());
}

#[test]
fn relayout_keep_root_preserves_existing_root_position() {
    let mut tab = MindMapTab::new(
        "tab".to_string(),
        "Map".to_string(),
        None,
        mind_map::parse("# Root\n\n- A\n- B\n"),
    );
    tab.node_positions.insert(Vec::new(), Point::new(100.0, 200.0));

    relayout_keep_root(&mut tab);

    let root = tab.node_positions.get(&Vec::new()).unwrap();
    assert!((root.x - 100.0).abs() < 0.01);
    assert!((root.y - 200.0).abs() < 0.01);
    assert!(tab.node_positions.contains_key(&vec![0]));
    assert!(tab.node_positions.contains_key(&vec![1]));
}
