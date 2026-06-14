//! 覆盖二维码工具消息处理、参数校验和 PNG 渲染行为。
//!
//! 测试保持在独立文件中，直接验证当前公开状态变更和局部渲染 helper。

use super::{
    QrEcLevel, QrIconMode, QrRenderRequest, QrSaveOutcome, QrToolMessage, build_render_request,
    clear_notification_task, max_scroll_top_line, notify_error, notify_success, overlay_icon,
    parse_qr_size_input, render_qr_png, update,
    visible_line_count,
};
use crate::app::App;
use iced::mouse;
use iced::widget::text_editor;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

fn apply_update(app: &mut App, message: QrToolMessage) {
    let _ = update(app, message);
}

fn tiny_png_bytes() -> Vec<u8> {
    let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .expect("encode png fixture");
    bytes
}

#[test]
fn parse_qr_size_input_accepts_bounds_and_rejects_invalid_values() {
    assert_eq!(parse_qr_size_input("64"), Ok(64));
    assert_eq!(parse_qr_size_input(" 2048 "), Ok(2048));
    assert_eq!(parse_qr_size_input(""), Err("请输入二维码尺寸".to_string()));
    assert_eq!(parse_qr_size_input("abc"), Err("二维码尺寸必须是数字".to_string()));
    assert_eq!(parse_qr_size_input("63"), Err("二维码尺寸需在 64-2048px 之间".to_string()));
    assert_eq!(parse_qr_size_input("2049"), Err("二维码尺寸需在 64-2048px 之间".to_string()));
}

#[test]
fn qr_level_and_icon_mode_display_all_user_labels() {
    assert_eq!(QrEcLevel::all(), [QrEcLevel::L, QrEcLevel::M, QrEcLevel::Q, QrEcLevel::H]);
    assert_eq!(QrEcLevel::L.to_string(), "L");
    assert_eq!(QrEcLevel::M.to_string(), "M");
    assert_eq!(QrEcLevel::Q.to_string(), "Q");
    assert_eq!(QrEcLevel::H.to_string(), "H");

    assert_eq!(QrIconMode::all(), [QrIconMode::None, QrIconMode::Default, QrIconMode::Upload]);
    assert_eq!(QrIconMode::None.to_string(), "不添加");
    assert_eq!(QrIconMode::Default.to_string(), "默认 Logo");
    assert_eq!(QrIconMode::Upload.to_string(), "上传图片");
}

#[test]
fn build_render_request_validates_content_size_and_icon_mode() {
    let mut app = test_app();

    assert_eq!(build_render_request(&mut app).unwrap_err(), "请输入二维码内容");

    app.qr_editor = text_editor::Content::with_text("https://example.com");
    app.qr_size_input = "128".to_string();
    let request = build_render_request(&mut app).expect("valid request");
    assert_eq!(request.data, "https://example.com");
    assert_eq!(request.size, 128);
    assert_eq!(app.qr_size, 128);
    assert_eq!(app.qr_size_input, "128");
    assert!(request.icon_bytes.is_none());

    app.qr_icon_mode = QrIconMode::Upload;
    app.qr_icon_bytes = None;
    assert_eq!(build_render_request(&mut app).unwrap_err(), "请先选择上传图标");

    app.qr_icon_bytes = Some(vec![1, 2, 3]);
    let upload_request = build_render_request(&mut app).expect("upload request");
    assert_eq!(upload_request.icon_bytes, Some(vec![1, 2, 3]));

    app.qr_icon_mode = QrIconMode::Default;
    let default_request = build_render_request(&mut app).expect("default logo request");
    assert!(default_request.icon_bytes.expect("logo bytes").len() > 100);
}

#[test]
fn build_render_request_normalizes_whitespace_size_and_preserves_data() {
    let mut app = test_app();
    app.qr_editor = text_editor::Content::with_text("  line one\nline two  ");
    app.qr_size_input = " 256 ".to_string();
    app.qr_color_hex = "#008877".to_string();
    app.qr_level = QrEcLevel::M;

    let request = build_render_request(&mut app).expect("valid request");

    assert_eq!(request.data, "  line one\nline two  ");
    assert_eq!(request.size, 256);
    assert_eq!(request.level, QrEcLevel::M);
    assert_eq!(request.color_hex, "#008877");
    assert_eq!(app.qr_size_input, "256");
}

