use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中删除仅限内部的节点
///
/// 递归遍历JSON树，过滤掉有的节点
/// `internalOnly: true`。这些是 Figma 内部节点，并不意味着
/// 用于渲染，不应包含在最终输出中。
///
/// 仅内部节点通常从 "children" 数组中删除。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有内部节点
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_internal_only_nodes;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "children": [
///         {"name": "Visible", "visible": true},
///         {"name": "Internal", "internalOnly": true}
///     ]
/// });
/// remove_internal_only_nodes(&mut tree).unwrap();
/// // 子数组现在只包含 "Visible" 节点
/// ```
pub fn remove_internal_only_nodes(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除仅限内部的节点
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 递归到所有值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }

            // 递归后，删除internalOnly字段本身
            // (仅用于过滤，输出中不需要)
            map.remove("internalOnly");
        }
        JsonValue::Array(arr) => {
            // 用internalOnly: true FIRST 过滤掉节点(在递归之前)
            arr.retain(|node| {
                if let Some(obj) = node.as_object() {
                    // 如果internalOnly不为true则保留节点
                    !obj.get("internalOnly").and_then(|v| v.as_bool()).unwrap_or(false)
                } else {
                    // 保留非对象值
                    true
                }
            });

            // 然后递归到剩余的数组元素
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
}
