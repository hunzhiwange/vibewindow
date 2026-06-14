#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("find_tests"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn find_tab(id: &str) -> crate::app::state::FindInFolderTab {
    crate::app::state::FindInFolderTab {
        id: id.to_string(),
        title: "查找".to_string(),
        scope_path: "/tmp/project".to_string(),
        query_input: String::new(),
        replace_input: String::new(),
        query_editor: iced::widget::text_editor::Content::new(),
        replace_editor: iced::widget::text_editor::Content::new(),
        query: String::new(),
        replace_text: String::new(),
        case_sensitive: false,
        whole_word: false,
        use_regex: false,
        running: false,
        error: None,
        limit_reached: false,
        matches: Vec::new(),
    }
}

#[test]
fn option_toggles_update_matching_tab_only() {
    let mut app = app();
    app.find_results_tabs.push(find_tab("a"));
    app.find_results_tabs.push(find_tab("b"));

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindCaseSensitiveToggled(
            "a".to_string(),
            true,
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindWholeWordToggled(
            "a".to_string(),
            true,
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindRegexToggled(
            "a".to_string(),
            true,
        ),
    );

    assert!(app.find_results_tabs[0].case_sensitive);
    assert!(app.find_results_tabs[0].whole_word);
    assert!(app.find_results_tabs[0].use_regex);
    assert!(!app.find_results_tabs[1].case_sensitive);
    assert!(!app.find_results_tabs[1].whole_word);
    assert!(!app.find_results_tabs[1].use_regex);
}

#[test]
fn find_run_with_empty_query_sets_error_without_running() {
    let mut app = app();
    app.find_results_tabs.push(find_tab("a"));

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindRun("a".to_string()),
    );

    let tab = &app.find_results_tabs[0];
    assert_eq!(tab.error.as_deref(), Some("请输入查找关键字"));
    assert!(!tab.running);
}

#[test]
fn refresh_active_with_empty_query_sets_error() {
    let mut app = app();
    app.find_results_tabs.push(find_tab("a"));
    app.active_find_results_tab_id = Some("a".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindRefreshActive,
    );

    assert_eq!(app.find_results_tabs[0].error.as_deref(), Some("请输入查找关键字"));
}

#[test]
fn find_in_project_creates_active_find_tab_and_open_tab() {
    let mut app = app();
    app.project_path = Some("/tmp/project".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindInProject,
    );

    assert_eq!(app.find_results_tabs.len(), 1);
    assert_eq!(app.find_results_tabs[0].scope_path, "/tmp/project");
    assert_eq!(app.active_find_results_tab_id, Some(app.find_results_tabs[0].id.clone()));
    assert!(app.show_file_manager);
    assert!(app.active_tab_id.as_deref().is_some_and(|id| id.starts_with("find:")));
    assert!(matches!(app.screen, crate::app::Screen::Project));
}

#[test]
fn find_completed_updates_existing_tab_without_replacing_matches_on_error() {
    let mut app = app();
    let mut tab = find_tab("a");
    tab.matches.push(crate::app::state::FindInFolderMatch {
        path: "/tmp/project/a.rs".to_string(),
        line: 1,
        column: 2,
        preview: "old".to_string(),
        match_len: 3,
    });
    app.find_results_tabs.push(tab);

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindCompleted {
            tab_id: "a".to_string(),
            title: "查找: main".to_string(),
            scope_path: "/tmp/project".to_string(),
            query: "main".to_string(),
            replace_text: "lib".to_string(),
            case_sensitive: true,
            whole_word: true,
            use_regex: false,
            matches: Vec::new(),
            error: Some("bad regex".to_string()),
            limit_reached: true,
        },
    );

    let tab = &app.find_results_tabs[0];
    assert_eq!(tab.title, "查找: main");
    assert_eq!(tab.query, "main");
    assert_eq!(tab.replace_text, "lib");
    assert!(tab.case_sensitive);
    assert!(tab.whole_word);
    assert!(!tab.running);
    assert_eq!(tab.error.as_deref(), Some("bad regex"));
    assert!(tab.limit_reached);
    assert_eq!(tab.matches.len(), 1);
}

#[test]
fn find_completed_creates_missing_tab_with_matches() {
    let mut app = app();
    let matches = vec![crate::app::state::FindInFolderMatch {
        path: "/tmp/project/a.rs".to_string(),
        line: 4,
        column: 5,
        preview: "fn main()".to_string(),
        match_len: 4,
    }];

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindCompleted {
            tab_id: "new".to_string(),
            title: "查找: main".to_string(),
            scope_path: "/tmp/project".to_string(),
            query: "main".to_string(),
            replace_text: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            matches,
            error: None,
            limit_reached: false,
        },
    );

    assert_eq!(app.find_results_tabs.len(), 1);
    assert_eq!(app.find_results_tabs[0].query_input, "main");
    assert_eq!(app.find_results_tabs[0].matches.len(), 1);
    assert_eq!(app.active_find_results_tab_id.as_deref(), Some("new"));
}

#[test]
fn tab_selected_empty_clears_active_find_tab() {
    let mut app = app();
    app.active_find_results_tab_id = Some("a".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindTabSelected(String::new()),
    );

    assert!(app.active_find_results_tab_id.is_none());
}

#[test]
fn tab_closed_removes_find_and_open_tabs_and_selects_last() {
    let mut app = app();
    app.find_results_tabs.push(find_tab("a"));
    app.find_results_tabs.push(find_tab("b"));
    app.open_tabs.push(crate::app::AppTab {
        id: "find:a".to_string(),
        title: "A".to_string(),
        screen: crate::app::Screen::Project,
        project_path: Some("/tmp/project".to_string()),
    });
    app.open_tabs.push(crate::app::AppTab {
        id: "find:b".to_string(),
        title: "B".to_string(),
        screen: crate::app::Screen::Project,
        project_path: Some("/tmp/project".to_string()),
    });
    app.active_find_results_tab_id = Some("a".to_string());
    app.active_tab_id = Some("find:a".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::FileTreeFindTabClosed("a".to_string()),
    );

    assert_eq!(
        app.find_results_tabs.iter().map(|tab| tab.id.as_str()).collect::<Vec<_>>(),
        vec!["b"]
    );
    assert_eq!(
        app.open_tabs.iter().map(|tab| tab.id.as_str()).collect::<Vec<_>>(),
        vec!["home", "find:b"]
    );
    assert_eq!(app.active_find_results_tab_id.as_deref(), Some("b"));
    assert_eq!(app.active_tab_id.as_deref(), Some("find:b"));
}
