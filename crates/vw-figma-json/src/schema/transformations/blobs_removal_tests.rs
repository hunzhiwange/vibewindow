use super::*;
use serde_json::json;

#[test]
fn test_remove_root_blobs() {
    let mut output = json!({
        "version": 48,
        "fileType": "figma",
        "document": {
            "name": "Root",
            "children": []
        },
        "blobs": [
            {"bytes": "SGVsbG8="},
            {"bytes": "V29ybGQ="}
        ]
    });

    remove_root_blobs(&mut output).unwrap();

    // Blobs 字段应该被删除
    assert!(output.get("blobs").is_none());

    // 其他字段应保留
    assert_eq!(output.get("version").unwrap().as_i64(), Some(48));
    assert_eq!(output.get("fileType").unwrap().as_str(), Some("figma"));
    assert!(output.get("document").is_some());
    assert_eq!(output.get("document").unwrap().get("name").unwrap().as_str(), Some("Root"));
}

#[test]
fn test_remove_root_blobs_already_missing() {
    let mut output = json!({
        "version": 101,
        "fileType": "figjam",
        "document": {
            "name": "Board"
        }
    });

    // 如果 blob 已经丢失，则不应失败
    remove_root_blobs(&mut output).unwrap();

    assert!(output.get("blobs").is_none());
    assert_eq!(output.get("version").unwrap().as_i64(), Some(101));
}

#[test]
fn test_remove_root_blobs_preserves_all_fields() {
    let mut output = json!({
        "version": 50,
        "fileType": "figma",
        "document": {
            "name": "Document",
            "children": [
                {"name": "Child1"},
                {"name": "Child2"}
            ]
        },
        "blobs": [],
        "metadata": {
            "custom": "field"
        }
    });

    remove_root_blobs(&mut output).unwrap();

    // 只应删除斑点
    assert!(output.get("blobs").is_none());

    // 保留所有其他字段
    assert_eq!(output.get("version").unwrap().as_i64(), Some(50));
    assert_eq!(output.get("fileType").unwrap().as_str(), Some("figma"));
    assert!(output.get("document").is_some());
    assert!(output.get("metadata").is_some());
    assert_eq!(output.get("metadata").unwrap().get("custom").unwrap().as_str(), Some("field"));
}

#[test]
fn test_remove_root_blobs_not_an_object() {
    let mut output = json!([1, 2, 3]);

    // 非对象输入不应失败
    remove_root_blobs(&mut output).unwrap();

    // 数组应保持不变
    assert_eq!(output.as_array().unwrap().len(), 3);
}
