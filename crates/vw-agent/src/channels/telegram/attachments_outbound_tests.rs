use super::attachments::TelegramAttachment;

#[test]
fn telegram_attachment_preserves_target_verbatim() {
    let attachment = TelegramAttachment {
        kind: super::attachments::TelegramAttachmentKind::Image,
        target: "https://example.test/a b.png".to_string(),
    };

    assert_eq!(attachment.target, "https://example.test/a b.png");
}
