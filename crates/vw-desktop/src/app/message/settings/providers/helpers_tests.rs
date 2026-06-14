use super::*;
use std::collections::HashMap;
use vw_shared::provider::types::{
    ApiInfo, Capabilities, CapabilityIO, Info, InterleavedCapability, Model, ModelCost,
    ModelCostCache, ModelLimit, ProviderSource,
};

fn model(provider_id: &str, model_id: &str, name: &str) -> Model {
    Model {
        id: model_id.to_string(),
        provider_id: provider_id.to_string(),
        api: ApiInfo {
            id: model_id.to_string(),
            url: "https://example.test".to_string(),
            adapter: "openai-compatible".to_string(),
        },
        name: name.to_string(),
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
        limit: ModelLimit { context: 8_000, input: None, output: 1_000 },
        status: "stable".to_string(),
        options: HashMap::new(),
        headers: HashMap::new(),
        release_date: String::new(),
        variants: HashMap::new(),
    }
}

fn provider(id: &str, name: &str, source: ProviderSource, models: Vec<Model>) -> Info {
    Info {
        id: id.to_string(),
        name: name.to_string(),
        source,
        env: Vec::new(),
        key: None,
        options: HashMap::new(),
        models: models.into_iter().map(|m| (m.id.clone(), m)).collect(),
    }
}

#[test]
fn validates_ids_summarizes_and_builds_catalogs() {
    assert!(is_valid_provider_id("my-provider_2"));
    assert!(!is_valid_provider_id(""));
    assert!(!is_valid_provider_id("OpenAI"));
    assert!(!is_valid_provider_id("bad id"));

    let providers = HashMap::from([
        ("b".to_string(), provider("b", "Beta", ProviderSource::Env, Vec::new())),
        (
            "a".to_string(),
            provider("a", "Alpha", ProviderSource::Api, vec![model("a", "m1", "Model A")]),
        ),
        ("c".to_string(), provider("c", "Custom", ProviderSource::Custom, Vec::new())),
    ]);
    let summaries = summarize_providers(providers.clone());
    assert_eq!(summaries.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
    assert!(summaries[0].connected);
    assert!(!summaries[2].connected);

    let catalog = build_catalog_from_provider_infos(&providers);
    assert!(catalog.iter().any(|m| m.provider_id == "a" && m.model_id == "m1"));

    let mut raw_provider = raw_model_provider::Provider {
        api: None,
        name: "Raw".to_string(),
        env: Vec::new(),
        id: "raw".to_string(),
        adapter: None,
        models: HashMap::new(),
    };
    raw_provider.models.insert(
        "raw-model".to_string(),
        raw_model_provider::Model {
            id: "raw-model".to_string(),
            name: "Raw Model".to_string(),
            ..serde_json::from_value(serde_json::json!({})).unwrap()
        },
    );
    let merged =
        build_catalog_from_sources(&HashMap::from([("raw".to_string(), raw_provider)]), &providers);
    assert!(merged.iter().any(|m| m.model_id == "raw-model"));
    assert!(merged.iter().any(|m| m.model_id == "m1"));
}
