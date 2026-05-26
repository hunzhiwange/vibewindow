use super::openai_oauth::{
    DeviceCodeResponse, build_authorize_url, exchange_code_for_tokens, extract_account_id_from_jwt,
    generate_pkce_state, parse_code_from_redirect, poll_device_code_tokens, receive_loopback_code,
    start_device_code_flow,
};

#[test]
fn placeholder_pkce_and_parsers_are_explicitly_empty() {
    let pkce = generate_pkce_state();

    assert!(pkce.verifier.is_empty());
    assert!(pkce.challenge.is_empty());
    assert!(pkce.state.is_empty());
    assert!(build_authorize_url(&pkce).is_empty());
    assert_eq!(extract_account_id_from_jwt("not-a-jwt"), None);
    assert_eq!(parse_code_from_redirect("http://localhost/callback?code=abc"), None);
}

#[tokio::test]
async fn unimplemented_oauth_network_flows_fail_explicitly() {
    let client = reqwest::Client::new();
    let device = DeviceCodeResponse {
        device_code: "device".to_string(),
        user_code: "user".to_string(),
        verification_uri: "https://example.test".to_string(),
        expires_in: 600,
        interval: 5,
    };
    let pkce = generate_pkce_state();

    assert!(
        start_device_code_flow(&client)
            .await
            .unwrap_err()
            .to_string()
            .contains("not fully implemented")
    );
    assert!(
        poll_device_code_tokens(&client, &device)
            .await
            .unwrap_err()
            .to_string()
            .contains("not fully implemented")
    );
    assert!(
        receive_loopback_code(0).await.unwrap_err().to_string().contains("not fully implemented")
    );
    assert!(
        exchange_code_for_tokens(&client, "code", &pkce)
            .await
            .unwrap_err()
            .to_string()
            .contains("not fully implemented")
    );
}
