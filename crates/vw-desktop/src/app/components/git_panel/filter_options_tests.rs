#[test]
fn task_717_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("filter_options_tests.rs"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

#[test]
fn help_copy_is_stable_and_mentions_all_filter_controls() {
    assert_eq!(super::filter_options::FILTER_HELP_TITLE, "筛选选项");

    for expected in
        ["路径筛选", "已包含到提交", "已排除出提交", "新增文件", "修改文件", "删除文件", "清除筛选"]
    {
        assert!(super::filter_options::FILTER_HELP_TEXT.contains(expected), "{expected}");
    }
}

#[test]
fn view_builds_with_empty_filters_and_zero_counts() {
    let app = app();

    let _element = super::filter_options::view(&app, 0, 0, 0, 0, 0);
}

#[test]
fn view_builds_with_active_query_and_all_toggles() {
    let mut app = app();
    app.git_filter_query = "src/lib".to_string();
    app.git_filter_included = true;
    app.git_filter_excluded = true;
    app.git_filter_new = true;
    app.git_filter_modified = true;
    app.git_filter_deleted = true;

    let _element = super::filter_options::view(&app, 3, 4, 1, 2, 3);
}
