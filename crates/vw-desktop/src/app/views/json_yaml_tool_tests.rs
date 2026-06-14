use super::*;
use iced::widget::text_editor;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("json_yaml_tool_tests"));
}

#[test]
fn view_covers_idle_loading_and_notification_statuses() {
    let mut app = test_app();

    keep_element(view(&app));

    app.json_yaml_loading = true;
    keep_element(view(&app));

    app.json_yaml_loading = false;
    app.json_yaml_notification = Some("转换成功".to_string());
    keep_element(view(&app));
}

#[test]
fn workspace_covers_wide_and_narrow_layouts() {
    let app = test_app();

    keep_element(build_workspace(&app, Size::new(1280.0, 720.0)));
    keep_element(build_workspace(&app, Size::new(900.0, 720.0)));
}

#[test]
fn controls_cover_enabled_and_loading_buttons() {
    let mut app = test_app();

    keep_element(build_controls_panel(&app));

    app.json_yaml_loading = true;
    keep_element(build_controls_panel(&app));
}

#[test]
fn action_button_covers_primary_secondary_enabled_and_disabled() {
    let mut app = test_app();

    keep_element(build_action_button(
        &app,
        "YAML->JSON",
        JsonYamlToolMessage::YamlToJson,
        true,
        true,
    ));
    keep_element(build_action_button(
        &app,
        "复制左侧",
        JsonYamlToolMessage::CopyLeft,
        false,
        false,
    ));

    app.json_yaml_loading = true;
    keep_element(build_action_button(
        &app,
        "JSON->YAML",
        JsonYamlToolMessage::JsonToYaml,
        false,
        true,
    ));
}

#[test]
fn editor_workspace_covers_horizontal_and_vertical_layouts() {
    let app = test_app();

    keep_element(build_editor_workspace(&app, Size::new(1100.0, 640.0)));
    keep_element(build_editor_workspace(&app, Size::new(820.0, 640.0)));
}

#[test]
fn editor_cards_cover_both_sides_with_content() {
    let mut app = test_app();
    app.json_yaml_left_editor = text_editor::Content::with_text("name: vibe\nactive: true");
    app.json_yaml_right_editor =
        text_editor::Content::with_text("{\n  \"name\": \"vibe\",\n  \"active\": true\n}");

    keep_element(build_editor_card(&app, Size::new(720.0, 480.0), EditorSide::Left));
    keep_element(build_editor_card(&app, Size::new(720.0, 480.0), EditorSide::Right));
}

#[test]
fn editor_panels_cover_both_sides_and_context_menu_state() {
    let mut app = test_app();
    app.json_yaml_left_context_menu_open = true;
    app.json_yaml_left_context_menu_pos = Some((24.0, 36.0));
    app.json_yaml_right_context_menu_open = true;
    app.json_yaml_right_context_menu_pos = Some((48.0, 60.0));
    app.json_yaml_left_scroll_top_line = 2.0;
    app.json_yaml_right_scroll_top_line = 3.0;

    keep_element(build_editor_panel(&app, Size::new(720.0, 480.0), EditorSide::Left));
    keep_element(build_editor_panel(&app, Size::new(720.0, 480.0), EditorSide::Right));
}

#[test]
fn small_builders_return_elements() {
    let mut app = test_app();

    keep_element(build_section_title("编辑区"));
    keep_element(build_metric_badge("3 行".to_string()));
    keep_element(build_status_badge(&app));

    app.json_yaml_loading = true;
    keep_element(build_status_badge(&app));

    app.json_yaml_loading = false;
    app.json_yaml_notification = Some("已复制".to_string());
    keep_element(build_status_badge(&app));
}

#[test]
fn editor_style_is_theme_aware_for_all_statuses() {
    for theme in [Theme::Light, Theme::Dark] {
        for status in [
            iced::widget::text_editor::Status::Active,
            iced::widget::text_editor::Status::Hovered,
            iced::widget::text_editor::Status::Focused { is_hovered: false },
            iced::widget::text_editor::Status::Disabled,
        ] {
            let style = editor_style(&theme, status);

            assert_eq!(style.border.width, 0.0);
            assert_eq!(style.border.color, Color::TRANSPARENT);
            assert_eq!(style.value, theme.palette().text);
            assert_eq!(style.placeholder, theme.palette().text.scale_alpha(0.55));
        }
    }
}
