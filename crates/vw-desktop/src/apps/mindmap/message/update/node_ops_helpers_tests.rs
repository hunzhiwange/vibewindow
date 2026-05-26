use super::node_ops_helpers::{
    dir_for_node, remove_prefix, remove_prefix_set, shift_on_delete, shift_on_insert,
    shift_set_on_delete, shift_set_on_insert,
};
use crate::apps::mindmap::state::MindMapLayoutFormat;
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
}
