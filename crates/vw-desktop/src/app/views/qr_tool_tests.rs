use crate::app::message::qr_tool::{QrEcLevel, QrIconMode, QrToolMessage};
use crate::app::{App, Message};
use iced::widget::text_editor;
use iced::{Color, Size, Theme};

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

fn keep_element(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("qr_tool_tests"));
}

#[test]
fn view_builds_base_and_color_picker_overlay() {
    let mut app = test_app();

    keep_element(super::view(&app));

    app.show_qr_color_picker = true;
    app.qr_color_hex = "#336699cc".to_string();
    app.window_size = (1280.0, 800.0);

    keep_element(super::view(&app));
    keep_element(super::build_color_picker_drawer(&app));

    app.window_size = (400.0, 800.0);
    keep_element(super::build_color_picker_drawer(&app));

    app.window_size = (4000.0, 800.0);
    keep_element(super::build_color_picker_drawer(&app));
}

#[test]
fn workspace_uses_wide_and_narrow_layouts() {
    let app = test_app();

    keep_element(super::build_workspace(&app, Size::new(1200.0, 720.0)));
    keep_element(super::build_workspace(&app, Size::new(640.0, 720.0)));
}

#[test]
fn controls_panel_covers_icon_modes_and_picker_state() {
    let mut app = test_app();

    for mode in [QrIconMode::None, QrIconMode::Default, QrIconMode::Upload] {
        app.qr_icon_mode = mode;
        app.qr_icon_bytes = None;
        keep_element(super::build_controls_panel(&app));

        app.qr_icon_bytes = Some(vec![1, 2, 3]);
        keep_element(super::build_controls_panel(&app));
    }

    app.show_qr_color_picker = true;
    app.qr_color_hex = "invalid-color".to_string();
    keep_element(super::build_controls_panel(&app));
}

#[test]
fn editor_and_preview_cards_cover_empty_loading_and_image_states() {
    let mut app = test_app();
    app.qr_editor = text_editor::Content::with_text("https://example.com\nsecond line");
    app.current_line_height = 20.0;
    app.qr_scroll_top_line = 1.0;

    keep_element(super::build_editor_card(&app, Size::new(900.0, 600.0)));
    keep_element(super::build_editor_panel(&app, Size::new(900.0, 600.0)));
    keep_element(super::build_preview_card(&app));

    app.qr_loading = true;
    keep_element(super::build_preview_card(&app));

    app.qr_loading = false;
    app.qr_image = Some(iced::widget::image::Handle::from_bytes(vec![1, 2, 3, 4]));
    keep_element(super::build_preview_card(&app));
}

#[test]
fn status_badge_covers_idle_loading_success_and_error() {
    let mut app = test_app();

    keep_element(super::build_status_badge(&app));

    app.qr_loading = true;
    keep_element(super::build_status_badge(&app));

    app.qr_loading = false;
    app.qr_notification = Some("生成成功".to_string());
    app.qr_notification_is_error = false;
    keep_element(super::build_status_badge(&app));

    app.qr_notification = Some("请输入二维码内容".to_string());
    app.qr_notification_is_error = true;
    keep_element(super::build_status_badge(&app));
}

#[test]
fn small_builders_return_expected_variants() {
    assert_eq!(
        super::qr_icon_mode_description(QrIconMode::None),
        "保持标准二维码结构，兼容性最佳。"
    );
    assert_eq!(
        super::qr_icon_mode_description(QrIconMode::Default),
        "使用内置 Logo，适合快速生成品牌二维码。"
    );
    assert_eq!(
        super::qr_icon_mode_description(QrIconMode::Upload),
        "建议上传透明底 PNG，中心区域会自动留白。"
    );

    let app = test_app();
    keep_element(super::build_color_swatch_button(Color::from_rgb8(1, 2, 3)));
    keep_element(super::build_metric_badge("256 px".to_string()));
    keep_element(super::build_form_row(
        "尺寸",
        "输出 PNG 的边长。",
        iced::widget::text("256"),
    ));
    keep_element(super::build_action_button(&app, "生成二维码", QrToolMessage::Generate, true));
    keep_element(super::build_action_button(&app, "清空", QrToolMessage::Clear, false));

    let mut loading_app = test_app();
    loading_app.qr_loading = true;
    keep_element(super::build_action_button(
        &loading_app,
        "保存 PNG",
        QrToolMessage::SavePng,
        false,
    ));
}

#[test]
fn styles_have_backgrounds_and_borders() {
    for theme in [Theme::Light, Theme::Dark] {
        let style = super::preview_surface_style(&theme);
        assert!(style.background.is_some());
        assert!(style.border.width >= 1.0);
    }
}

#[test]
fn enum_display_values_match_user_labels() {
    assert_eq!(QrEcLevel::all(), [QrEcLevel::L, QrEcLevel::M, QrEcLevel::Q, QrEcLevel::H]);
    assert_eq!(QrIconMode::all(), [QrIconMode::None, QrIconMode::Default, QrIconMode::Upload]);
    assert_eq!(QrEcLevel::L.to_string(), "L");
    assert_eq!(QrIconMode::None.to_string(), "不添加");
}
