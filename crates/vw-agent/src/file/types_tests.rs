use super::types::{Content, ContentType, Error, Info, Node, NodeType, SearchInput, Status};

#[test]
fn file_types_serialize_with_expected_names() {
    let info = Info { path: "a.txt".to_string(), added: 1, removed: 0, status: Status::Added };
    let node = Node {
        name: "src".to_string(),
        path: "src".to_string(),
        absolute: "/tmp/src".to_string(),
        r#type: NodeType::Directory,
        ignored: false,
    };
    let content = Content {
        r#type: ContentType::Text,
        content: "hello".to_string(),
        diff: None,
        encoding: None,
        mime_type: None,
    };

    assert!(serde_json::to_value(info).expect("serialize").to_string().contains("added"));
    assert_eq!(serde_json::to_value(node).expect("serialize")["type"], "directory");
    assert_eq!(serde_json::to_value(content).expect("serialize")["type"], "text");
    assert!(Error::AccessDenied("/root".to_string()).to_string().contains("/root"));
    assert_eq!(SearchInput { query: "q".to_string(), limit: 1, dirs: false, r#type: None }.limit, 1);
}
