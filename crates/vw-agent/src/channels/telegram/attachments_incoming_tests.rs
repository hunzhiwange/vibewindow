use super::attachments::IncomingAttachmentKind;
use super::TelegramChannel;

#[test]
fn parse_attachment_metadata_selects_largest_photo_variant() {
    let message = serde_json::json!({
        "photo": [
            {"file_id": "small", "file_size": 10},
            {"file_id": "large", "file_size": 20}
        ],
        "caption": "hello"
    });

    let attachment = TelegramChannel::parse_attachment_metadata(&message).unwrap();

    assert_eq!(attachment.kind, IncomingAttachmentKind::Photo);
    assert_eq!(attachment.file_id, "large");
    assert_eq!(attachment.caption.as_deref(), Some("hello"));
}
