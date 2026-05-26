use super::models::{self, Model as RawModel, Provider as RawProvider};
use super::types::*;
use serde_json::Value;
use std::collections::HashMap;

/// provider 运行时状态快照。
#[derive(Debug, Default)]
pub struct State {
    pub providers: HashMap<String, Info>,
}

/// 归一化适配器名称，兼容历史别名。
pub fn normalize_adapter(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return default_adapter();
    }
    let lowered = trimmed.to_ascii_lowercase();
    match lowered.as_str() {
        "acp" | "agent-client-protocol" | "agent_client_protocol" => {
            "openai-compatible".to_string()
        }
        _ => trimmed.to_string(),
    }
}

/// 将原始模型定义转换为对外暴露的模型结构。
pub fn from_models_dev_model(provider: &RawProvider, model: &RawModel) -> Model {
    let adapter = model
        .provider
        .as_ref()
        .map(|p| p.adapter.clone())
        .or_else(|| provider.adapter.clone())
        .unwrap_or_else(default_adapter);

    let api = ApiInfo {
        id: model.id.clone(),
        url: provider.api.clone().unwrap_or_default(),
        adapter: normalize_adapter(&adapter),
    };

    let cost = ModelCost {
        input: model.cost.as_ref().map(|c| c.input).unwrap_or(0.0),
        output: model.cost.as_ref().map(|c| c.output).unwrap_or(0.0),
        cache: ModelCostCache {
            read: model.cost.as_ref().and_then(|c| c.cache_read).unwrap_or(0.0),
            write: model.cost.as_ref().and_then(|c| c.cache_write).unwrap_or(0.0),
        },
        experimental_over_200k: model.cost.as_ref().and_then(|c| c.context_over_200k.as_ref()).map(
            |x| ModelCostOver200k {
                input: x.input,
                output: x.output,
                cache: ModelCostCache {
                    read: x.cache_read.unwrap_or(0.0),
                    write: x.cache_write.unwrap_or(0.0),
                },
            },
        ),
    };

    let modalities = model.modalities.as_ref();
    let input = CapabilityIO {
        text: modalities.is_some_and(|m| m.input.iter().any(|s| s == "text")),
        audio: modalities.is_some_and(|m| m.input.iter().any(|s| s == "audio")),
        image: modalities.is_some_and(|m| m.input.iter().any(|s| s == "image")),
        video: modalities.is_some_and(|m| m.input.iter().any(|s| s == "video")),
        pdf: modalities.is_some_and(|m| m.input.iter().any(|s| s == "pdf")),
    };
    let output = CapabilityIO {
        text: modalities.is_some_and(|m| m.output.iter().any(|s| s == "text")),
        audio: modalities.is_some_and(|m| m.output.iter().any(|s| s == "audio")),
        image: modalities.is_some_and(|m| m.output.iter().any(|s| s == "image")),
        video: modalities.is_some_and(|m| m.output.iter().any(|s| s == "video")),
        pdf: modalities.is_some_and(|m| m.output.iter().any(|s| s == "pdf")),
    };

    let interleaved =
        match model.interleaved.clone().unwrap_or(models::ModelInterleaved::Bool(false)) {
            models::ModelInterleaved::Bool(b) => InterleavedCapability::Bool(b),
            models::ModelInterleaved::Field { field } => InterleavedCapability::Field { field },
        };

    Model {
        id: model.id.clone(),
        provider_id: provider.id.clone(),
        api,
        name: model.name.clone(),
        family: model.family.clone(),
        capabilities: Capabilities {
            temperature: model.temperature,
            reasoning: model.reasoning,
            attachment: model.attachment,
            toolcall: model.tool_call,
            input,
            output,
            interleaved,
        },
        cost,
        limit: ModelLimit {
            context: model.limit.context,
            input: model.limit.input,
            output: model.limit.output,
        },
        status: model.status.clone().unwrap_or_else(|| "active".to_string()),
        headers: model.headers.clone().unwrap_or_default(),
        options: model.options.clone(),
        release_date: model.release_date.clone(),
        variants: model.variants.clone().unwrap_or_default(),
    }
}

/// 将原始 provider 定义转换为对外暴露的 provider 结构。
pub fn from_models_dev_provider(provider: RawProvider) -> Info {
    let models = provider
        .models
        .values()
        .map(|m| (m.id.clone(), from_models_dev_model(&provider, m)))
        .collect::<HashMap<_, _>>();

    Info {
        id: provider.id.clone(),
        source: ProviderSource::Custom,
        name: provider.name.clone(),
        env: provider.env.clone(),
        key: None,
        options: HashMap::new(),
        models,
    }
}

/// 从 JSON 值中提取字符串键值对映射。
pub fn as_string_map(v: &Value) -> HashMap<String, String> {
    v.as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default()
}

/// 预留的内建模型限制查询入口，当前始终返回空。
pub fn builtin_model_limit(_provider_id: &str, _model_id: &str) -> Option<(u64, u64, Option<u64>)> {
    None
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod state_tests;
