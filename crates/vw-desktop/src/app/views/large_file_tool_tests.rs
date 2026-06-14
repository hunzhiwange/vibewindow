use crate::app::App;
use crate::app::message::large_file_tool::{
    LargeFileCategory, LargeFileEntry, LargeFileScanReport,
};
use iced::Theme;
use iced::widget::button;

fn test_app() -> App {
    App::new().0
}

fn report_with_files() -> LargeFileScanReport {
    LargeFileScanReport {
        root: "/tmp/project".to_string(),
        total_bytes: 1_700_000_000,
        total_files: 2,
        categories: vec![
            LargeFileCategory {
                id: "giga".to_string(),
                title: "1GB+".to_string(),
                subtitle: "超大文件".to_string(),
                total_bytes: 1_200_000_000,
                files: vec![LargeFileEntry {
                    name: "vm.img".to_string(),
                    path: "/tmp/project/vm.img".to_string(),
                    parent: "/tmp/project".to_string(),
                    size_bytes: 1_200_000_000,
                }],
            },
            LargeFileCategory {
                id: "500m".to_string(),
                title: "500MB-1GB".to_string(),
                subtitle: "较大文件".to_string(),
                total_bytes: 500_000_000,
                files: vec![LargeFileEntry {
                    name: "cache.bin".to_string(),
                    path: "/tmp/project/cache.bin".to_string(),
                    parent: "/tmp/project".to_string(),
                    size_bytes: 500_000_000,
                }],
            },
        ],
    }
}

#[test]
fn module_is_wired() {
    assert!(module_path!().ends_with("large_file_tool_tests"));
}

#[test]
fn view_builds_initial_state() {
    let app = test_app();

    let _element = super::view(&app);
}

#[test]
fn view_builds_scanning_state_with_current_path() {
    let mut app = test_app();
    app.large_file_scanning = true;
    app.large_file_animation_frame = 3;
    app.large_file_progress_label = "遍历目录".to_string();
    app.large_file_current_path = "/tmp/project/src".to_string();
    app.large_file_progress_value = 0.42;
    app.large_file_processed_files = 21;
    app.large_file_total_files = 50;
    app.large_file_report = Some(report_with_files());
    app.large_file_notification = Some("正在扫描".to_string());

    let _element = super::view(&app);
}

#[test]
fn view_builds_deleting_state_with_selection() {
    let mut app = test_app();
    app.large_file_deleting = true;
    app.large_file_scanned = true;
    app.large_file_report = Some(report_with_files());
    app.large_file_selected_entries.insert("/tmp/project/vm.img".to_string());

    let _element = super::view(&app);
}

#[test]
fn view_builds_empty_scanned_result() {
    let mut app = test_app();
    app.large_file_scanned = true;
    app.large_file_report = Some(LargeFileScanReport {
        root: "/tmp/project".to_string(),
        total_bytes: 0,
        total_files: 0,
        categories: Vec::new(),
    });

    let _element = super::view(&app);
}

#[test]
fn view_builds_empty_filtered_result() {
    let mut app = test_app();
    app.large_file_scanned = true;
    app.large_file_active_filter = "50m".to_string();
    app.large_file_report = Some(report_with_files());

    let _element = super::view(&app);
}

#[test]
fn view_builds_populated_filtered_result() {
    let mut app = test_app();
    app.large_file_scanned = true;
    app.large_file_active_filter = "giga".to_string();
    app.large_file_report = Some(report_with_files());
    app.large_file_selected_entries.insert("/tmp/project/vm.img".to_string());

    let _element = super::view(&app);
}

#[test]
fn component_helpers_build_standalone_elements() {
    let mut app = test_app();
    app.large_file_report = Some(report_with_files());
    let report = app.large_file_report.as_ref().expect("report");

    let _stat = super::stat_card("命中文件", "2");
    let _filter = super::filter_button(&app, "all", "全部");
    {
        let _scan_waiting = super::scanning_view(&app, "◐");
    }
    app.large_file_current_path = "/tmp/project".to_string();
    let _scan_path = super::scanning_view(&app, "◓");
    let _empty = super::empty_view("没有结果");
    let _category = super::category_card(&app, &report.categories[0]);
}

#[test]
fn container_styles_follow_theme_palette() {
    let theme = Theme::Dark;

    assert!(super::hero_style(&theme).background.is_some());
    assert!(super::card_style(&theme).background.is_some());
    assert!(super::sub_card_style(&theme).background.is_some());
}

#[test]
fn text_styles_use_expected_palette_slots() {
    let theme = Theme::Dark;
    let palette = theme.extended_palette();

    assert_eq!(super::success_text_style(&theme).color, Some(palette.success.base.color));
    assert_eq!(
        super::secondary_strong_text_style(&theme).color,
        Some(palette.secondary.strong.color)
    );
    assert_eq!(super::secondary_base_text_style(&theme).color, Some(palette.secondary.base.color));
    assert_eq!(super::primary_strong_text_style(&theme).color, Some(palette.primary.strong.color));
}

#[test]
fn button_styles_cover_interaction_states() {
    let theme = Theme::Dark;
    let statuses = [
        button::Status::Active,
        button::Status::Hovered,
        button::Status::Pressed,
        button::Status::Disabled,
    ];

    for status in statuses {
        assert!(super::primary_button_style(&theme, status).background.is_some());
        assert!(super::secondary_button_style(&theme, status).background.is_some());
        assert!(super::danger_button_style(&theme, status).background.is_some());
        assert!(super::filter_button_style(&theme, status, false).background.is_some());
        assert!(super::filter_button_style(&theme, status, true).background.is_some());
    }
}
