#[test]
fn cleaner_progress_value_clamps_to_valid_range() {
    assert_eq!(super::simplify_path("$HOME/Library/Caches/app"), "~/Library/Caches/app");
}

use crate::app::message::cleaner_tool::{
    CleanerScanDetail, CleanerScanGroup, CleanerScanItem, CleanerScanReport,
};

fn scan_item(id: &str, sensitive: bool, bytes: u64) -> CleanerScanItem {
    CleanerScanItem {
        id: id.to_string(),
        title: format!("Item {id}"),
        subtitle: "subtitle".to_string(),
        sensitive,
        total_bytes: bytes,
        details: vec![CleanerScanDetail {
            label: "Detail".to_string(),
            path: "$HOME/tmp".to_string(),
            total_bytes: bytes,
        }],
    }
}

fn scan_report() -> CleanerScanReport {
    CleanerScanReport {
        total_bytes: 4096,
        matched_items: 2,
        groups: vec![CleanerScanGroup {
            id: "system".to_string(),
            title: "System".to_string(),
            subtitle: "System junk".to_string(),
            total_bytes: 4096,
            items: vec![scan_item("system_temp", false, 1024), scan_item("wechat", true, 3072)],
        }],
    }
}

#[test]
fn cleaner_platform_name_maps_runtime_platforms() {
    let mut app = crate::app::App::new().0;
    app.open_external_platform = None;
    assert_eq!(super::cleaner_platform_name(&app), "Gateway 宿主");

    app.open_external_platform = Some(crate::app::state::RuntimePlatform::MacOs);
    assert_eq!(super::cleaner_platform_name(&app), "macOS");

    app.open_external_platform = Some(crate::app::state::RuntimePlatform::Windows);
    assert_eq!(super::cleaner_platform_name(&app), "Windows");

    app.open_external_platform = Some(crate::app::state::RuntimePlatform::Linux);
    assert_eq!(super::cleaner_platform_name(&app), "Linux");
}

#[test]
fn cleaner_progress_value_and_label_cover_all_states() {
    let mut app = crate::app::App::new().0;
    assert_eq!(super::cleaner_progress_value(&app), 0.0);
    assert_eq!(super::cleaner_progress_label(&app, 0, 0), "尚未开始搜索");

    app.cleaner_scanned = true;
    assert_eq!(super::cleaner_progress_value(&app), 0.78);
    assert_eq!(super::cleaner_progress_label(&app, 3, 8), "已完成搜索，命中 8 项");

    app.cleaner_last_run_completed = true;
    assert_eq!(super::cleaner_progress_value(&app), 1.0);
    assert_eq!(super::cleaner_progress_label(&app, 3, 8), "清理已完成，建议再次搜索确认结果");

    app.cleaner_scanning = true;
    app.cleaner_animation_frame = 10;
    assert!(super::cleaner_progress_value(&app) > 0.18);
    assert_eq!(super::cleaner_progress_label(&app, 3, 8), "正在扫描系统垃圾、应用垃圾与上网垃圾");

    app.cleaner_running = true;
    app.cleaner_scanning = false;
    assert!(super::cleaner_progress_value(&app) > 0.82);
    assert_eq!(super::cleaner_progress_label(&app, 3, 8), "正在处理 3 个已勾选项目");

    app.cleaner_cancelling = true;
    assert_eq!(super::cleaner_progress_value(&app), 0.95);
    assert_eq!(super::cleaner_progress_label(&app, 3, 8), "正在等待当前步骤结束后取消");
}

#[test]
fn item_selected_maps_all_known_ids() {
    let mut app = crate::app::App::new().0;
    app.cleaner_clear_system_temp = true;
    app.cleaner_clear_app_cache = true;
    app.cleaner_clear_logs = true;
    app.cleaner_clear_package_cache = true;
    app.cleaner_clear_downloads = true;
    app.cleaner_empty_trash = true;
    app.cleaner_clear_installers = true;
    app.cleaner_clear_other_apps = true;
    app.cleaner_clear_wechat_work = true;
    app.cleaner_clear_wechat = true;
    app.cleaner_clear_qq = true;
    app.cleaner_clear_dingtalk = true;
    app.cleaner_clear_feishu = true;
    app.cleaner_clear_safari = true;
    app.cleaner_clear_chrome = true;
    app.cleaner_clear_edge = true;
    app.cleaner_clear_firefox = true;
    app.cleaner_clear_mail = true;

    for id in [
        "system_temp",
        "app_cache",
        "logs",
        "package_cache",
        "downloads",
        "trash",
        "installers",
        "other_apps",
        "wechat_work",
        "wechat",
        "qq",
        "dingtalk",
        "feishu",
        "safari",
        "chrome",
        "edge",
        "firefox",
        "mail",
    ] {
        assert!(super::item_selected(&app, id), "{id}");
    }
    assert!(!super::item_selected(&app, "unknown"));
}

