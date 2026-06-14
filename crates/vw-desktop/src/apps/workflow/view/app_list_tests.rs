//! Workflow 应用列表视图测试模块，覆盖列表状态、文案回退、时间格式化与样式分支。

use super::*;
use crate::apps::workflow::state::WorkflowSavedAppSummary;
use iced::widget::button;
use iced::{Background, Theme};

#[test]
fn saved_app_title_uses_fallback_for_blank_name() {
    assert_eq!(saved_app_title("  "), "未命名应用");
}

#[test]
fn saved_app_title_trims_non_blank_name() {
    assert_eq!(saved_app_title("  客服助手  "), "客服助手");
}

#[test]
fn saved_app_description_uses_fallback_for_blank_description() {
    assert_eq!(saved_app_description("  "), "暂无描述");
}

#[test]
fn saved_app_description_trims_non_blank_description() {
    assert_eq!(saved_app_description("  自动回复客户问题  "), "自动回复客户问题");
}

#[test]
fn format_saved_app_time_rejects_invalid_timestamp() {
    assert_eq!(format_saved_app_time(u64::MAX), "--");
}

#[test]
fn format_saved_app_time_rejects_out_of_range_i64_timestamp() {
    assert_eq!(format_saved_app_time(i64::MAX as u64), "--");
}

#[test]
fn format_saved_app_time_formats_valid_timestamp() {
    let formatted = format_saved_app_time(0);

    assert_ne!(formatted, "--");
    assert_eq!(formatted.len(), "1970/01/01 08:00".len());
}

#[test]
fn saved_app_matches_search_query() {
    let app = WorkflowSavedAppSummary {
        name: "DHB 数据洞察".to_string(),
        description: "数据智能分析".to_string(),
        ..saved_app_summary()
    };

    assert!(saved_app_matches_query(&app, "dhb"));
    assert!(saved_app_matches_query(&app, "智能"));
    assert!(saved_app_matches_query(&app, " UUID "));
    assert!(saved_app_matches_query(&app, "  "));
    assert!(!saved_app_matches_query(&app, "客服"));
}

#[test]
fn saved_apps_view_builds_loading_error_empty_and_filtered_states() {
    let mut state = WorkflowState { saved_apps_loading: true, ..WorkflowState::default() };
    let _ = build_saved_apps_view(&state);

    state.saved_apps_loading = false;
    state.saved_apps_error = Some("数据库不可用".to_string());
    let _ = build_saved_apps_view(&state);

    state.saved_apps_error = None;
    let _ = build_saved_apps_view(&state);

    state.saved_apps = vec![saved_app_summary()];
    let _ = build_saved_apps_view(&state);

    state.saved_app_search_query = "无匹配".to_string();
    let _ = build_saved_apps_view(&state);
}

#[test]
fn saved_apps_header_and_empty_state_parts_build() {
    let state = WorkflowState::default();

    let _header = build_saved_apps_header(&state);
    let _search = build_search_input(&state);
    let _refresh = saved_apps_header_button("刷新", WorkflowMessage::LoadSavedApps);
    let _create_card = build_create_app_card();
    let _create_button =
        create_action_button(Icon::FileEarmarkPlus, "创建空白应用", WorkflowMessage::OpenFile);
    let _create_row = create_action_row(Icon::FolderOpen, "导入 DSL 文件");
    let _no_result = build_no_result_card();
    let _notice = build_saved_apps_notice("暂无应用");
    let _error = build_saved_apps_error("数据库不可用");
}

#[test]
fn saved_app_card_builds_action_and_progress_states() {
    let state = WorkflowState {
        saved_apps: vec![saved_app_summary()],
        opening_saved_app_uuid: Some("uuid".to_string()),
        deleting_saved_app_uuid: Some("uuid".to_string()),
        saved_app_actions_menu_uuid: Some("uuid".to_string()),
        copied_saved_app_uuid: Some("uuid".to_string()),
        ..WorkflowState::default()
    };

    let _ = build_saved_app_card(&state, &state.saved_apps[0]);
    let _ = build_saved_app_actions_menu("uuid".to_string(), true, true);
    let _ = build_saved_app_action_item(Icon::Trash, "删除中", None, true);
    let _ = saved_app_uuid_row("uuid", true);
    let _ = saved_app_uuid_row("uuid", false);
}

#[test]
fn delete_confirm_dialog_uses_matching_app_or_generic_name() {
    let state = WorkflowState {
        saved_apps: vec![saved_app_summary()],
        confirm_delete_saved_app_uuid: Some("uuid".to_string()),
        ..WorkflowState::default()
    };

    assert!(build_saved_app_delete_confirm_dialog(&state).is_some());

    let missing_state = WorkflowState {
        confirm_delete_saved_app_uuid: Some("missing".to_string()),
        ..WorkflowState::default()
    };

    assert!(build_saved_app_delete_confirm_dialog(&missing_state).is_some());
    assert!(build_saved_app_delete_confirm_dialog(&WorkflowState::default()).is_none());
}

#[test]
fn saved_app_styles_cover_light_dark_and_button_statuses() {
    let light = Theme::Light;
    let dark = Theme::Dark;

    assert!(saved_app_actions_menu_style(&light).background.is_some());
    assert!(saved_app_actions_menu_style(&dark).background.is_some());
    assert!(saved_app_card_container_style(&light).background.is_some());
    assert!(saved_app_card_container_style(&dark).background.is_some());
    assert!(saved_app_name_text_style(&dark).color.is_some());

    let enabled_hovered = saved_app_action_item_style(&light, button::Status::Hovered, true, true);
    let enabled_pressed = saved_app_action_item_style(&light, button::Status::Pressed, false, true);
    let enabled_active = saved_app_action_item_style(&light, button::Status::Active, false, true);
    let disabled_active = saved_app_action_item_style(&light, button::Status::Active, false, false);
    let disabled_hovered =
        saved_app_action_item_style(&light, button::Status::Hovered, true, false);

    assert!(enabled_hovered.background.is_some());
    assert!(enabled_pressed.background.is_some());
    assert!(enabled_active.background.is_none());
    assert!(disabled_active.background.is_none());
    assert!(disabled_hovered.background.is_none());
    assert!(saved_app_card_button_style(&light, button::Status::Active).background.is_none());
}

#[test]
fn saved_app_tag_and_badge_styles_build_for_dark_theme() {
    let _ = saved_app_robot_badge();
    let _ = saved_app_tag("工作流");

    let card_style = saved_app_card_container_style(&Theme::Dark);
    let Background::Color(background) = card_style.background.expect("card background") else {
        panic!("card background should be a color");
    };

    assert!(background.a > 0.0);
}

fn saved_app_summary() -> WorkflowSavedAppSummary {
    WorkflowSavedAppSummary {
        uuid: "uuid".to_string(),
        name: "应用".to_string(),
        description: String::new(),
        created_at_ms: 0,
        updated_at_ms: 0,
    }
}
