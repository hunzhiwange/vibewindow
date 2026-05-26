use super::*;

#[test]
fn enums_use_stable_wire_names() {
    assert_eq!(serde_json::to_value(OutputFormat::Quiet).unwrap(), "quiet");
    assert_eq!(serde_json::to_value(PermissionMode::ApproveReads).unwrap(), "approve-reads");
    assert_eq!(
        serde_json::to_value(OutputErrorCode::PermissionDenied).unwrap(),
        "PERMISSION_DENIED"
    );
    assert_eq!(
        serde_json::from_value::<OutputStream>(serde_json::json!("prompt")).unwrap(),
        OutputStream::Prompt
    );
}
