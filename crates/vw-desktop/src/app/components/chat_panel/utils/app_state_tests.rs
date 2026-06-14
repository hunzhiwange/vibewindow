use super::app_state::{
    current_branch_label, current_project_path_label, get_session_title, is_recent_copy,
};
use crate::app::App;
use web_time::{Duration as WebDuration, SystemTime as WebSystemTime};

#[test]
fn app_state_labels_fall_back_when_values_are_missing() {
    let app = App::new().0;

    assert_eq!(get_session_title(&app), "暂无");
    assert_eq!(current_project_path_label(&app), "未打开项目");
    assert_eq!(current_branch_label(&app), "-");
}

#[test]
fn is_recent_copy_requires_matching_hash_and_fresh_timestamp() {
    let mut app = App::new().0;
    app.last_copied_code_hash = Some(42);
    app.last_copy_time = Some(WebSystemTime::now() - WebDuration::from_millis(500));

    assert!(is_recent_copy(&app, 42));
    assert!(!is_recent_copy(&app, 7));

    app.last_copy_time = Some(WebSystemTime::now() - WebDuration::from_millis(2_000));
    assert!(!is_recent_copy(&app, 42));
}
