use super::*;
use crate::app::components::markdown_editor::MarkdownViewMode;
use iced::widget::button;
use iced::widget::markdown::Viewer;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

fn assert_background(style: iced::widget::container::Style) {
    assert!(style.background.is_some());
}

fn assert_markdown_message(message: Message, expected: fn(&MarkdownToolMessage) -> bool) {
    match message {
        Message::MarkdownTool(message) => assert!(expected(&message)),
        other => panic!("expected markdown tool message, got {other:?}"),
    }
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("markdown_tool_tests"));
}

#[test]
fn editor_action_maps_to_markdown_message() {
    let message = on_editor_action(text_editor::Action::Move(text_editor::Motion::Left));

    assert_markdown_message(message, |message| {
        matches!(message, MarkdownToolMessage::EditorAction(_))
    });
}

#[test]
fn mode_change_maps_to_markdown_message() {
    let message = on_mode_change(MarkdownViewMode::Preview);

    assert_markdown_message(message, |message| {
        matches!(message, MarkdownToolMessage::SetViewMode(MarkdownViewMode::Preview))
    });
}

#[test]
fn theme_helpers_distinguish_light_and_dark() {
    assert!(!is_dark_theme(&Theme::Light));
    assert!(is_dark_theme(&Theme::Dark));
    assert_ne!(danger_color(&Theme::Light), danger_color(&Theme::Dark));
}

#[test]
fn container_styles_have_backgrounds_and_borders() {
    for theme in [Theme::Light, Theme::Dark] {
        for style in
            [chrome_chip_style(&theme), editor_surface_style(&theme), tooltip_card_style(&theme)]
        {
            assert_background(style);
            assert!(style.border.width >= 1.0);
        }
    }
}

#[test]
fn toolbar_icon_color_uses_tone_palette() {
    for theme in [Theme::Light, Theme::Dark] {
        let default = toolbar_icon_color(&theme, MarkdownActionTone::Default);
        let primary = toolbar_icon_color(&theme, MarkdownActionTone::Primary);
        let success = toolbar_icon_color(&theme, MarkdownActionTone::Success);
        let danger = toolbar_icon_color(&theme, MarkdownActionTone::Danger);

        assert_ne!(default, primary);
        assert_ne!(primary, success);
        assert_ne!(success, danger);
    }
}

#[test]
fn toolbar_button_style_keeps_default_and_accents_other_tones() {
    for theme in [Theme::Light, Theme::Dark] {
        for status in [
            button::Status::Active,
            button::Status::Hovered,
            button::Status::Pressed,
            button::Status::Disabled,
        ] {
            let default = toolbar_button_style(&theme, status, MarkdownActionTone::Default);
            let primary = toolbar_button_style(&theme, status, MarkdownActionTone::Primary);
            let success = toolbar_button_style(&theme, status, MarkdownActionTone::Success);
            let danger = toolbar_button_style(&theme, status, MarkdownActionTone::Danger);

            assert_ne!(default.border.color, primary.border.color);
            assert!(primary.background.is_some());
            assert!(success.background.is_some());
            assert!(danger.background.is_some());
        }
    }
}

#[test]
fn toolbar_button_style_uses_distinct_pressed_hover_alphas() {
    for theme in [Theme::Light, Theme::Dark] {
        let active = toolbar_button_style(&theme, button::Status::Active, MarkdownActionTone::Primary);
        let hovered =
            toolbar_button_style(&theme, button::Status::Hovered, MarkdownActionTone::Primary);
        let pressed =
            toolbar_button_style(&theme, button::Status::Pressed, MarkdownActionTone::Primary);

        assert_ne!(active.background, hovered.background);
        assert_ne!(hovered.background, pressed.background);
    }
}