#[test]
fn render_qr_png_outputs_decodable_png_for_levels_and_color_fallback() {
    for level in QrEcLevel::all() {
        let request = QrRenderRequest {
            data: "https://example.com".to_string(),
            size: 128,
            level,
            color_hex: if level == QrEcLevel::H { "not-a-color" } else { "#123456" }.to_string(),
            icon_bytes: None,
        };

        let png = render_qr_png(&request).expect("qr png");
        let decoded = image::load_from_memory(&png).expect("decode qr png");
        assert_eq!(decoded.color(), image::ColorType::Rgba8);
        assert!(decoded.width() >= 64);
        assert_eq!(decoded.width(), decoded.height());
    }
}

#[test]
fn render_qr_png_rejects_oversized_content() {
    let request = QrRenderRequest {
        data: "x".repeat(10_000),
        size: 64,
        level: QrEcLevel::H,
        color_hex: "#000000".to_string(),
        icon_bytes: None,
    };

    assert_eq!(render_qr_png(&request), Err("二维码内容过长或当前纠错等级不支持".to_string()));
}

#[test]
fn render_qr_png_overlays_valid_icon_and_rejects_invalid_icon() {
    let request = QrRenderRequest {
        data: "with icon".to_string(),
        size: 128,
        level: QrEcLevel::Q,
        color_hex: "#000000".to_string(),
        icon_bytes: Some(tiny_png_bytes()),
    };
    let png = render_qr_png(&request).expect("qr png with icon");
    assert!(image::load_from_memory(&png).is_ok());

    let invalid = QrRenderRequest { icon_bytes: Some(vec![0, 1, 2]), ..request };
    assert_eq!(render_qr_png(&invalid).unwrap_err(), "无法读取图标图片");
}

#[test]
fn overlay_icon_blends_center_region_and_rejects_bad_bytes() {
    let side = 32;
    let mut rgba = vec![255_u8; side * side * 4];

    overlay_icon(&mut rgba, side as u32, 2, &tiny_png_bytes()).expect("overlay icon");

    assert!(rgba.chunks_exact(4).any(|pixel| pixel[0] == 255 && pixel[1] == 0 && pixel[2] == 0));
    assert_eq!(
        overlay_icon(&mut rgba, side as u32, 2, &[1, 2, 3]).unwrap_err(),
        "无法读取图标图片"
    );
}

#[test]
fn notify_helpers_set_success_error_and_clear_task_is_constructible() {
    let mut app = test_app();

    notify_success(&mut app, "ok");
    assert_eq!(app.qr_notification.as_deref(), Some("ok"));
    assert!(!app.qr_notification_is_error);

    notify_error(&mut app, "bad");
    assert_eq!(app.qr_notification.as_deref(), Some("bad"));
    assert!(app.qr_notification_is_error);

    let _task = clear_notification_task();
}

#[test]
fn update_mutates_simple_fields_and_result_states() {
    let mut app = test_app();

    apply_update(&mut app, QrToolMessage::ColorChanged("#abcdef".to_string()));
    assert_eq!(app.qr_color_hex, "#abcdef");

    apply_update(&mut app, QrToolMessage::ToggleColorPicker);
    assert!(app.show_qr_color_picker);

    apply_update(&mut app, QrToolMessage::SizeChanged("512".to_string()));
    assert_eq!(app.qr_size_input, "512");
    assert_eq!(app.qr_size, 512);

    apply_update(&mut app, QrToolMessage::SizeChanged("bad".to_string()));
    assert_eq!(app.qr_size, 512);

    apply_update(&mut app, QrToolMessage::LevelSelected(QrEcLevel::H));
    apply_update(&mut app, QrToolMessage::IconModeSelected(QrIconMode::Upload));
    assert_eq!(app.qr_level, QrEcLevel::H);
    assert_eq!(app.qr_icon_mode, QrIconMode::Upload);

    apply_update(&mut app, QrToolMessage::IconLoaded(Some(vec![9, 8, 7])));
    assert_eq!(app.qr_icon_bytes, Some(vec![9, 8, 7]));
    assert_eq!(app.qr_notification.as_deref(), Some("已更新图标"));

    apply_update(&mut app, QrToolMessage::IconLoaded(None));
    assert_eq!(app.qr_icon_bytes, Some(vec![9, 8, 7]));

    apply_update(&mut app, QrToolMessage::ClearNotification);
    assert!(app.qr_notification.is_none());
    assert!(!app.qr_notification_is_error);
}

