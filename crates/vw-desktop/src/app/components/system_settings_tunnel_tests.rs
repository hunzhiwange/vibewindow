use super::*;
use iced::Element;
use crate::app::message::settings::{SettingsMessage, TunnelMessage};
use crate::app::{App, Message};
use iced::widget::text;

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn labeled_option_displays_label() {
    let option = LabeledOption { value: "cloudflare", label: "Cloudflare" };

    assert_eq!(option.to_string(), "Cloudflare");
}

#[test]
fn row_helpers_build_text_and_boolean_controls() {
    keep_element(field_row("标签", "说明", text("control")));
    keep_element(text_row("令牌", "说明", "placeholder", "token", |value| {
        Message::Settings(SettingsMessage::Tunnel(TunnelMessage::CloudflareTokenChanged(value)))
    }));
    keep_element(bool_row("漏斗", "说明", true, "启用", |value| {
        Message::Settings(SettingsMessage::Tunnel(TunnelMessage::TailscaleFunnelToggled(value)))
    }));
}

#[test]
fn view_builds_all_provider_sections_and_unknown_fallback() {
    let mut app = test_app();
    for provider in ["none", "cloudflare", "tailscale", "ngrok", "custom", "invalid"] {
        app.tunnel_settings.provider = provider.to_string();
        app.tunnel_settings.cloudflare_token = "cf-token".to_string();
        app.tunnel_settings.tailscale_funnel = true;
        app.tunnel_settings.tailscale_hostname = "node.tailnet.ts.net".to_string();
        app.tunnel_settings.ngrok_auth_token = "ngrok-token".to_string();
        app.tunnel_settings.ngrok_domain = "example.ngrok.app".to_string();
        app.tunnel_settings.custom_start_command = "bore local {port}".to_string();
        app.tunnel_settings.custom_health_url = "http://127.0.0.1:4040".to_string();
        app.tunnel_settings.custom_url_pattern = "https://".to_string();
        keep_element(view(&app));
    }
}

#[test]
fn view_appends_save_error_banner() {
    let mut app = test_app();
    app.tunnel_settings.save_error = Some("保存失败".to_string());

    keep_element(view(&app));
}
