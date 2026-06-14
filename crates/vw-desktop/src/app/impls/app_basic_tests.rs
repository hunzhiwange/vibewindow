use crate::app::state::{ExternalOpenApp, ToastKind};
use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::state::DesignState;
use crate::app::{App, Screen};
use iced::Theme;

// Tests for plan6 task 847.
const SOURCE: &str = include_str!("app_basic.rs");

fn source_declares_symbol(name: &str) -> bool {
    let needles = [
        format!("fn {name}"),
        format!("pub fn {name}"),
        format!("struct {name}"),
        format!("pub struct {name}"),
        format!("enum {name}"),
        format!("pub enum {name}"),
        format!("type {name}"),
        format!("pub type {name}"),
        format!("const {name}"),
        format!("pub const {name}"),
        format!("static {name}"),
        format!("pub static {name}"),
        format!("impl {name}"),
    ];

    needles.iter().any(|needle| SOURCE.contains(needle))
}

#[test]
fn app_basic_tests_keeps_planned_coverage_targets() {
    for name in [
        "normalize_file_search_query",
        "is_diff_file_expanded",
        "toggle_diff_file_expanded",
        "ensure_diff_file_expanded",
        "replace_expanded_files",
        "clear_expanded_files",
        "set_single_expanded_file",
        "show_error_toast",
        "is_file_tree_dir_expanded",
        "toggle_file_tree_dir_expanded",
        "ensure_file_tree_dir_expanded",
    ] {
        assert!(source_declares_symbol(name), "expected source to declare coverage target {name}");
    }
}

fn new_app() -> App {
    App::new().0
}

#[test]
fn normalize_file_search_query_trims_slashes_and_case() {
    assert_eq!(App::normalize_file_search_query("  Src\\Main.RS  "), "src/main.rs");
}

#[test]
fn diff_file_expansion_helpers_keep_order_and_set_in_sync() {
    let mut app = new_app();

    assert!(!app.is_diff_file_expanded("src/lib.rs"));

    app.ensure_diff_file_expanded("src/lib.rs".to_string());
    app.ensure_diff_file_expanded("src/lib.rs".to_string());
    assert_eq!(app.expanded_files, ["src/lib.rs"]);
    assert!(app.is_diff_file_expanded("src/lib.rs"));

    app.toggle_diff_file_expanded("src/lib.rs".to_string());
    assert!(app.expanded_files.is_empty());
    assert!(!app.is_diff_file_expanded("src/lib.rs"));

    app.toggle_diff_file_expanded("src/main.rs".to_string());
    app.replace_expanded_files(vec!["README.md".to_string(), "src/main.rs".to_string()]);
    assert_eq!(app.expanded_files, ["README.md", "src/main.rs"]);
    assert!(app.is_diff_file_expanded("README.md"));

    app.set_single_expanded_file("Cargo.toml".to_string());
    assert_eq!(app.expanded_files, ["Cargo.toml"]);
    assert!(app.is_diff_file_expanded("Cargo.toml"));
    assert!(!app.is_diff_file_expanded("README.md"));

    app.clear_expanded_files();
    assert!(app.expanded_files.is_empty());
    assert!(app.expanded_files_set.is_empty());
}

#[test]
fn file_tree_expansion_helpers_toggle_and_deduplicate() {
    let mut app = new_app();
    app.file_tree_expanded.clear();
    app.file_tree_expanded_set.clear();

    assert!(!app.is_file_tree_dir_expanded("src"));

    app.ensure_file_tree_dir_expanded("src".to_string());
    app.ensure_file_tree_dir_expanded("src".to_string());
    assert_eq!(app.file_tree_expanded, ["src"]);
    assert!(app.is_file_tree_dir_expanded("src"));

    app.toggle_file_tree_dir_expanded("src".to_string());
    assert!(!app.is_file_tree_dir_expanded("src"));
    assert!(app.file_tree_expanded.is_empty());

    app.toggle_file_tree_dir_expanded("crates".to_string());
    assert_eq!(app.file_tree_expanded, ["crates"]);
    assert!(app.is_file_tree_dir_expanded("crates"));
}

