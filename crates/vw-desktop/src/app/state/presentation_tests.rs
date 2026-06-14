#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("presentation_tests"));
}

use super::{ConventionalCommitType, ExternalOpenApp, RuntimePlatform, TopBarGatewayTab};

#[test]
fn top_bar_gateway_tab_defaults_to_gateway() {
    assert_eq!(TopBarGatewayTab::default(), TopBarGatewayTab::Gateway);
}

#[test]
fn runtime_platform_parses_gateway_aliases_and_rejects_unknown_values() {
    assert_eq!(RuntimePlatform::from_gateway_str("macos"), Some(RuntimePlatform::MacOs));
    assert_eq!(RuntimePlatform::from_gateway_str(" Darwin "), Some(RuntimePlatform::MacOs));
    assert_eq!(RuntimePlatform::from_gateway_str("win32"), Some(RuntimePlatform::Windows));
    assert_eq!(RuntimePlatform::from_gateway_str("LINUX"), Some(RuntimePlatform::Linux));
    assert_eq!(RuntimePlatform::from_gateway_str("freebsd"), None);
}

#[test]
fn runtime_platform_file_manager_labels_match_platform() {
    assert_eq!(RuntimePlatform::MacOs.file_manager_label(), "Finder");
    assert_eq!(RuntimePlatform::Windows.file_manager_label(), "File Explorer");
    assert_eq!(RuntimePlatform::Linux.file_manager_label(), "File Manager");
}

#[test]
fn external_open_app_identifiers_and_labels_cover_all_variants() {
    let cases = [
        (ExternalOpenApp::Finder, "finder", "文件管理器"),
        (ExternalOpenApp::VSCode, "vscode", "VS Code"),
        (ExternalOpenApp::Cursor, "cursor", "Cursor"),
        (ExternalOpenApp::Trae, "trae", "Trae"),
        (ExternalOpenApp::Windsurf, "windsurf", "Windsurf"),
        (ExternalOpenApp::Kiro, "kiro", "Kiro"),
        (ExternalOpenApp::Zed, "zed", "Zed"),
        (ExternalOpenApp::TextMate, "textmate", "TextMate"),
        (ExternalOpenApp::Antigravity, "antigravity", "Antigravity"),
        (ExternalOpenApp::Terminal, "terminal", "Terminal"),
        (ExternalOpenApp::ITerm2, "iterm2", "iTerm2"),
        (ExternalOpenApp::Ghostty, "ghostty", "Ghostty"),
        (ExternalOpenApp::Xcode, "xcode", "Xcode"),
        (ExternalOpenApp::AndroidStudio, "android-studio", "Android Studio"),
        (ExternalOpenApp::PowerShell, "powershell", "PowerShell"),
        (ExternalOpenApp::SublimeText, "sublime-text", "Sublime Text"),
    ];

    for (app, identifier, label) in cases {
        assert_eq!(app.as_str(), identifier);
        assert_eq!(app.label(), label);
    }
}

#[test]
fn conventional_commit_types_round_trip_prefixes_and_display() {
    let prefixes = ConventionalCommitType::all()
        .into_iter()
        .map(|kind| {
            assert_eq!(kind.to_string(), kind.as_str());
            kind.as_str()
        })
        .collect::<Vec<_>>();

    assert_eq!(prefixes.len(), 19);
    assert_eq!(ConventionalCommitType::from_prefix("feat"), Some(ConventionalCommitType::Feat));
    assert_eq!(ConventionalCommitType::from_prefix("locale"), Some(ConventionalCommitType::Locale));
    assert_eq!(ConventionalCommitType::from_prefix("unknown"), None);
}