#[test]
fn badge_and_toolbar_builders_return_elements() {
    keep_element(build_metric_badge("3 行"));
    keep_element(build_status_badge("加载", MarkdownBadgeTone::Loading));
    keep_element(build_status_badge("成功", MarkdownBadgeTone::Success));
    keep_element(build_status_badge("空闲", MarkdownBadgeTone::Idle));
    keep_element(build_toolbar_button(
        crate::app::assets::Icon::Clipboard,
        "复制",
        MarkdownToolMessage::Copy,
        MarkdownActionTone::Default,
    ));
    keep_element(build_panel_card(
        "面板",
        build_metric_badge("状态"),
        build_status_badge("内容", MarkdownBadgeTone::Idle),
    ));
}

#[test]
fn viewer_link_click_opens_external_url() {
    let message = App::on_link_click("https://example.com/docs".to_string());

    match message {
        Message::View(ViewMessage::OpenUrlExternal(url)) => {
            assert_eq!(url, "https://example.com/docs");
        }
        other => panic!("expected external url message, got {other:?}"),
    }
}

#[test]
fn viewer_image_covers_remote_states_and_missing_file() {
    let mut app = test_app();
    let remote = "https://example.com/image.png".to_string();
    let insecure_remote = "http://example.com/image.png".to_string();

    keep_element(build_markdown_image(&app, remote.clone()));
    keep_element(build_markdown_image(&app, insecure_remote));

    app.markdown_tool_remote_images_loading.insert(remote.clone());
    keep_element(build_markdown_image(&app, remote.clone()));

    app.markdown_tool_remote_images_loading.clear();
    app.markdown_tool_remote_images
        .insert(remote.clone(), ImageHandle::from_bytes(vec![1, 2, 3, 4]));
    keep_element(build_markdown_image(&app, remote));

    let missing = "/tmp/vibe-window-missing-markdown-image.png".to_string();
    keep_element(build_markdown_image(&app, missing));
}

#[test]
fn viewer_image_covers_existing_file_uri_forms() {
    let app = test_app();
    let file_path = std::env::temp_dir().join("vibe-window-markdown-tool-test-image.bin");
    std::fs::write(&file_path, [1_u8, 2, 3, 4]).expect("write image fixture");

    let absolute = file_path.to_string_lossy().to_string();
    let file_three_slash = format!("file:///{absolute}");
    let file_two_slash = format!("file://{absolute}");

    keep_element(build_markdown_image(&app, absolute));
    keep_element(build_markdown_image(&app, file_three_slash));
    keep_element(build_markdown_image(&app, file_two_slash));

    let _ = std::fs::remove_file(file_path);
}

#[test]
fn body_builder_covers_all_view_modes() {
    for mode in [MarkdownViewMode::Edit, MarkdownViewMode::Preview, MarkdownViewMode::Split] {
        let mut app = test_app();
        app.markdown_tool_view_mode = mode;
        app.markdown_tool_stream_enabled = mode == MarkdownViewMode::Preview;
        app.markdown_tool_editor = text_editor::Content::with_text("# 标题\n\n正文");
        app.markdown_tool_content = markdown::Content::parse(&app.markdown_tool_editor.text());
        app.markdown_tool_context_menu_open = true;
        app.markdown_tool_context_menu_pos = Some((18.0, 28.0));

        keep_element(build_body(&app, &Theme::Light, on_editor_action));
        keep_element(build_editor_panel(&app, Size::new(720.0, 480.0), on_editor_action));
    }
}

#[test]
fn view_covers_status_and_modal_branches() {
    let mut app = test_app();
    app.markdown_tool_view_mode = MarkdownViewMode::Edit;
    keep_element(view(&app));

    app.markdown_tool_view_mode = MarkdownViewMode::Preview;
    app.markdown_tool_notification = Some("已复制".to_string());
    keep_element(view(&app));

    app.markdown_tool_view_mode = MarkdownViewMode::Split;
    app.markdown_tool_remote_images_loading.insert("https://example.com/a.png".to_string());
    keep_element(view(&app));

    app.markdown_tool_remote_images_loading.clear();
    app.markdown_tool_show_html2md = true;
    keep_element(view(&app));

    app.markdown_tool_show_html2md = false;
    app.markdown_tool_show_image = true;
    app.markdown_tool_image_url_input = "https://example.com/a.png".to_string();
    keep_element(view(&app));
}
