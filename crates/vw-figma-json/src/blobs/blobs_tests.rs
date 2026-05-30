use super::*;
use serde_json::json;

#[test]
fn test_process_blobs_with_bytes() {
    let blobs = vec![json!({
        "id": 1,
        "bytes": [72, 101, 108, 108, 111]  // "Hello" in ASCII
    })];

    let processed = process_blobs(blobs).unwrap();
    let blobs_array = processed.as_array().unwrap();

    assert_eq!(blobs_array.len(), 1);
    let blob = &blobs_array[0];

    // 检查 bytes 字段现在是否为 base64 字符串
    let bytes_value = blob.get("bytes").unwrap();
    assert!(bytes_value.is_string());
    assert_eq!(bytes_value.as_str().unwrap(), "SGVsbG8="); // "Hello" in base64
}

#[test]
fn test_process_blobs_without_bytes() {
    let blobs = vec![json!({
        "id": 1,
        "type": "IMAGE"
    })];

    let processed = process_blobs(blobs).unwrap();
    let blobs_array = processed.as_array().unwrap();

    assert_eq!(blobs_array.len(), 1);
    assert_eq!(blobs_array[0].get("id").unwrap(), 1);
}

#[test]
fn test_process_empty_blobs() {
    let blobs = vec![];
    let processed = process_blobs(blobs).unwrap();
    let blobs_array = processed.as_array().unwrap();
    assert_eq!(blobs_array.len(), 0);
}

#[test]
fn test_process_multiple_blobs() {
    let blobs = vec![
        json!({
            "id": 1,
            "bytes": [65, 66, 67]  // "ABC"
        }),
        json!({
            "id": 2,
            "bytes": [88, 89, 90]  // "XYZ"
        }),
    ];

    let processed = process_blobs(blobs).unwrap();
    let blobs_array = processed.as_array().unwrap();

    assert_eq!(blobs_array.len(), 2);
    assert_eq!(blobs_array[0].get("bytes").unwrap().as_str().unwrap(), "QUJD"); // "ABC" in base64
    assert_eq!(blobs_array[1].get("bytes").unwrap().as_str().unwrap(), "WFla"); // "XYZ" in base64
}
