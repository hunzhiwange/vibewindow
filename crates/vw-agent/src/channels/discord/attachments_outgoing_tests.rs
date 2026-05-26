use std::fs;

use super::attachments_outgoing::{
    DiscordAttachment, DiscordAttachmentKind, classify_outgoing_attachments,
    parse_attachment_markers, resolve_local_attachment_path, with_inline_attachment_urls,
};

#[test]
fn attachment_marker_parsing_removes_valid_markers_and_keeps_invalid_text() {
    let (text, attachments) =
        parse_attachment_markers("hello [IMAGE:/workspace/a.png] [UNKNOWN:x] [FILE: report.pdf ]");

    assert_eq!(text, "hello  [UNKNOWN:x]");
    assert_eq!(attachments.len(), 2);
    assert_eq!(attachments[0].kind, DiscordAttachmentKind::Image);
    assert_eq!(attachments[1].kind.marker_name(), "DOCUMENT");
}

#[test]
fn outgoing_attachments_are_classified_by_url_scheme() {
    let attachments = vec![
        DiscordAttachment {
            kind: DiscordAttachmentKind::Image,
            target: "https://example.test/a.png".to_string(),
        },
        DiscordAttachment {
            kind: DiscordAttachmentKind::Document,
            target: "local.txt".to_string(),
        },
    ];

    let (local, remote, unresolved) = classify_outgoing_attachments(&attachments);

    assert_eq!(local.len(), 1);
    assert_eq!(remote, vec!["https://example.test/a.png"]);
    assert!(unresolved.is_empty());
    assert_eq!(
        with_inline_attachment_urls("body", &remote, &[]),
        "body\nhttps://example.test/a.png"
    );
}

#[test]
fn resolve_local_attachment_path_rejects_missing_workspace_and_path_escape() {
    assert!(resolve_local_attachment_path(None, "a.txt").is_err());

    let workspace = tempfile::tempdir().expect("workspace should exist");
    let inside = workspace.path().join("a.txt");
    fs::write(&inside, "ok").expect("file should be written");
    let resolved = resolve_local_attachment_path(Some(&workspace.path().to_path_buf()), "a.txt")
        .expect("inside file should resolve");

    assert_eq!(resolved, inside.canonicalize().unwrap());
    assert!(
        resolve_local_attachment_path(Some(&workspace.path().to_path_buf()), "../a.txt").is_err()
    );
}
