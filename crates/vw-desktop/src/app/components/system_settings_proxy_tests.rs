use super::*;
use iced::Element;
use iced::widget::text;
use vw_config_types::proxy::ProxyScope;

#[test]
fn proxy_scope_label_returns_stable_user_labels() {
    assert_eq!(proxy_scope_label(ProxyScope::Environment), "系统环境");
    assert_eq!(proxy_scope_label(ProxyScope::Vibewindow), "VibeWindow");
    assert_eq!(proxy_scope_label(ProxyScope::Services), "指定服务");
}

#[test]
fn proxy_scope_options_match_scope_labels() {
    assert_eq!(PROXY_SCOPE_OPTIONS, ["系统环境", "VibeWindow", "指定服务"]);

    for scope in [ProxyScope::Environment, ProxyScope::Vibewindow, ProxyScope::Services] {
        assert!(PROXY_SCOPE_OPTIONS.contains(&proxy_scope_label(scope)));
    }
}

#[test]
fn field_row_accepts_fill_width_control() {
    let element: Element<'_, Message> =
        field_row("HTTP 代理", "HTTP 请求代理地址。", text("value"));
    drop(element);
}
