    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_guid() {
        let node = json!({
            "guid": {
                "sessionID": 1,
                "localID": 42
            }
        });

        assert_eq!(format_guid(&node).unwrap(), "1:42");
    }

    #[test]
    fn test_format_parent_guid() {
        let parent_index = json!({
            "guid": {
                "sessionID": 0,
                "localID": 1
            },
            "position": "!"
        });

        assert_eq!(format_parent_guid(&parent_index).unwrap(), "0:1");
    }

    #[test]
    fn test_build_tree_simple() {
        let node_changes = vec![
            json!({
                "guid": {"sessionID": 0, "localID": 0},
                "name": "Root",
                "type": "DOCUMENT"
            }),
            json!({
                "guid": {"sessionID": 0, "localID": 1},
                "parentIndex": {
                    "guid": {"sessionID": 0, "localID": 0},
                    "position": "a"
                },
                "name": "Child1"
            }),
            json!({
                "guid": {"sessionID": 0, "localID": 2},
                "parentIndex": {
                    "guid": {"sessionID": 0, "localID": 0},
                    "position": "b"
                },
                "name": "Child2"
            }),
        ];

        let root = build_tree(node_changes).unwrap();

        // 检查根目录
        assert_eq!(root.get("name").and_then(|v| v.as_str()), Some("Root"));

        // 检查子节点
        let children = root.get("children").and_then(|v| v.as_array()).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].get("name").and_then(|v| v.as_str()), Some("Child1"));
        assert_eq!(children[1].get("name").and_then(|v| v.as_str()), Some("Child2"));

        // 检查parentIndex是否被删除
        assert!(children[0].get("parentIndex").is_none());
    }

    #[test]
    fn test_sort_children_by_position() {
        let node_changes = vec![
            json!({
                "guid": {"sessionID": 0, "localID": 0},
                "name": "Root"
            }),
            json!({
                "guid": {"sessionID": 0, "localID": 1},
                "parentIndex": {
                    "guid": {"sessionID": 0, "localID": 0},
                    "position": "z"  // Should be last
                },
                "name": "Third"
            }),
            json!({
                "guid": {"sessionID": 0, "localID": 2},
                "parentIndex": {
                    "guid": {"sessionID": 0, "localID": 0},
                    "position": "a"  // Should be first
                },
                "name": "First"
            }),
            json!({
                "guid": {"sessionID": 0, "localID": 3},
                "parentIndex": {
                    "guid": {"sessionID": 0, "localID": 0},
                    "position": "m"  // Should be second
                },
                "name": "Second"
            }),
        ];

        let root = build_tree(node_changes).unwrap();
        let children = root.get("children").and_then(|v| v.as_array()).unwrap();

        // 检查排序顺序
        assert_eq!(children[0].get("name").and_then(|v| v.as_str()), Some("First"));
        assert_eq!(children[1].get("name").and_then(|v| v.as_str()), Some("Second"));
        assert_eq!(children[2].get("name").and_then(|v| v.as_str()), Some("Third"));
    }
