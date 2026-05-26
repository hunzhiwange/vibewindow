use super::attachments::{TelegramAttachment, TelegramAttachmentKind};

#[test]
fn outbound_attachment_keeps_kind_and_target() {
    let attachment = TelegramAttachment {
        kind: TelegramAttachmentKind::Document,
        target: "/tmp/report.pdf".to_string(),
    };

    assert_eq!(attachment.kind, TelegramAttachmentKind::Document);
    assert_eq!(attachment.target, "/tmp/report.pdf");
}
