use super::time::{
    format_chat_time_label, project_last_modified_ms, relative_modified_label,
    relative_time_bucket, relative_time_label, relative_time_label_for_bucket,
};
use crate::app::App;

#[test]
fn relative_time_bucket_groups_boundaries() {
    assert_eq!(relative_time_bucket(1_000, 1_000), (0, 0));
    assert_eq!(relative_time_bucket(0, 70_000), (0, 0));
    assert_eq!(relative_time_bucket(1_000, 121_000), (1, 2));
}

#[test]
fn relative_time_label_for_bucket_formats_units() {
    assert_eq!(relative_time_label_for_bucket((1, 3)), "3 分钟前");
    assert_eq!(relative_time_label_for_bucket((5, 2)), "2 年前");
}

#[test]
fn chat_time_label_is_not_empty_for_epoch() {
    assert!(!format_chat_time_label(0).is_empty());
}

#[test]
fn relative_time_label_and_modified_label_use_fallbacks() {
    let app = App::new().0;

    assert_eq!(project_last_modified_ms(&app), None);
    assert_eq!(relative_modified_label(&app), "最后修改 暂无");
    assert_eq!(relative_time_label(0), "刚刚");
}
