use super::*;
use iced::Element;
use crate::app::message::settings::StorageMessage;
use crate::app::{App, Message, message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn field_rows_build_labels_inputs_and_controls() {
    keep_element(field_row("标签", "说明", text("control")));
    keep_element(text_row("地址", "说明", "placeholder", "value", |value| {
        Message::Settings(message::SettingsMessage::Storage(StorageMessage::DbUrlChanged(value)))
    }));
}

#[test]
fn view_builds_with_default_known_and_unknown_provider() {
    let app = test_app();
    keep_element(view(&app));

    let mut app = test_app();
    app.storage_settings.provider = "postgres".to_string();
    app.storage_settings.db_url_input = "postgres://user:pass@localhost/db".to_string();
    app.storage_settings.schema = "public".to_string();
    app.storage_settings.table = "memories".to_string();
    app.storage_settings.connect_timeout_secs_input = "30".to_string();
    app.storage_settings.tls = true;
    keep_element(view(&app));

    app.storage_settings.provider = "unsupported".to_string();
    keep_element(view(&app));
}

#[test]
fn view_appends_save_error_banner() {
    let mut app = test_app();
    app.storage_settings.save_error = Some("保存失败".to_string());

    keep_element(view(&app));
}
