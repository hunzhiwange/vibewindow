use super::*;

#[tokio::test]
async fn empty_content_is_rejected_before_provider_lookup() {
    let err =
        generate_from_content("session-title-test".to_string(), "   \n\t".to_string(), None, None)
            .await
            .expect_err("blank content must be rejected locally");

    assert_eq!(err, "Empty title source");
}

#[tokio::test]
async fn whitespace_after_truncation_is_rejected_before_agent_lookup() {
    let err = generate_from_content(
        "session-title-long-blank-test".to_string(),
        " ".repeat(CONTENT_TRUNCATE_CHARS + 5),
        Some("openai/gpt-4".to_string()),
        Some("fixture-agent".to_string()),
    )
    .await
    .expect_err("blank truncated content must be rejected locally");

    assert_eq!(err, "Empty title source");
    assert_eq!(MAX_TITLE_CHARS, TITLE_TRUNCATE_KEEP + 3);
}
