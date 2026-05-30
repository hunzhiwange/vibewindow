use super::*;
use serde_json::json;

#[test]
fn test_substitute_commands_blob() {
    // 创建一个简单的命令 blob (M 10 20 Z)
    let mut blob_bytes = Vec::new();
    blob_bytes.push(1); // M
    blob_bytes.extend_from_slice(&10.0f32.to_le_bytes());
    blob_bytes.extend_from_slice(&20.0f32.to_le_bytes());
    blob_bytes.push(0); // Z

    let blobs = vec![json!({
        "bytes": blob_bytes
    })];

    let mut tree = json!({
        "name": "Rectangle",
        "commandsBlob": 0
    });

    substitute_blobs(&mut tree, &blobs).unwrap();

    // 检查commandsBlob 是否已替换为命令
    assert!(tree.get("commandsBlob").is_none());
    assert!(tree.get("commands").is_some());

    let commands = tree.get("commands").unwrap().as_array().unwrap();
    assert_eq!(commands[0].as_str(), Some("M"));
    assert_eq!(commands[1].as_f64(), Some(10.0));
    assert_eq!(commands[2].as_f64(), Some(20.0));
    assert_eq!(commands[3].as_str(), Some("Z"));
}

#[test]
fn test_substitute_vector_network_blob() {
    // 创建一个简单的矢量网络 blob(2 个顶点、1 个线段、0 个区域)
    let mut blob_bytes = Vec::new();
    blob_bytes.extend_from_slice(&2u32.to_le_bytes()); // 2 vertices
    blob_bytes.extend_from_slice(&1u32.to_le_bytes()); // 1 segment
    blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // 0 regions

    // 顶点 0
    blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // styleID
    blob_bytes.extend_from_slice(&10.0f32.to_le_bytes()); // x
    blob_bytes.extend_from_slice(&20.0f32.to_le_bytes()); // y

    // 顶点 1
    blob_bytes.extend_from_slice(&0u32.to_le_bytes());
    blob_bytes.extend_from_slice(&30.0f32.to_le_bytes());
    blob_bytes.extend_from_slice(&40.0f32.to_le_bytes());

    // 段 0
    blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // styleID
    blob_bytes.extend_from_slice(&0u32.to_le_bytes()); // start vertex
    blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // start dx
    blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // start dy
    blob_bytes.extend_from_slice(&1u32.to_le_bytes()); // end vertex
    blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // end dx
    blob_bytes.extend_from_slice(&0.0f32.to_le_bytes()); // end dy

    let blobs = vec![json!({
        "bytes": blob_bytes
    })];

    let mut tree = json!({
        "name": "Vector",
        "vectorNetworkBlob": 0
    });

    substitute_blobs(&mut tree, &blobs).unwrap();

    // 检查vectorNetworkBlob是否已替换为vectorNetwork
    assert!(tree.get("vectorNetworkBlob").is_none());
    assert!(tree.get("vectorNetwork").is_some());

    let network = tree.get("vectorNetwork").unwrap();
    assert!(network.get("vertices").is_some());
    assert!(network.get("segments").is_some());
    assert!(network.get("regions").is_some());

    let vertices = network.get("vertices").unwrap().as_array().unwrap();
    assert_eq!(vertices.len(), 2);
}

#[test]
fn test_substitute_nested_tree() {
    // 创建命令 blob
    let mut blob_bytes = Vec::new();
    blob_bytes.push(1); // M
    blob_bytes.extend_from_slice(&5.0f32.to_le_bytes());
    blob_bytes.extend_from_slice(&10.0f32.to_le_bytes());
    blob_bytes.push(0); // Z

    let blobs = vec![json!({
        "bytes": blob_bytes
    })];

    let mut tree = json!({
        "name": "Root",
        "children": [
            {
                "name": "Child1",
                "commandsBlob": 0
            },
            {
                "name": "Child2",
                "children": [
                    {
                        "name": "GrandChild",
                        "commandsBlob": 0
                    }
                ]
            }
        ]
    });

    substitute_blobs(&mut tree, &blobs).unwrap();

    // 检查所有commandBlob 引用是否已替换
    let child1 = &tree["children"][0];
    assert!(child1.get("commandsBlob").is_none());
    assert!(child1.get("commands").is_some());

    let grandchild = &tree["children"][1]["children"][0];
    assert!(grandchild.get("commandsBlob").is_none());
    assert!(grandchild.get("commands").is_some());
}

