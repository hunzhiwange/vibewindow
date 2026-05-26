use crate::error::{FigError, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// 从平面 nodeChanges 数组构建树结构
///
/// 采用平面节点数组并构建分层树结构
/// 通过基于parentIndex 字段创建父子关系。
///
/// # 参数
/// * `node_changes` - 来自解码的 Kiwi 数据的节点对象数组
///
/// # 返回值
/// * `Ok(JsonValue)` - 具有子层次结构的根节点
/// * `Err(FigError)` - 如果树构建失败
///
/// # 示例
/// ```no_run
/// use fig2json::schema::build_tree;
/// use serde_json::json;
///
/// let node_changes = vec![/* node objects */];
/// let root = build_tree(node_changes).unwrap();
/// ```
pub fn build_tree(node_changes: Vec<JsonValue>) -> Result<JsonValue> {
    // 1. 创建映射：GUID -> 父节点和映射 -> 子节点(位置，GUID)元组
    let mut nodes: HashMap<String, JsonValue> = HashMap::new();
    let mut parent_to_children: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for node in &node_changes {
        let guid = format_guid(node)?;
        nodes.insert(guid, node.clone());
    }

    // 2.建立父子关系(分别存储位置和GUID)
    for node in &node_changes {
        if let Some(parent_index) = node.get("parentIndex") {
            let parent_guid = format_parent_guid(parent_index)?;
            let child_guid = format_guid(node)?;
            let position =
                parent_index.get("position").and_then(|v| v.as_str()).unwrap_or("").to_string();

            parent_to_children.entry(parent_guid).or_default().push((position, child_guid));
        }
    }

    // 3. 按位置对子节点进行排序
    for children in parent_to_children.values_mut() {
        children.sort_by(|a, b| a.0.cmp(&b.0));
    }

    // 4. 从根开始递归构建树
    build_node_tree("0:0", &nodes, &parent_to_children)
}

/// 递归地构建一个节点及其子节点
fn build_node_tree(
    guid: &str,
    nodes: &HashMap<String, JsonValue>,
    parent_to_children: &HashMap<String, Vec<(String, String)>>,
) -> Result<JsonValue> {
    // 获取节点
    let mut node = nodes
        .get(guid)
        .ok_or_else(|| FigError::ZipError(format!("Node {} not found", guid)))?
        .clone();

    // 删除父索引
    if let Some(obj) = node.as_object_mut() {
        obj.remove("parentIndex");

        // 递归添加子项
        if let Some(child_entries) = parent_to_children.get(guid) {
            let mut children = Vec::new();
            for (_position, child_guid) in child_entries {
                let child_node = build_node_tree(child_guid, nodes, parent_to_children)?;
                children.push(child_node);
            }

            if !children.is_empty() {
                obj.insert("children".to_string(), JsonValue::Array(children));
            }
        }
    }

    Ok(node)
}

/// 从节点的 guid 字段格式化 GUID
///
/// 将 `{sessionID: X, localID: Y}` 转换为字符串 "X:Y"
fn format_guid(node: &JsonValue) -> Result<String> {
    let guid_obj = node
        .get("guid")
        .and_then(|v| v.as_object())
        .ok_or_else(|| FigError::ZipError("Node missing guid field".to_string()))?;

    let session_id = guid_obj
        .get("sessionID")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| FigError::ZipError("Invalid sessionID in guid".to_string()))?;

    let local_id = guid_obj
        .get("localID")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| FigError::ZipError("Invalid localID in guid".to_string()))?;

    Ok(format!("{}:{}", session_id, local_id))
}

/// 从parentIndex的guid字段格式化GUID
fn format_parent_guid(parent_index: &JsonValue) -> Result<String> {
    let guid_obj = parent_index
        .get("guid")
        .and_then(|v| v.as_object())
        .ok_or_else(|| FigError::ZipError("parentIndex missing guid field".to_string()))?;

    let session_id = guid_obj
        .get("sessionID")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| FigError::ZipError("Invalid sessionID in parentIndex".to_string()))?;

    let local_id = guid_obj
        .get("localID")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| FigError::ZipError("Invalid localID in parentIndex".to_string()))?;

    Ok(format!("{}:{}", session_id, local_id))
}

#[cfg(test)]
mod tests {
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
}