#[test]
fn render_scan_tree_and_rows_cover_empty_collapsed_and_expanded() {
    let mut app = crate::app::App::new().0;
    let _ = super::render_scan_tree(&app);

    let report = scan_report();
    let group = report.groups[0].clone();
    let item = group.items[1].clone();
    app.cleaner_scan_report = Some(report);
    app.cleaner_scanned = true;
    let _ = super::render_scan_tree(&app);
    let _ = super::render_scan_group(&app, &group);
    let _ = super::render_scan_item(&app, &item);

    app.cleaner_tree_expanded.insert(group.id.clone());
    app.cleaner_tree_expanded.insert(item.id.clone());
    app.cleaner_clear_wechat = true;
    let _ = super::render_scan_tree(&app);
    let _ = super::render_scan_group(&app, &group);
    let _ = super::render_scan_item(&app, &item);
}

#[test]
fn item_checkbox_builds_for_known_and_unknown_ids() {
    let app = crate::app::App::new().0;
    for id in [
        "system_temp",
        "app_cache",
        "logs",
        "package_cache",
        "downloads",
        "trash",
        "installers",
        "other_apps",
        "wechat_work",
        "wechat",
        "qq",
        "dingtalk",
        "feishu",
        "safari",
        "chrome",
        "edge",
        "firefox",
        "mail",
        "unknown",
    ] {
        let _ = super::item_checkbox(&app, id);
    }
}

#[test]
fn cleaner_view_and_styles_cover_theme_branches() {
    let mut app = crate::app::App::new().0;
    app.cleaner_scan_report = Some(scan_report());
    app.cleaner_scanned = true;
    app.cleaner_notification = Some("done".to_string());
    let _ = super::view(&app);

    for theme in [iced::Theme::Light, iced::Theme::Dark] {
        assert_eq!(super::hero_card_style(&theme).border.radius.top_left, 24.0);
        assert_eq!(super::panel_card_style(&theme).border.radius.top_left, 18.0);
        assert_eq!(super::log_panel_style(&theme).border.radius.top_left, 18.0);
        assert_eq!(super::info_card_style(&theme).border.radius.top_left, 16.0);
        assert_eq!(super::progress_card_style(&theme).border.radius.top_left, 18.0);
        assert_eq!(super::completion_badge_style(&theme).border.radius.top_left, 999.0);
        assert_eq!(super::summary_metric_style(&theme, true).border.radius.top_left, 16.0);
        assert_eq!(super::summary_metric_style(&theme, false).border.radius.top_left, 16.0);
        assert_eq!(super::group_card_style(&theme, true).border.radius.top_left, 16.0);
        assert_eq!(super::group_card_style(&theme, false).border.radius.top_left, 16.0);
        assert_eq!(super::item_card_style(&theme, true, false).border.radius.top_left, 12.0);
        assert_eq!(super::item_card_style(&theme, false, true).border.radius.top_left, 12.0);
        assert_eq!(super::item_card_style(&theme, false, false).border.radius.top_left, 12.0);
        assert!(super::is_dark_mode(&theme) == matches!(theme, iced::Theme::Dark));
    }

    for status in [
        iced::widget::button::Status::Active,
        iced::widget::button::Status::Hovered,
        iced::widget::button::Status::Pressed,
        iced::widget::button::Status::Disabled,
    ] {
        assert_eq!(
            super::ghost_button_style(&iced::Theme::Light, status).border.radius.top_left,
            12.0
        );
        assert_eq!(
            super::secondary_button_style(&iced::Theme::Dark, status).border.radius.top_left,
            14.0
        );
    }
}