#[test]
fn test_substitute_multiple_blob_types() {
    // 创建命令和 vectorNetwork blob
    let mut commands_bytes = Vec::new();
    commands_bytes.push(1); // M
    commands_bytes.extend_from_slice(&1.0f32.to_le_bytes());
    commands_bytes.extend_from_slice(&2.0f32.to_le_bytes());
    commands_bytes.push(0); // Z

    let mut network_bytes = Vec::new();
    network_bytes.extend_from_slice(&1u32.to_le_bytes()); // 1 vertex
    network_bytes.extend_from_slice(&0u32.to_le_bytes()); // 0 segments
    network_bytes.extend_from_slice(&0u32.to_le_bytes()); // 0 regions
    network_bytes.extend_from_slice(&0u32.to_le_bytes()); // vertex styleID
    network_bytes.extend_from_slice(&5.0f32.to_le_bytes()); // vertex x
    network_bytes.extend_from_slice(&5.0f32.to_le_bytes()); // vertex y

    let blobs = vec![json!({"bytes": commands_bytes}), json!({"bytes": network_bytes})];

    let mut tree = json!({
        "name": "Shape",
        "commandsBlob": 0,
        "vectorNetworkBlob": 1
    });

    substitute_blobs(&mut tree, &blobs).unwrap();

    // 两个 blob 字段都应替换
    assert!(tree.get("commandsBlob").is_none());
    assert!(tree.get("vectorNetworkBlob").is_none());
    assert!(tree.get("commands").is_some());
    assert!(tree.get("vectorNetwork").is_some());
}

#[test]
fn test_substitute_unknown_blob_type() {
    let blobs = vec![json!({
        "bytes": vec![1, 2, 3, 4]
    })];

    let mut tree = json!({
        "name": "Node",
        "unknownBlob": 0
    });

    // 不应失败，只需按原样保留该字段即可
    substitute_blobs(&mut tree, &blobs).unwrap();

    // 未知的 blob 类型应保持不变
    assert!(tree.get("unknownBlob").is_some());
    assert_eq!(tree.get("unknownBlob").unwrap().as_u64(), Some(0));
}

#[test]
fn test_substitute_invalid_blob_index() {
    let blobs = vec![json!({
        "bytes": vec![1, 2, 3]
    })];

    let mut tree = json!({
        "name": "Node",
        "commandsBlob": 999  // Out of range
    });

    // 不应失败，只需按原样保留该字段即可
    substitute_blobs(&mut tree, &blobs).unwrap();

    // 超出范围的索引应保持不变
    assert!(tree.get("commandsBlob").is_some());
    assert_eq!(tree.get("commandsBlob").unwrap().as_u64(), Some(999));
}

#[test]
fn test_substitute_preserves_other_fields() {
    let blob_bytes = vec![0]; // Z

    let blobs = vec![json!({
        "bytes": blob_bytes
    })];

    let mut tree = json!({
        "name": "Node",
        "type": "VECTOR",
        "visible": true,
        "commandsBlob": 0,
        "x": 10,
        "y": 20
    });

    substitute_blobs(&mut tree, &blobs).unwrap();

    // 其他字段应保留
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Node"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("VECTOR"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
    assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));

    // 命令Blob 应该被替换
    assert!(tree.get("commandsBlob").is_none());
    assert!(tree.get("commands").is_some());
}
