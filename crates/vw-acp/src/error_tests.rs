use super::error::AcpError;

#[test]
fn display_messages_keep_contextual_prefixes() {
    assert_eq!(AcpError::EmptyCommand.to_string(), "acp command is empty");
    assert_eq!(
        AcpError::Initialize("bad handshake".to_string()).to_string(),
        "acp initialize failed: bad handshake"
    );
    assert_eq!(
        AcpError::SessionChanged {
            expected: "expected-session".to_string(),
            actual: "actual-session".to_string(),
        }
        .to_string(),
        "acp session changed: expected=expected-session actual=actual-session"
    );
}
