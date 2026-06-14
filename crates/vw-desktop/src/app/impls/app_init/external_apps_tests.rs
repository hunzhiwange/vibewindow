use super::*;
use crate::app::state::RuntimePlatform;
use serde_json::json;

#[test]
fn configured_external_app_maps_supported_keys() {
    let cases = [
        ("finder", ExternalOpenApp::Finder),
        ("vscode", ExternalOpenApp::VSCode),
        ("cursor", ExternalOpenApp::Cursor),
        ("trae", ExternalOpenApp::Trae),
        ("windsurf", ExternalOpenApp::Windsurf),
        ("kiro", ExternalOpenApp::Kiro),
        ("zed", ExternalOpenApp::Zed),
        ("textmate", ExternalOpenApp::TextMate),
        ("antigravity", ExternalOpenApp::Antigravity),
        ("terminal", ExternalOpenApp::Terminal),
        ("iterm2", ExternalOpenApp::ITerm2),
        ("ghostty", ExternalOpenApp::Ghostty),
        ("xcode", ExternalOpenApp::Xcode),
        ("android-studio", ExternalOpenApp::AndroidStudio),
        ("powershell", ExternalOpenApp::PowerShell),
        ("sublime-text", ExternalOpenApp::SublimeText),
    ];

    for (key, expected) in cases {
        let cfg = json!({ "open_external_app": key });
        assert_eq!(configured_external_app(&cfg), Some(expected));
    }
}

#[test]
fn configured_external_app_rejects_missing_non_string_and_unknown_keys() {
    for cfg in
        [json!({}), json!({ "open_external_app": true }), json!({ "open_external_app": "unknown" })]
    {
        assert_eq!(configured_external_app(&cfg), None);
    }
}

#[test]
fn priority_external_apps_uses_macos_order() {
    assert_eq!(
        priority_external_apps(Some(RuntimePlatform::MacOs)),
        &[
            ExternalOpenApp::Trae,
            ExternalOpenApp::Windsurf,
            ExternalOpenApp::Kiro,
            ExternalOpenApp::Cursor,
            ExternalOpenApp::VSCode,
            ExternalOpenApp::Zed,
            ExternalOpenApp::Xcode,
            ExternalOpenApp::AndroidStudio,
            ExternalOpenApp::SublimeText,
            ExternalOpenApp::TextMate,
            ExternalOpenApp::Antigravity,
            ExternalOpenApp::Ghostty,
            ExternalOpenApp::ITerm2,
            ExternalOpenApp::Terminal,
            ExternalOpenApp::Finder,
        ]
    );
}

#[test]
fn priority_external_apps_uses_windows_order() {
    assert_eq!(
        priority_external_apps(Some(RuntimePlatform::Windows)),
        &[
            ExternalOpenApp::Trae,
            ExternalOpenApp::Windsurf,
            ExternalOpenApp::Kiro,
            ExternalOpenApp::Cursor,
            ExternalOpenApp::VSCode,
            ExternalOpenApp::Zed,
            ExternalOpenApp::SublimeText,
            ExternalOpenApp::PowerShell,
            ExternalOpenApp::Finder,
        ]
    );
}

#[test]
fn priority_external_apps_uses_linux_order_for_linux_and_unknown_platforms() {
    let expected = &[
        ExternalOpenApp::Trae,
        ExternalOpenApp::Windsurf,
        ExternalOpenApp::Kiro,
        ExternalOpenApp::Cursor,
        ExternalOpenApp::VSCode,
        ExternalOpenApp::Zed,
        ExternalOpenApp::SublimeText,
        ExternalOpenApp::Finder,
    ];

    assert_eq!(priority_external_apps(Some(RuntimePlatform::Linux)), expected);
    assert_eq!(priority_external_apps(None), expected);
}

#[test]
fn resolve_external_apps_honors_configured_available_finder() {
    let (platform, exists, selected) = resolve_external_apps(&json!({
        "open_external_app": "finder",
    }));

    assert_eq!(platform, RuntimePlatform::current_target());
    assert_eq!(exists.get(&ExternalOpenApp::Finder), Some(&true));
    assert_eq!(selected, ExternalOpenApp::Finder);
}

#[test]
fn resolve_external_apps_falls_back_when_configured_app_is_unknown() {
    let (platform, exists, selected) = resolve_external_apps(&json!({
        "open_external_app": "unknown",
    }));

    assert_eq!(platform, RuntimePlatform::current_target());
    assert_eq!(exists.get(&ExternalOpenApp::Finder), Some(&true));
    assert!(exists.get(&selected).copied().unwrap_or(false));
}
