#[test]
fn settings_tab_display_labels_are_stable() {
    assert_eq!(super::SettingsTab::Settings.to_string(), "设置");
    assert_eq!(super::SettingsTab::SettingsJson.to_string(), "JSON配置");
    assert_eq!(super::SettingsTab::Files.to_string(), "文件管理器");
    assert_eq!(super::SettingsTab::Projects.to_string(), "项目管理器");
    assert_eq!(super::SettingsTab::Sessions.to_string(), "历史会话");
}

#[test]
fn settings_tab_all_keeps_primary_tabs_only() {
    assert_eq!(
        super::SettingsTab::all(),
        [super::SettingsTab::Settings, super::SettingsTab::Files, super::SettingsTab::Projects]
    );
}