#[test]
fn update_handles_generate_validation_clear_and_async_results() {
    let mut app = test_app();

    apply_update(&mut app, QrToolMessage::Generate);
    assert_eq!(app.qr_notification.as_deref(), Some("请输入二维码内容"));
    assert!(app.qr_notification_is_error);

    app.qr_editor = text_editor::Content::with_text("hello");
    app.qr_size_input = "bad".to_string();
    apply_update(&mut app, QrToolMessage::SavePng);
    assert_eq!(app.qr_notification.as_deref(), Some("二维码尺寸必须是数字"));

    app.qr_size_input = "128".to_string();
    apply_update(&mut app, QrToolMessage::Generate);
    assert!(app.qr_loading);
    assert!(app.qr_notification.is_none());
    assert!(!app.qr_notification_is_error);

    apply_update(&mut app, QrToolMessage::Generated(Err("渲染失败".to_string())));
    assert!(!app.qr_loading);
    assert_eq!(app.qr_notification.as_deref(), Some("渲染失败"));
    assert!(app.qr_notification_is_error);

    apply_update(&mut app, QrToolMessage::Generated(Ok(tiny_png_bytes())));
    assert!(app.qr_image.is_some());
    assert_eq!(app.qr_notification.as_deref(), Some("生成成功"));
    assert!(!app.qr_notification_is_error);

    apply_update(&mut app, QrToolMessage::Saved(Ok(QrSaveOutcome::Saved)));
    assert_eq!(app.qr_notification.as_deref(), Some("已保存 PNG"));

    apply_update(&mut app, QrToolMessage::Saved(Ok(QrSaveOutcome::Cancelled)));
    assert_eq!(app.qr_notification.as_deref(), Some("已取消保存"));

    apply_update(&mut app, QrToolMessage::Saved(Err("保存失败".to_string())));
    assert_eq!(app.qr_notification.as_deref(), Some("保存失败"));
    assert!(app.qr_notification_is_error);

    apply_update(&mut app, QrToolMessage::Clear);
    assert_eq!(app.qr_editor.text(), "");
    assert!(app.qr_image.is_none());
    assert_eq!(app.qr_scroll_top_line, 0.0);
    assert_eq!(app.qr_scroll_remainder, 0.0);
    assert_eq!(app.qr_notification.as_deref(), Some("已清空"));
}

#[test]
fn scroll_helpers_clamp_wheel_and_scrollbar_updates() {
    let mut app = test_app();
    app.qr_editor = text_editor::Content::with_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
    app.current_line_height = 10.0;
    app.qr_viewport_height = 30.0;

    assert_eq!(visible_line_count(&app), 3.0);
    assert_eq!(max_scroll_top_line(&app), 7.0);

    apply_update(
        &mut app,
        QrToolMessage::EditorWheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -4.0 },
            viewport_height: 30.0,
        },
    );
    assert!(app.qr_scroll_top_line > 0.0);

    apply_update(
        &mut app,
        QrToolMessage::EditorWheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -15.0 },
            viewport_height: 30.0,
        },
    );
    assert!(app.qr_scroll_top_line <= 7.0);

    apply_update(
        &mut app,
        QrToolMessage::ScrollbarChanged { top_line: 99.0, viewport_height: 30.0 },
    );
    assert_eq!(app.qr_scroll_top_line, 7.0);

    apply_update(&mut app, QrToolMessage::EditorAction(text_editor::Action::Scroll { lines: -99 }));
    assert_eq!(app.qr_scroll_top_line, 0.0);
}

#[test]
fn scroll_helpers_handle_zero_viewport_short_content_and_remainder() {
    let mut app = test_app();
    app.qr_editor = text_editor::Content::with_text("only one line");
    app.current_line_height = 0.0;
    app.qr_viewport_height = 0.0;

    assert_eq!(visible_line_count(&app), 1.0);
    assert_eq!(max_scroll_top_line(&app), 0.0);

    apply_update(
        &mut app,
        QrToolMessage::EditorWheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x: 0.0, y: -0.5 },
            viewport_height: -10.0,
        },
    );
    assert_eq!(app.qr_viewport_height, 0.0);
    assert_eq!(app.qr_scroll_top_line, 0.0);

    apply_update(
        &mut app,
        QrToolMessage::ScrollbarChanged { top_line: -50.0, viewport_height: -1.0 },
    );
    assert_eq!(app.qr_scroll_top_line, 0.0);
}
