use super::*;
use iced::Element;
use crate::app::{App, Message, message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn field_and_text_rows_build_controls() {
    keep_element(field_row("标签", "说明", text("control")));
    keep_element(text_row("模型", "说明", "placeholder", "whisper", |value| {
        Message::Settings(message::SettingsMessage::TranscriptionModelChanged(value))
    }));
}

#[test]
fn view_builds_enabled_disabled_and_error_states() {
    let mut app = test_app();
    keep_element(view(&app));

    app.transcription_settings.enabled = true;
    app.transcription_settings.api_url = "https://transcribe.example/v1".to_string();
    app.transcription_settings.model = "whisper-large-v3".to_string();
    app.transcription_settings.language = "zh".to_string();
    app.transcription_settings.max_duration_secs = 3600;
    keep_element(view(&app));

    app.transcription_settings.max_duration_secs = 1;
    app.transcription_settings.save_error = Some("保存失败".to_string());
    keep_element(view(&app));
}

#[test]
fn overlays_return_dialog_or_help_modal() {
    let mut app = test_app();
    keep_element(view_overlays(&app, text("dialog").into()));

    app.transcription_settings.show_help_modal = true;
    keep_element(view_overlays(&app, text("dialog").into()));
}
