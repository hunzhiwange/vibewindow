use std::collections::HashMap;

use serde_json::{Value, json};

use crate::app::agent::provider::provider;
use crate::app::agent::session::llm::types::Error;
use crate::app::agent::session::message::AssistantError;
use crate::app::agent::{auth, env, tools};

use super::{bearer_fingerprint, do_stream_request_aisdk, redact_bearer_for_log};

const API_KEY_EMPTY: &str = "API key 为空";
const OAUTH_ACCESS_TOKEN_EMPTY: &str = "OAuth access token 为空";

fn test_model(api_url: &str) -> provider::Model {
    serde_json::from_value(json!({
        "id": "gpt-test",
        "providerID": "openai",
        "api": {
            "id": "gpt-test",
            "url": api_url,
            "adapter": "openai-compatible"
        },
        "name": "GPT Test",
        "family": null,
        "capabilities": {
            "temperature": true,
            "reasoning": true,
            "attachment": false,
            "toolcall": true,
            "input": {
                "text": true,
                "audio": false,
                "image": false,
                "video": false,
                "pdf": false
            },
            "output": {
                "text": true,
                "audio": false,
                "image": false,
                "video": false,
                "pdf": false
            },
            "interleaved": false
        },
        "cost": {
            "input": 0.0,
            "output": 0.0,
            "cache": {
                "read": 0.0,
                "write": 0.0
            },
            "experimental_over_200k": null
        },
        "limit": {
            "context": 8192,
            "input": null,
            "output": 4096
        },
        "status": "active",
        "options": {},
        "headers": {},
        "release_date": "2026-01-01",
        "variants": {}
    }))
    .expect("test model should deserialize")
}

fn test_provider(key: Option<&str>, env_keys: Vec<&str>) -> provider::Info {
    provider::Info {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        source: provider::ProviderSource::Custom,
        env: env_keys.into_iter().map(ToString::to_string).collect(),
        key: key.map(ToString::to_string),
        options: HashMap::new(),
        models: HashMap::new(),
    }
}

fn user_messages() -> Value {
    json!([{ "role": "user", "content": "hello" }])
}

fn empty_tools() -> HashMap<String, tools::ToolSpec> {
    HashMap::new()
}

async fn call_request(
    provider_info: &provider::Info,
    auth_info: Option<&auth::Info>,
    headers: &HashMap<String, String>,
    merged_options: &Value,
) -> Result<(), Error> {
    let model = test_model("not a url");
    let messages = user_messages();
    let tools = empty_tools();
    let mut events = Vec::new();

    do_stream_request_aisdk(
        provider_info,
        auth_info,
        headers,
        merged_options,
        &model,
        &messages,
        &tools,
        Some(0.7),
        Some(0.9),
        Some(1234),
        2,
        None,
        &mut |event| events.push(event),
    )
    .await
}

fn assert_provider_auth_error(result: Result<(), Error>, expected_message: &str) {
    match result {
        Err(Error::Api(AssistantError::ProviderAuthError { provider_id, message })) => {
            assert_eq!(provider_id, "openai");
            assert_eq!(message, expected_message);
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

fn assert_invalid_base_url(result: Result<(), Error>) {
    match result {
        Err(Error::Api(AssistantError::Unknown { message })) => {
            assert!(message.contains("Invalid base URL"), "message was {message:?}");
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[test]
fn redact_bearer_handles_empty_short_and_long_values() {
    assert_eq!(redact_bearer_for_log(""), "*");
    assert_eq!(redact_bearer_for_log("a"), "*");
    assert_eq!(redact_bearer_for_log("abcd"), "****");
    assert_eq!(redact_bearer_for_log("abcde"), "ab***de");
    assert_eq!(redact_bearer_for_log("token-value"), "to***ue");
}

#[test]
fn bearer_fingerprint_uses_first_twelve_hex_chars() {
    assert_eq!(bearer_fingerprint("secret"), "2bb80d537b1d");
}

#[tokio::test]
async fn custom_headers_are_rejected_before_auth_or_dispatch() {
    let mut headers = HashMap::new();
    headers.insert("x-extra".to_string(), "1".to_string());

    let result = call_request(&test_provider(None, vec![]), None, &headers, &json!({})).await;

    match result {
        Err(Error::Api(AssistantError::Unknown { message })) => {
            assert!(message.contains("headers"));
        }
        other => panic!("unexpected result: {other:?}"),
    }
}

#[tokio::test]
async fn missing_auth_defaults_to_api_key_empty_error() {
    let headers = HashMap::new();

    let result = call_request(&test_provider(None, vec![]), None, &headers, &json!({})).await;

    assert_provider_auth_error(result, API_KEY_EMPTY);
}

#[tokio::test]
async fn empty_api_and_oauth_auth_report_specific_messages() {
    let headers = HashMap::new();
    let provider = test_provider(None, vec![]);

    let api = auth::Info::Api(auth::ApiInfo { key: "  ".to_string() });
    let oauth = auth::Info::Oauth(auth::OauthInfo {
        refresh: "refresh".to_string(),
        access: "  ".to_string(),
        expires: 0,
        account_id: None,
        enterprise_url: None,
    });

    assert_provider_auth_error(
        call_request(&provider, Some(&api), &headers, &json!({})).await,
        API_KEY_EMPTY,
    );
    assert_provider_auth_error(
        call_request(&provider, Some(&oauth), &headers, &json!({})).await,
        OAUTH_ACCESS_TOKEN_EMPTY,
    );
}

#[tokio::test]
async fn provider_key_takes_priority_over_empty_auth_info() {
    let headers = HashMap::from([("User-Agent".to_string(), "vw-test".to_string())]);
    let provider = test_provider(Some(" provider-token "), vec![]);
    let auth = auth::Info::Api(auth::ApiInfo { key: "  ".to_string() });

    let result =
        call_request(&provider, Some(&auth), &headers, &json!({ "z": true, "a": 1 })).await;

    assert_invalid_base_url(result);
}

#[tokio::test]
async fn provider_env_token_is_used_before_auth_info() {
    let key = "VW_AGENT_AISDK_REQUEST_TEST_TOKEN";
    env::set(key, " env-token ");

    let headers = HashMap::new();
    let provider = test_provider(None, vec![key]);
    let auth = auth::Info::Api(auth::ApiInfo { key: "auth-token".to_string() });
    let result = call_request(&provider, Some(&auth), &headers, &json!(null)).await;

    env::remove(key);

    assert_invalid_base_url(result);
}

#[tokio::test]
async fn api_and_oauth_auth_fallbacks_reach_dispatch_when_non_empty() {
    let headers = HashMap::new();
    let provider = test_provider(None, vec![]);
    let api = auth::Info::Api(auth::ApiInfo { key: " api-token ".to_string() });
    let oauth = auth::Info::Oauth(auth::OauthInfo {
        refresh: "refresh".to_string(),
        access: " oauth-token ".to_string(),
        expires: 0,
        account_id: Some("account".to_string()),
        enterprise_url: None,
    });

    assert_invalid_base_url(call_request(&provider, Some(&api), &headers, &json!({})).await);
    assert_invalid_base_url(call_request(&provider, Some(&oauth), &headers, &json!({})).await);
}
