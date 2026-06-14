use super::*;

#[test]
fn method_serializes_with_lowercase_type_tag() {
    let value = serde_json::to_value(Method::Api { label: "API Key".into() }).unwrap();
    assert_eq!(value["type"], "api");
    assert_eq!(value["label"], "API Key");
}

#[test]
fn auth_error_display_is_stable() {
    assert_eq!(Error::MissingCode.to_string(), "missing code");
    assert_eq!(Error::Unsupported.to_string(), "unsupported auth method");
}

#[test]
fn authorization_and_method_round_trip_json() {
    let auth = Authorization {
        url: "https://example.test/oauth".to_string(),
        method: "oauth".to_string(),
        instructions: "Open the URL".to_string(),
    };
    let value = serde_json::to_value(&auth).unwrap();
    assert_eq!(value["url"], "https://example.test/oauth");

    let method: Method =
        serde_json::from_value(serde_json::json!({"type": "oauth", "label": "OAuth"})).unwrap();
    assert!(matches!(method, Method::Oauth { label } if label == "OAuth"));
}

#[tokio::test]
async fn auth_stubs_return_documented_values() {
    assert!(methods().await.is_empty());
    assert!(authorize("provider", 0).await.unwrap().is_none());
    assert!(matches!(callback("provider", 0, Some("code")).await, Err(Error::Unsupported)));
}

#[tokio::test]
async fn api_stores_key_in_auth_store() {
    let provider_id = format!("provider-auth-test-{}", uuid::Uuid::new_v4());
    api(&provider_id, "secret-key").await.expect("api key should store");

    let stored = crate::app::agent::auth::get(&provider_id).expect("auth should be stored");
    match stored {
        crate::app::agent::auth::Info::Api(info) => assert_eq!(info.key, "secret-key"),
        other => panic!("unexpected auth info: {other:?}"),
    }

    crate::app::agent::auth::remove(&provider_id).expect("auth cleanup should succeed");
}

#[test]
fn auth_error_from_io_and_display_variants() {
    let err: Error = std::io::Error::new(std::io::ErrorKind::Other, "disk full").into();
    assert_eq!(err.to_string(), "disk full");
    assert_eq!(Error::MissingOauth.to_string(), "missing oauth authorization");
    assert_eq!(Error::CallbackFailed.to_string(), "oauth callback failed");
}