#[test]
fn title_covers_each_screen_and_pet_window_override() {
    let mut app = new_app();
    let expected = [
        (Screen::Home, "Vibe Window 氛围视窗 - 项目"),
        (Screen::Project, "Vibe Window 氛围视窗 - 项目"),
        (Screen::Design, "Vibe Window 氛围视窗 - 设计"),
        (Screen::Preview, "Vibe Window 氛围视窗 - 预览"),
        (Screen::Apps, "Vibe Window 氛围视窗 - 应用"),
        (Screen::Usage, "Vibe Window 氛围视窗 - 用量"),
        (Screen::JsonTool, "Vibe Window 氛围视窗 - JSON工具"),
        (Screen::JsonYamlTool, "Vibe Window 氛围视窗 - JSON/YAML互转工具"),
        (Screen::Knowledge, "Vibe Window 氛围视窗 - 知识库"),
        (Screen::SqlTool, "Vibe Window 氛围视窗 - SQL美化工具"),
        (Screen::RedisTool, "Vibe Window 氛围视窗 - Redis客户端"),
        (Screen::HtmlTool, "Vibe Window 氛围视窗 - HTML美化工具"),
        (Screen::JsonDiffTool, "Vibe Window 氛围视窗 - JSON比对工具"),
        (Screen::MarkdownTool, "Vibe Window 氛围视窗 - Markdown编辑器"),
        (Screen::WorkflowTool, "Vibe Window 氛围视窗 - 工作流"),
        (Screen::MindMapTool, "Vibe Window 氛围视窗 - 思维导图"),
        (Screen::PasswordTool, "Vibe Window 氛围视窗 - 随机密码生成器"),
        (Screen::BaseTool, "Vibe Window 氛围视窗 - 进制转换器"),
        (Screen::TimestampTool, "Vibe Window 氛围视窗 - 时间戳转换器"),
        (Screen::QrTool, "Vibe Window 氛围视窗 - 二维码生成器"),
        (Screen::ColorTool, "Vibe Window 氛围视窗 - 颜色转换工具"),
        (Screen::CleanerTool, "Vibe Window 氛围视窗 - 垃圾清理工具"),
        (Screen::LargeFileTool, "Vibe Window 氛围视窗 - 大文件查找工具"),
        (Screen::TaskBoard, "Vibe Window 氛围视窗 - 任务看板"),
    ];

    for (screen, title) in expected {
        app.screen = screen;
        assert_eq!(app.title(), title);
    }

    let pet_window = iced::window::Id::unique();
    let main_window = iced::window::Id::unique();
    app.task_pet_window_id = Some(pet_window);
    app.screen = Screen::Project;
    assert_eq!(app.title_for_window(pet_window), "VibeWindow Pet");
    assert_eq!(app.title_for_window(main_window), "Vibe Window 氛围视窗 - 项目");
}

#[test]
fn file_index_cache_tracks_project_results_and_tree_model() {
    let mut app = new_app();
    app.project_path = Some("/workspace".to_string());
    app.search_text = " SRC\\LIB ".to_string();
    app.file_search_query = "src".to_string();

    app.set_file_index(
        "/workspace",
        vec![
            "src/lib.rs".to_string(),
            "src/main.rs".to_string(),
            "README.md".to_string(),
            "tests/app_basic.rs".to_string(),
        ],
    );

    assert!(app.has_file_index("/workspace"));
    assert_eq!(
        app.current_file_index(),
        ["src/lib.rs", "src/main.rs", "README.md", "tests/app_basic.rs"]
    );
    assert!(app.current_file_tree_model().is_some());
    assert_eq!(app.cached_search_panel_file_results(), ["src/lib.rs"]);
    assert!(app.cached_file_search_entries().iter().any(|entry| entry.path == "src/"));
    assert!(app.cached_file_search_entries().iter().any(|entry| entry.path == "src/lib.rs"));

    let cached_results = app.cached_search_panel_file_results().to_vec();
    app.refresh_search_panel_file_cache();
    assert_eq!(app.cached_search_panel_file_results(), cached_results);

    app.project_path = Some("/other".to_string());
    assert!(app.current_file_index().is_empty());
    assert!(app.current_file_tree_model().is_none());
}

