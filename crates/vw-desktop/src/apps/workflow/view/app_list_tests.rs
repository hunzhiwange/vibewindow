//! Workflow 应用列表视图测试模块，覆盖列表文案回退与时间格式化。

use super::{
    format_saved_app_time, saved_app_description, saved_app_matches_query, saved_app_title,
};
use crate::apps::workflow::state::WorkflowSavedAppSummary;

#[test]
fn saved_app_title_uses_fallback_for_blank_name() {
    assert_eq!(saved_app_title("  "), "未命名应用");
}

#[test]
fn saved_app_description_uses_fallback_for_blank_description() {
    assert_eq!(saved_app_description("  "), "暂无描述");
}

#[test]
fn format_saved_app_time_rejects_invalid_timestamp() {
    assert_eq!(format_saved_app_time(u64::MAX), "--");
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
    assert!(!saved_app_matches_query(&app, "客服"));
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
