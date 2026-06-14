use super::*;
use crate::app::App;
use crate::app::message::settings::channels::toggles;

fn app() -> App {
    App::new().0
}

#[test]
fn enabled_channels_reports_human_readable_names() {
    let mut app = app();
    app.channels_settings.cli = true;
    toggles::toggle_enabled(&mut app, "telegram", true);
    toggles::toggle_enabled(&mut app, "lark", true);

    let names = enabled_channels(&app.channels_settings);
    assert!(names.contains(&"CLI"));
    assert!(names.contains(&"Telegram"));
    assert!(names.contains(&"Lark"));
}

#[test]
fn persist_channels_requires_at_least_one_enabled_channel() {
    let mut app = app();
    app.channels_settings.cli = false;
    app.channels_settings.telegram = None;
    app.channels_settings.discord = None;
    app.channels_settings.slack = None;
    app.channels_settings.mattermost = None;
    app.channels_settings.webhook = None;
    app.channels_settings.imessage = None;
    app.channels_settings.matrix = None;
    app.channels_settings.signal = None;
    app.channels_settings.whatsapp = None;
    app.channels_settings.linq = None;
    app.channels_settings.wati = None;
    app.channels_settings.nextcloud_talk = None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.channels_settings.email = None;
    }
    app.channels_settings.irc = None;
    app.channels_settings.lark = None;
    app.channels_settings.feishu = None;
    app.channels_settings.dingtalk = None;
    app.channels_settings.qq = None;
    app.channels_settings.nostr = None;
    app.channels_settings.clawdtalk = None;

    assert!(persist_channels_settings(&mut app).is_none());
    assert!(app.channels_settings.save_error.as_deref().unwrap_or("").contains("至少保留一个"));

    app.channels_settings.cli = true;
    assert!(persist_channels_settings(&mut app).is_some());
    assert!(app.channels_settings.save_error.is_none());
}
