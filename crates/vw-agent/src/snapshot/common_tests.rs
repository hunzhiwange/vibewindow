use super::*;

#[test]
fn patch_serializes_hash_and_files() {
    let patch = Patch { hash: "abc123".to_string(), files: vec!["/tmp/file.rs".to_string()] };

    let value = serde_json::to_value(&patch).unwrap();
    assert_eq!(value["hash"], "abc123");
    assert_eq!(value["files"][0], "/tmp/file.rs");
}
