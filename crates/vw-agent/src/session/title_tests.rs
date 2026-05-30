use super::*;

#[tokio::test]
async fn empty_content_is_rejected_before_provider_lookup() {
    let err =
        generate_from_content("session-title-test".to_string(), "   \n\t".to_string(), None, None)
            .await
            .expect_err("blank content must be rejected locally");

    assert_eq!(err, "Empty title source");
}
