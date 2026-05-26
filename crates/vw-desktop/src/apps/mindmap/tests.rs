use super::MindMapMessage;

#[test]
fn mindmap_module_reexports_message_type() {
    assert!(format!("{:?}", MindMapMessage::New).contains("New"));
}
