use super::Severity;
use super::config_checks::{
    check_config_semantics, embedding_provider_validation_error, provider_validation_error,
};
use crate::app::agent::config::{Config, EmbeddingRouteConfig, ModelRouteConfig};

#[test]
fn provider_validation_errors_include_provider_name() {
    let err = provider_validation_error("vibewindow-provider-that-should-not-exist")
        .expect("invalid provider should produce an error");
    let embedding_err = embedding_provider_validation_error("voyage")
        .expect("unsupported embedding provider should produce an error");

    assert!(!err.is_empty());
    assert!(embedding_err.contains("supported values"));
}

#[test]
fn alibaba_embedding_providers_are_valid() {
    assert!(embedding_provider_validation_error("alibaba").is_none());
    assert!(embedding_provider_validation_error("alibaba-cn").is_none());
}

#[test]
fn custom_embedding_provider_validation_checks_url_shape() {
    assert!(embedding_provider_validation_error("custom:https://embeddings.example.com").is_none());
    assert!(embedding_provider_validation_error("custom:http://localhost:8080").is_none());

    let empty = embedding_provider_validation_error("custom:").unwrap();
    assert!(empty.contains("requires a non-empty URL"));

    let scheme = embedding_provider_validation_error("custom:file:///tmp/embed").unwrap();
    assert!(scheme.contains("must use http/https"));

    let invalid = embedding_provider_validation_error("custom:not a url").unwrap();
    assert!(invalid.contains("invalid custom provider URL"));
}

#[test]
fn ollama_provider_does_not_warn_about_missing_api_key() {
    let mut config = Config::default();
    config.default_provider = Some("ollama".into());
    config.api_key = None;
    let mut items = Vec::new();

    check_config_semantics(&config, &mut items);

    assert!(!items.iter().any(|item| item.message.contains("no api_key set")));
}

#[test]
fn config_semantics_reports_model_route_hint_provider_and_model_issues() {
    let mut config = Config::default();
    config.model_routes = vec![ModelRouteConfig {
        hint: String::new(),
        provider: "provider-that-does-not-exist".into(),
        model: String::new(),
        max_tokens: None,
        api_key: None,
    }];
    let mut items = Vec::new();

    check_config_semantics(&config, &mut items);

    assert!(items.iter().any(
        |item| item.severity == Severity::Warn && item.message == "model route with empty hint"
    ));
    assert!(
        items.iter().any(|item| item.severity == Severity::Warn
            && item.message.contains("uses invalid provider"))
    );
    assert!(
        items
            .iter()
            .any(|item| item.severity == Severity::Warn && item.message.contains("has empty model"))
    );
}

#[test]
fn config_semantics_reports_embedding_route_dimensions_and_missing_hint() {
    let mut config = Config::default();
    config.embedding_routes = vec![EmbeddingRouteConfig {
        hint: "semantic".into(),
        provider: "openai".into(),
        model: "text-embedding-3-small".into(),
        dimensions: Some(0),
        api_key: None,
    }];
    config.memory.embedding_model = "hint:missing".into();
    let mut items = Vec::new();

    check_config_semantics(&config, &mut items);

    assert!(items.iter().any(|item| item.message.contains("invalid dimensions=0")));
    assert!(
        items
            .iter()
            .any(|item| item.message.contains("memory.embedding_model uses hint \"missing\""))
    );
}

#[test]
fn config_semantics_accepts_matching_embedding_hint() {
    let mut config = Config::default();
    config.embedding_routes = vec![EmbeddingRouteConfig {
        hint: "semantic".into(),
        provider: "openai".into(),
        model: "text-embedding-3-small".into(),
        dimensions: Some(1536),
        api_key: None,
    }];
    config.memory.embedding_model = "hint:semantic".into();
    let mut items = Vec::new();

    check_config_semantics(&config, &mut items);

    assert!(
        !items
            .iter()
            .any(|item| item.message.contains("no matching [[embedding_routes]] entry exists"))
    );
}
