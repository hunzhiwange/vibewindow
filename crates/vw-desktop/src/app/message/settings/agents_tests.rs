//! 处理系统设置页面中对应功能区的消息、校验和配置持久化。

use super::{bundled_workspace_identity_content, bundled_workspace_identity_path};
use super::summarize_models;
use std::collections::HashMap;
use vw_shared::provider::types::{
    ApiInfo, Capabilities, CapabilityIO, Info, InterleavedCapability, Model, ModelCost,
    ModelCostCache, ModelLimit, ProviderSource,
};

#[test]
fn bundled_workspace_identity_path_points_to_assets_agent() {
    assert_eq!(bundled_workspace_identity_path("SOUL.md"), "assets/agent/SOUL.md");
    assert_eq!(bundled_workspace_identity_path("MEMORY.md"), "assets/agent/MEMORY.md");
}

#[test]
fn bundled_workspace_identity_content_is_embedded() {
    let soul = bundled_workspace_identity_content("SOUL.md").expect("missing embedded SOUL.md");
    let agents =
        bundled_workspace_identity_content("AGENTS.md").expect("missing embedded AGENTS.md");

    assert!(!soul.trim().is_empty());
    assert!(!agents.trim().is_empty());
    assert!(bundled_workspace_identity_content("UNKNOWN.md").is_none());
}

#[test]
fn summarize_models_keeps_builtin_provider_models_even_when_all_disabled() {
    let model = Model {
        id: "gpt-4.1".to_string(),
        provider_id: "openai".to_string(),
        api: ApiInfo {
            id: "gpt-4.1".to_string(),
            url: "https://api.example.com/v1".to_string(),
            adapter: "openai-compatible".to_string(),
        },
        name: "GPT-4.1".to_string(),
        family: None,
        capabilities: Capabilities {
            temperature: true,
            reasoning: false,
            attachment: false,
            toolcall: true,
            input: CapabilityIO {
                text: true,
                audio: false,
                image: false,
                video: false,
                pdf: false,
            },
            output: CapabilityIO {
                text: true,
                audio: false,
                image: false,
                video: false,
                pdf: false,
            },
            interleaved: InterleavedCapability::Bool(false),
        },
        cost: ModelCost {
            input: 0.0,
            output: 0.0,
            cache: ModelCostCache { read: 0.0, write: 0.0 },
            experimental_over_200k: None,
        },
        limit: ModelLimit { context: 128_000, input: None, output: 16_384 },
        status: "disabled".to_string(),
        options: HashMap::new(),
        headers: HashMap::new(),
        release_date: String::new(),
        variants: HashMap::new(),
    };

    let mut models = HashMap::new();
    models.insert(model.id.clone(), model);

    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        Info {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            source: ProviderSource::Custom,
            env: vec!["OPENAI_API_KEY".to_string()],
            key: None,
            options: HashMap::new(),
            models,
        },
    );

    let summary = summarize_models(providers);

    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].id, "openai");
    assert_eq!(summary[0].models.len(), 1);
    assert_eq!(summary[0].models[0].id, "gpt-4.1");
    assert!(!summary[0].models[0].enabled);
}