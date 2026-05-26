use super::types::MindMapMessage;

#[test]
fn mindmap_message_debug_includes_variant_name() {
    assert!(format!("{:?}", MindMapMessage::New).contains("New"));
}
