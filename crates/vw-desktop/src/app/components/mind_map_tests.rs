use super::mind_map::{
    MindNode, add_child, add_sibling, delete_node, insert_child_node, insert_sibling_node, node,
    node_mut, node_text, node_text_mut, parse, path_exists, take_node, to_markdown,
};

fn sample_tree() -> MindNode {
    parse(
        r#"
# Root
- One
  - One A
    - One A i
* Two
	+ Two A
1. Three
2) Four
not a list item
"#,
    )
}

#[test]
fn task_745_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("mind_map_tests.rs"));
}

#[test]
fn default_node_uses_localized_root_text() {
    assert_eq!(MindNode::default(), MindNode { text: "思维导图".to_string(), children: vec![] });
}

#[test]
fn parse_uses_heading_root_and_accepts_common_list_markers() {
    let root = sample_tree();

    assert_eq!(root.text, "Root");
    assert_eq!(
        root.children.iter().map(|n| n.text.as_str()).collect::<Vec<_>>(),
        ["One", "Two", "Three", "Four"]
    );
    assert_eq!(node_text(&root, &[0, 0]), Some("One A"));
    assert_eq!(node_text(&root, &[0, 0, 0]), Some("One A i"));
    assert_eq!(node_text(&root, &[1, 0]), Some("Two A"));
}

#[test]
fn parse_defaults_root_and_clamps_over_deep_indentation() {
    let root = parse("      - Deep\n          - Deeper");

    assert_eq!(root.text, "思维导图");
    assert_eq!(node_text(&root, &[0]), Some("Deep"));
    assert_eq!(node_text(&root, &[0, 0]), Some("Deeper"));
    assert_eq!(node_text(&root, &[0, 0, 0]), None);
}

#[test]
fn to_markdown_trims_node_text_and_renders_children() {
    let root = MindNode {
        text: "  Root  ".to_string(),
        children: vec![MindNode {
            text: "  Child  ".to_string(),
            children: vec![MindNode { text: " Grand ".to_string(), children: vec![] }],
        }],
    };

    assert_eq!(to_markdown(&root), "# Root\n\n- Child\n  - Grand\n");
}

#[test]
fn node_accessors_return_none_for_invalid_paths_and_mutate_valid_paths() {
    let mut root = sample_tree();

    assert!(path_exists(&root, &[0, 0]));
    assert!(!path_exists(&root, &[99]));
    assert_eq!(node(&root, &[3]).map(|n| n.text.as_str()), Some("Four"));
    assert!(node(&root, &[3, 0]).is_none());

    *node_text_mut(&mut root, &[0, 0]).expect("text") = "Renamed".to_string();
    assert_eq!(node_text(&root, &[0, 0]), Some("Renamed"));

    node_mut(&mut root, &[1]).expect("node").children.clear();
    assert!(node(&root, &[1]).expect("node").children.is_empty());
}

#[test]
fn add_and_insert_child_return_new_paths() {
    let mut root = MindNode::default();

    assert_eq!(add_child(&mut root, &[], "A".to_string()), Some(vec![0]));
    assert_eq!(
        insert_child_node(&mut root, &[0], MindNode { text: "A1".to_string(), children: vec![] },),
        Some(vec![0, 0])
    );
    assert_eq!(node_text(&root, &[0, 0]), Some("A1"));
    assert_eq!(add_child(&mut root, &[99], "missing".to_string()), None);
}

#[test]
fn sibling_insertion_handles_root_path_and_out_of_range_index() {
    let mut root = MindNode::default();

    assert_eq!(add_sibling(&mut root, &[], "first".to_string()), Some(vec![0]));
    assert_eq!(add_sibling(&mut root, &[0], "second".to_string()), Some(vec![1]));
    assert_eq!(
        insert_sibling_node(
            &mut root,
            &[20],
            MindNode { text: "last".to_string(), children: vec![] },
        ),
        Some(vec![2])
    );
    assert_eq!(
        root.children.iter().map(|n| n.text.as_str()).collect::<Vec<_>>(),
        ["first", "second", "last"]
    );
    assert_eq!(add_sibling(&mut root, &[9, 9], "missing".to_string()), None);
}

#[test]
fn take_and_delete_node_remove_children_and_return_parent_path() {
    let mut root = sample_tree();

    let taken = take_node(&mut root, &[0, 0]).expect("taken node");
    assert_eq!(taken.text, "One A");
    assert_eq!(node_text(&root, &[0, 0]), None);
    assert_eq!(take_node(&mut root, &[]), None);
    assert_eq!(take_node(&mut root, &[99]), None);

    assert_eq!(delete_node(&mut root, &[1, 0]), Some(vec![1]));
    assert_eq!(node_text(&root, &[1, 0]), None);
    assert_eq!(delete_node(&mut root, &[]), None);
    assert_eq!(delete_node(&mut root, &[99]), None);
}
