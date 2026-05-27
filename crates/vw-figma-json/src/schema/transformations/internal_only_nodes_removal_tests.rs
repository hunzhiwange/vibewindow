    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_internal_only_node() {
        let mut tree = json!({
            "children": [
                {"name": "Visible", "visible": true},
                {"name": "Internal", "internalOnly": true, "visible": false}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["name"].as_str(), Some("Visible"));
    }

    #[test]
    fn test_preserve_visible_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Node1", "visible": true},
                {"name": "Node2", "visible": true},
                {"name": "Node3", "visible": false}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // 所有没有internalOnly的节点都应该被保留
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_remove_multiple_internal_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Visible1", "visible": true},
                {"name": "Internal1", "internalOnly": true},
                {"name": "Visible2", "visible": true},
                {"name": "Internal2", "internalOnly": true}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0]["name"].as_str(), Some("Visible1"));
        assert_eq!(children[1]["name"].as_str(), Some("Visible2"));
    }

    #[test]
    fn test_all_internal_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Internal1", "internalOnly": true},
                {"name": "Internal2", "internalOnly": true}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // 删除所有节点，空数组
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn test_no_internal_nodes() {
        let mut tree = json!({
            "children": [
                {"name": "Node1"},
                {"name": "Node2"}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // 保留所有节点
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_nested_children() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Parent",
                    "children": [
                        {"name": "Child1", "visible": true},
                        {"name": "Internal", "internalOnly": true}
                    ]
                }
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let parent_children = tree["children"][0]["children"].as_array().unwrap();
        assert_eq!(parent_children.len(), 1);
        assert_eq!(parent_children[0]["name"].as_str(), Some("Child1"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "name": "Canvas",
                        "children": [
                            {"name": "Frame", "visible": true},
                            {"name": "Internal Canvas", "internalOnly": true}
                        ]
                    }
                ]
            }
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let canvas_children = tree["document"]["children"][0]["children"].as_array().unwrap();
        assert_eq!(canvas_children.len(), 1);
        assert_eq!(canvas_children[0]["name"].as_str(), Some("Frame"));
    }

    #[test]
    fn test_internal_only_false() {
        let mut tree = json!({
            "children": [
                {"name": "Node1", "internalOnly": false},
                {"name": "Node2", "internalOnly": true}
            ]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        // 只有internalOnly: true 应该被过滤
        assert_eq!(children.len(), 1);
        assert_eq!(children[0]["name"].as_str(), Some("Node1"));
        // 内部仅字段应从保留的节点中删除
        assert!(children[0].get("internalOnly").is_none());
    }

    #[test]
    fn test_remove_internal_only_field() {
        let mut tree = json!({
            "name": "Node",
            "internalOnly": false,
            "visible": true
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        // 内部唯一字段即使为 false 也应删除
        assert!(tree.get("internalOnly").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_non_object_array_elements() {
        let mut tree = json!({
            "data": [1, 2, 3, "string"]
        });

        remove_internal_only_nodes(&mut tree).unwrap();

        let data = tree.get("data").unwrap().as_array().unwrap();
        // 应保留非对象元素
        assert_eq!(data.len(), 4);
    }
