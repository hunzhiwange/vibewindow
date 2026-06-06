use std::collections::HashMap;

use serde_json::json;

use crate::app::agent::provider::provider;
use crate::app::agent::{auth, config};

use super::with_copilot_auth_environment;

fn test_model(provider_id: &str) -> provider::Model {
    serde_json::from_value(json!({
        "id": "gpt-5-mini",
        "providerID": provider_id,
        "api": {
            "id": "gpt-5-mini",
            "url": "http://localhost",
            "adapter": "acp"
        },
        "name": "gpt-5-mini",
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

fn test_provider(key: Option<&str>) -> provider::Info {
    provider::Info {
        id: "github-copilot".to_string(),
        name: "GitHub Copilot".to_string(),
        source: provider::ProviderSource::Api,
        env: Vec::new(),
        key: key.map(ToString::to_string),
        options: HashMap::new(),
        models: HashMap::new(),
    }
}

fn copilot_config() -> config::schema::AcpAgentConfig {
    config::schema::AcpAgentConfig {
        command: "copilot".to_string(),
        args: vec!["--acp".to_string(), "--stdio".to_string()],
        env: HashMap::new(),
    }
}

#[test]
fn copilot_auth_environment_uses_provider_key() {
    let cfg = with_copilot_auth_environment(
        &test_model("github-copilot"),
        &test_provider(Some(" provider-token ")),
        None,
        "copilot",
        copilot_config(),
    );

    assert_eq!(cfg.env.get("GH_COPILOT_TOKEN").map(String::as_str), Some("provider-token"));
    assert_eq!(cfg.env.get("GITHUB_COPILOT_TOKEN").map(String::as_str), Some("provider-token"));
}

#[test]
fn copilot_auth_environment_preserves_explicit_agent_env() {
    let mut cfg = copilot_config();
    cfg.env.insert("GH_COPILOT_TOKEN".to_string(), "explicit-token".to_string());

    let cfg = with_copilot_auth_environment(
        &test_model("github-copilot"),
        &test_provider(Some("provider-token")),
        None,
        "copilot",
        cfg,
    );

    assert_eq!(cfg.env.get("GH_COPILOT_TOKEN").map(String::as_str), Some("explicit-token"));
    assert_eq!(cfg.env.get("GITHUB_COPILOT_TOKEN").map(String::as_str), Some("provider-token"));
}

#[test]
fn copilot_auth_environment_uses_api_auth_fallback() {
    let cfg = with_copilot_auth_environment(
        &test_model("github-copilot"),
        &test_provider(None),
        Some(&auth::Info::Api(auth::ApiInfo { key: " auth-token ".to_string() })),
        "github-copilot",
        copilot_config(),
    );

    assert_eq!(cfg.env.get("GH_COPILOT_TOKEN").map(String::as_str), Some("auth-token"));
}

#[test]
fn copilot_auth_environment_ignores_oauth_dummy_key() {
    let cfg = with_copilot_auth_environment(
        &test_model("github-copilot"),
        &test_provider(Some(auth::OAUTH_DUMMY_KEY)),
        None,
        "copilot",
        copilot_config(),
    );

    assert!(cfg.env.is_empty());
}

#[test]
fn copilot_auth_environment_ignores_non_copilot_provider() {
    let cfg = with_copilot_auth_environment(
        &test_model("openai"),
        &test_provider(Some("provider-token")),
        None,
        "copilot",
        copilot_config(),
    );

    assert!(cfg.env.is_empty());
}
