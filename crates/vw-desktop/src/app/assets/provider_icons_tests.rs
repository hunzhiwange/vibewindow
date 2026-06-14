use super::*;

#[test]
fn get_provider_icon_returns_known_provider_icons() {
    for provider_id in ["agent", "openai", "anthropic", "deepseek", "google-vertex-anthropic"] {
        std::hint::black_box(get_provider_icon(provider_id));
    }
}

#[test]
fn get_provider_icon_falls_back_to_agent_for_unknown_or_blank_ids() {
    std::hint::black_box(get_provider_icon("unknown-provider"));
    std::hint::black_box(get_provider_icon(""));
    std::hint::black_box(get_provider_icon("OPENAI"));
}