#[test]
fn search_panel_file_cache_limits_to_first_eight_matches() {
    let mut app = new_app();
    app.project_path = Some("/workspace".to_string());
    app.search_text = "src".to_string();
    app.set_file_index(
        "/workspace",
        (0..10).map(|idx| format!("src/file_{idx}.rs")).collect::<Vec<_>>(),
    );

    assert_eq!(app.cached_search_panel_file_results().len(), 8);
    assert_eq!(app.cached_search_panel_file_results()[0], "src/file_0.rs");
    assert_eq!(app.cached_search_panel_file_results()[7], "src/file_7.rs");
}

#[test]
fn can_open_external_defaults_false_and_uses_cached_value() {
    let mut app = new_app();
    app.open_external_exists.clear();

    assert!(!app.can_open_external(ExternalOpenApp::VSCode));

    app.open_external_exists.insert(ExternalOpenApp::VSCode, true);
    app.open_external_exists.insert(ExternalOpenApp::Cursor, false);
    assert!(app.can_open_external(ExternalOpenApp::VSCode));
    assert!(!app.can_open_external(ExternalOpenApp::Cursor));
}

#[test]
fn push_notification_assigns_incrementing_ids() {
    let mut app = new_app();
    app.notifications.clear();
    app.next_notification_id = 7;

    app.push_notification("first".to_string());
    app.push_notification("second".to_string());

    assert_eq!(app.notifications.len(), 2);
    assert_eq!(app.notifications[0].id, 7);
    assert_eq!(app.notifications[0].message, "first");
    assert_eq!(app.notifications[1].id, 8);
    assert_eq!(app.notifications[1].message, "second");
    assert_eq!(app.next_notification_id, 9);
}

#[test]
fn toast_helpers_store_kind_message_and_increment_id() {
    let mut app = new_app();
    app.next_toast_id = 3;

    let _ = app.show_success_toast("saved");
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.id), Some(3));
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.kind), Some(ToastKind::Success));
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.message.as_str()), Some("saved"));

    let _ = app.show_error_toast("failed");
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.id), Some(4));
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.kind), Some(ToastKind::Error));

    let _ = app.show_info_toast("noted");
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.id), Some(5));
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.kind), Some(ToastKind::Info));

    let _ = app.show_warning_toast("careful");
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.id), Some(6));
    assert_eq!(app.active_toast.as_ref().map(|toast| toast.kind), Some(ToastKind::Warning));
    assert_eq!(app.next_toast_id, 7);
}

#[test]
fn active_design_state_uses_active_tab_id() {
    let mut app = new_app();
    let tab_id = "design-tab".to_string();

    assert!(app.active_design_state().is_none());
    assert!(app.active_design_state_mut().is_none());

    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(DesignDoc::default()));

    assert!(app.active_design_state().is_some());
    app.active_design_state_mut().unwrap().zoom = 1.5;
    assert_eq!(app.active_design_state().map(|state| state.zoom), Some(1.5));
}

#[test]
fn theme_helpers_select_app_editor_and_pet_themes() {
    let mut app = new_app();
    app.app_theme = Theme::Dark;
    app.editor_theme = Theme::Light;

    assert_eq!(app.theme().to_string(), Theme::Dark.to_string());

    app.editor_follow_system_theme = true;
    assert_eq!(app.effective_editor_theme().to_string(), Theme::Dark.to_string());
    assert_eq!(app.effective_editor_theme_ref().to_string(), Theme::Dark.to_string());

    app.editor_follow_system_theme = false;
    assert_eq!(app.effective_editor_theme().to_string(), Theme::Light.to_string());
    assert_eq!(app.effective_editor_theme_ref().to_string(), Theme::Light.to_string());

    let pet_window = iced::window::Id::unique();
    let main_window = iced::window::Id::unique();
    app.task_pet_window_id = Some(pet_window);
    assert_eq!(app.theme_for_window(main_window).to_string(), Theme::Dark.to_string());
    assert_eq!(app.theme_for_window(pet_window).to_string(), "VibeWindow Pet");
}
