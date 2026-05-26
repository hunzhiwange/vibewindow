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

