use super::core::Agent;
use crate::app::agent::memory::NoneMemory;
use crate::app::agent::observability::NoopObserver;
use crate::app::agent::providers::{ChatMessage, Provider};
use async_trait::async_trait;
use std::sync::Arc;

struct CoreTestProvider;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Provider for CoreTestProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        _message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok("ok".to_string())
    }
}

fn build_agent() -> Agent {
    Agent::builder()
        .tools(Vec::new())
        .provider(Box::new(CoreTestProvider))
        .memory(Arc::new(NoneMemory::new()))
        .observer(Arc::new(NoopObserver))
        .build()
        .expect("core test agent should build")
}

#[test]
fn builder_returns_empty_builder_that_fails_without_required_parts() {
    let error = match Agent::builder().build() {
        Ok(_) => panic!("builder should reject missing required dependencies"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("tools are required"));
}

#[test]
fn history_returns_empty_slice_for_new_agent() {
    let agent = build_agent();

    assert!(agent.history().is_empty());
}

#[test]
fn history_returns_messages_in_insertion_order() {
    let mut agent = build_agent();

    agent.history.push(ChatMessage::system("system prompt"));
    agent.history.push(ChatMessage::user("hello"));
    agent.history.push(ChatMessage::assistant("hi"));

    let history = agent.history();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].role, "system");
    assert_eq!(history[0].content, "system prompt");
    assert_eq!(history[1].role, "user");
    assert_eq!(history[1].content, "hello");
    assert_eq!(history[2].role, "assistant");
    assert_eq!(history[2].content, "hi");
}

#[test]
fn clear_history_removes_all_messages() {
    let mut agent = build_agent();
    agent.history.push(ChatMessage::user("first"));
    agent.history.push(ChatMessage::assistant("second"));

    agent.clear_history();

    assert!(agent.history().is_empty());
}

#[test]
fn clear_history_is_idempotent_for_empty_history() {
    let mut agent = build_agent();

    agent.clear_history();
    agent.clear_history();

    assert!(agent.history().is_empty());
}

#[test]
fn from_config_builds_agent_with_none_memory_and_supported_provider() {
    let mut config = crate::app::agent::config::Config::default();
    config.memory.backend = "none".to_string();
    config.default_provider = Some("openai".to_string());
    config.default_model = Some("testcov-0076-model".to_string());
    config.default_temperature = 0.25;
    config.workspace_dir =
        std::env::temp_dir().join(format!("vibe-window-testcov-0076-{}", std::process::id()));

    let agent = Agent::from_config(&config).expect("supported provider should build");

    assert!(agent.history().is_empty());
    assert_eq!(agent.model_name, "testcov-0076-model");
    assert_eq!(agent.temperature, 0.25);
    assert_eq!(agent.workspace_dir, config.workspace_dir);
    assert_eq!(agent.memory.name(), "none");
}

#[test]
fn from_config_rejects_unknown_provider() {
    let mut config = crate::app::agent::config::Config::default();
    config.memory.backend = "none".to_string();
    config.default_provider = Some("testcov-0076-unknown-provider".to_string());
    config.workspace_dir =
        std::env::temp_dir().join(format!("vibe-window-testcov-0076-error-{}", std::process::id()));

    let error = match Agent::from_config(&config) {
        Ok(_) => panic!("unknown provider should be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("Unknown provider: testcov-0076-unknown-provider"));
}
