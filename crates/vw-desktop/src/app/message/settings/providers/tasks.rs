//! 处理模型提供商设置子模块的目录、认证和连接状态。

use crate::app::Message;
use crate::app::config;
use crate::app::config::server_config_unreachable_error;
use crate::app::provider::provider as model_provider;
use crate::app::provider::provider_models;
use iced::Task;

use super::super::messages::SettingsMessage;
use super::helpers::{build_catalog_from_sources, load_popular_providers_from_config};

/// 处理 `refresh_task` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn refresh_task() -> Task<Message> {
    Task::perform(
        async move {
            model_provider::invalidate_cache().await;
            let providers = model_provider::list_for_settings().await;
            let raw_providers = provider_models::get().await;
            let cfg = config::load_full_agent_config_async()
                .await
                .map_err(server_config_unreachable_error)?;
            let (patterns, configured) = load_popular_providers_from_config(&cfg);
            let catalog = build_catalog_from_sources(&raw_providers, &providers);
            Ok((providers, patterns, configured, catalog))
        },
        |res| Message::Settings(SettingsMessage::ProvidersRefreshed(res)),
    )
}

#[cfg(test)]
#[path = "tasks_tests.rs"]
mod tasks_tests;

/// 处理 `sync_remote_task` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn sync_remote_task() -> Task<Message> {
    Task::perform(
        async move {
            provider_models::refresh().await;
            model_provider::invalidate_cache().await;
            let providers = model_provider::list_for_settings().await;
            let raw_providers = provider_models::get().await;
            let cfg = config::load_full_agent_config_async()
                .await
                .map_err(server_config_unreachable_error)?;
            let (patterns, configured) = load_popular_providers_from_config(&cfg);
            let catalog = build_catalog_from_sources(&raw_providers, &providers);
            Ok((providers, patterns, configured, catalog))
        },
        |res| Message::Settings(SettingsMessage::ProviderModelsSyncDone(res)),
    )
}

/// 处理 `load_catalog_task` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn load_catalog_task() -> Task<Message> {
    Task::perform(
        async move {
            model_provider::invalidate_cache().await;
            let providers = model_provider::list_for_settings().await;
            let raw_providers = provider_models::get().await;
            Ok(build_catalog_from_sources(&raw_providers, &providers))
        },
        |res| Message::Settings(SettingsMessage::ProviderCatalogLoaded(res)),
    )
}

/// 处理 `save_popular_providers_task` 对应的用户输入、异步结果或状态转换。
///
/// 参数来自已匹配的消息载荷或当前设置状态，函数只在当前消息边界内产生状态变更。
/// 返回的 `Task` 用于继续执行异步保存、加载或通知清理；没有后续动作时返回空任务。
pub fn save_popular_providers_task(list: Vec<String>) -> Task<Message> {
    Task::perform(
        async move {
            let mut root = serde_json::Map::new();
            root.insert(
                "popular_providers".to_string(),
                serde_json::Value::Array(list.into_iter().map(serde_json::Value::String).collect()),
            );
            config::patch_full_agent_config_async(serde_json::Value::Object(root))
                .await
                .map_err(server_config_unreachable_error)
        },
        |res| Message::Settings(SettingsMessage::PopularProvidersSaved(res)),
    )
}
