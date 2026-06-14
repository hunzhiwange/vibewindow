use super::basic::view;
use crate::app::state::DelegateAgentSettingsEntry;

#[test]
fn basic_view_builds_for_main_and_worker_entries() {
    let (app, _) = crate::app::App::new();
    let main = app
        .agents_settings
        .entries
        .iter()
        .find(|entry| entry.key == "main")
        .expect("default settings should contain main agent");
    let worker = app
        .agents_settings
        .entries
        .iter()
        .find(|entry| entry.key != "main")
        .expect("default settings should contain a worker agent");

    let provider_options = vec!["openai".to_string(), main.provider.clone()];
    let _: iced::Element<'_, crate::app::Message> = view(&app, main, provider_options.clone());
    let _: iced::Element<'_, crate::app::Message> = view(&app, worker, provider_options);
}

#[test]
fn basic_view_keeps_current_provider_and_model_even_when_missing_from_catalog() {
    let (app, _) = crate::app::App::new();
    let mut entry: DelegateAgentSettingsEntry = app.agents_settings.entries[0].clone();
    entry.provider = "custom-provider".to_string();
    entry.model = "custom-model".to_string();

    let _: iced::Element<'_, crate::app::Message> = view(&app, &entry, vec!["openai".to_string()]);
}
