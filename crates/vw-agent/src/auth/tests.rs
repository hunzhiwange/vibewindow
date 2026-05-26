use super::AuthService;

#[test]
fn auth_service_new_preserves_state_dir_and_encryption_flag() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let service = AuthService::new(dir.path(), true);

    assert_eq!(service.state_dir.as_path(), dir.path());
    assert!(service.secrets_encrypt);
}

#[tokio::test]
async fn auth_service_token_getters_return_none_until_oauth_is_configured() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let service = AuthService::new(dir.path(), false);

    assert_eq!(service.get_valid_gemini_access_token(None).await.unwrap(), None);
    assert_eq!(service.get_valid_openai_access_token(None).await.unwrap(), None);
}
