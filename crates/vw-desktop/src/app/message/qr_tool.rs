//! 处理二维码工具的编辑、生成、图标加载和保存流程。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::views::design::properties::color_picker::parse_color;
use crate::app::{App, Message};
use iced::mouse;
use iced::widget::text_editor;
use iced::{Color, Task};

const QR_MIN_SIZE: u32 = 64;
const QR_MAX_SIZE: u32 = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// QrEcLevel 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub enum QrEcLevel {
    L,
    M,
    Q,
    H,
}

impl QrEcLevel {
    /// all 处理当前模块对应的消息或状态转换。
    ///
    /// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
    /// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
    pub fn all() -> [QrEcLevel; 4] {
        [QrEcLevel::L, QrEcLevel::M, QrEcLevel::Q, QrEcLevel::H]
    }
}

impl std::fmt::Display for QrEcLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QrEcLevel::L => write!(f, "L"),
            QrEcLevel::M => write!(f, "M"),
            QrEcLevel::Q => write!(f, "Q"),
            QrEcLevel::H => write!(f, "H"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// QrIconMode 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub enum QrIconMode {
    None,
    Default,
    Upload,
}

impl QrIconMode {
    /// all 处理当前模块对应的消息或状态转换。
    ///
    /// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
    /// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
    pub fn all() -> [QrIconMode; 3] {
        [QrIconMode::None, QrIconMode::Default, QrIconMode::Upload]
    }
}

impl std::fmt::Display for QrIconMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QrIconMode::None => write!(f, "不添加"),
            QrIconMode::Default => write!(f, "默认 Logo"),
            QrIconMode::Upload => write!(f, "上传图片"),
        }
    }
}

#[derive(Debug, Clone)]
/// QrToolMessage 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub enum QrToolMessage {
    EditorAction(text_editor::Action),
    EditorWheelScrolled {
        delta: mouse::ScrollDelta,
        viewport_height: f32,
    },
    ScrollbarChanged {
        top_line: f32,
        viewport_height: f32,
    },
    SizeChanged(String),
    ColorChanged(String),
    ColorFormatChanged(crate::app::views::design::models::ColorFormat),
    ToggleColorPicker,
    LevelSelected(QrEcLevel),
    IconModeSelected(QrIconMode),
    PickUploadedIcon,
    IconLoaded(Option<Vec<u8>>),
    Generate,
    SavePng,
    Clear,
    ClearNotification,
    Generated(Result<Vec<u8>, String>),
    Saved(Result<QrSaveOutcome, String>),
}

#[derive(Debug, Clone)]
/// QrSaveOutcome 表示该流程中可枚举的状态或用户动作。
///
/// 变体与界面事件或后台任务结果保持对应，便于在消息分发时显式匹配。
pub enum QrSaveOutcome {
    Saved,
    Cancelled,
}

#[derive(Debug, Clone)]
struct QrRenderRequest {
    data: String,
    size: u32,
    level: QrEcLevel,
    color_hex: String,
    icon_bytes: Option<Vec<u8>>,
}

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub fn update(app: &mut App, message: QrToolMessage) -> Task<Message> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        QrToolMessage::ClearNotification => {
            app.qr_notification = None;
            app.qr_notification_is_error = false;
            Task::none()
        }
        QrToolMessage::ColorChanged(value) => {
            app.qr_color_hex = value;
            Task::none()
        }
        QrToolMessage::ColorFormatChanged(format) => {
            app.qr_color_format = format;
            Task::none()
        }
        QrToolMessage::ToggleColorPicker => {
            app.show_qr_color_picker = !app.show_qr_color_picker;
            Task::none()
        }
        QrToolMessage::EditorAction(action) => {
            if let text_editor::Action::Scroll { lines } = &action {
                apply_scroll_lines(app, *lines);
            }
            app.qr_editor.perform(action);
            Task::none()
        }
        QrToolMessage::EditorWheelScrolled { delta, viewport_height } => {
            app.qr_viewport_height = viewport_height.max(0.0);

            let line_height = app.current_line_height.max(1.0);
            let delta_lines = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -y * 1.25,
                mouse::ScrollDelta::Pixels { y, .. } => -y / line_height,
            };

            app.qr_scroll_remainder += delta_lines;

            let whole_lines = if app.qr_scroll_remainder >= 0.0 {
                app.qr_scroll_remainder.floor() as i32
            } else {
                app.qr_scroll_remainder.ceil() as i32
            };

            if whole_lines != 0 {
                app.qr_scroll_remainder -= whole_lines as f32;
                apply_scroll_lines(app, whole_lines);
                app.qr_editor.perform(text_editor::Action::Scroll { lines: whole_lines });
            }

            Task::none()
        }
        QrToolMessage::ScrollbarChanged { top_line, viewport_height } => {
            app.qr_viewport_height = viewport_height.max(0.0);

            let max_scroll = max_scroll_top_line(app);
            let target_top_line = top_line.round().clamp(0.0, max_scroll);
            let current_top_line = app.qr_scroll_top_line.round();
            let delta = (target_top_line - current_top_line) as i32;

            if delta != 0 {
                apply_scroll_lines(app, delta);
                app.qr_editor.perform(text_editor::Action::Scroll { lines: delta });
            }

            Task::none()
        }
        QrToolMessage::SizeChanged(value) => {
            app.qr_size_input = value;
            if let Ok(size) = parse_qr_size_input(&app.qr_size_input) {
                app.qr_size = size;
            }
            Task::none()
        }
        QrToolMessage::LevelSelected(level) => {
            app.qr_level = level;
            Task::none()
        }
        QrToolMessage::IconModeSelected(mode) => {
            app.qr_icon_mode = mode;
            Task::none()
        }
        QrToolMessage::PickUploadedIcon => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use rfd::FileDialog;

                return Task::perform(
                    async move {
                        crate::app::message::spawn_blocking_opt(move || {
                            let path = FileDialog::new()
                                .set_title("选择图标图片")
                                .add_filter("图片", &["png", "jpg", "jpeg", "webp"])
                                .pick_file();
                            path.and_then(|selected| std::fs::read(selected).ok())
                        })
                        .await
                    },
                    |bytes| Message::QrTool(QrToolMessage::IconLoaded(bytes)),
                );
            }

            #[cfg(target_arch = "wasm32")]
            {
                notify_error(app, "当前平台暂不支持上传图标");
                Task::batch(vec![clear_notification_task()])
            }
        }
        QrToolMessage::IconLoaded(bytes) => {
            if let Some(bytes) = bytes {
                app.qr_icon_bytes = Some(bytes);
                notify_success(app, "已更新图标");
                return clear_notification_task();
            }

            Task::none()
        }
        QrToolMessage::Generate => {
            let request = match build_render_request(app) {
                Ok(request) => request,
                Err(error) => {
                    notify_error(app, &error);
                    return clear_notification_task();
                }
            };

            app.qr_loading = true;
            app.qr_notification = None;
            app.qr_notification_is_error = false;

            // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。

            Task::perform(
                async move {
                    crate::app::message::spawn_blocking_opt(move || Some(render_qr_png(&request)))
                        .await
                        .unwrap_or_else(|| Err("二维码生成失败".to_string()))
                },
                |result| Message::QrTool(QrToolMessage::Generated(result)),
            )
        }
        QrToolMessage::SavePng => {
            let request = match build_render_request(app) {
                Ok(request) => request,
                Err(error) => {
                    notify_error(app, &error);
                    return clear_notification_task();
                }
            };

            #[cfg(not(target_arch = "wasm32"))]
            {
                use rfd::FileDialog;

                app.qr_loading = true;
                app.qr_notification = None;
                app.qr_notification_is_error = false;

                return Task::perform(
                    async move {
                        crate::app::message::spawn_blocking_opt(move || {
                            Some((|| {
                                let path = FileDialog::new()
                                    .set_title("保存二维码")
                                    .add_filter("PNG", &["png"])
                                    .save_file();
                                let Some(path) = path else {
                                    return Ok(QrSaveOutcome::Cancelled);
                                };

                                let png_bytes = render_qr_png(&request)?;
                                std::fs::write(path, png_bytes)
                                    .map_err(|_| "保存 PNG 文件失败".to_string())?;
                                Ok(QrSaveOutcome::Saved)
                            })())
                        })
                        .await
                        .unwrap_or_else(|| Err("保存二维码失败".to_string()))
                    },
                    |result| Message::QrTool(QrToolMessage::Saved(result)),
                );
            }

            #[cfg(target_arch = "wasm32")]
            {
                let _ = request;
                notify_error(app, "当前平台暂不支持保存 PNG");
                clear_notification_task()
            }
        }
        QrToolMessage::Clear => {
            app.qr_editor = text_editor::Content::new();
            app.qr_image = None;
            app.qr_scroll_top_line = 0.0;
            app.qr_scroll_remainder = 0.0;
            notify_success(app, "已清空");
            clear_notification_task()
        }
        QrToolMessage::Generated(result) => {
            app.qr_loading = false;

            match result {
                Ok(png_bytes) => {
                    app.qr_image = Some(iced::widget::image::Handle::from_bytes(png_bytes));
                    notify_success(app, "生成成功");
                }
                Err(error) => {
                    notify_error(app, &error);
                }
            }

            clear_notification_task()
        }
        QrToolMessage::Saved(result) => {
            app.qr_loading = false;

            match result {
                Ok(QrSaveOutcome::Saved) => notify_success(app, "已保存 PNG"),
                Ok(QrSaveOutcome::Cancelled) => notify_success(app, "已取消保存"),
                Err(error) => notify_error(app, &error),
            }

            clear_notification_task()
        }
    }
}

fn build_render_request(app: &mut App) -> Result<QrRenderRequest, String> {
    let data = app.qr_editor.text();
    if data.trim().is_empty() {
        return Err("请输入二维码内容".to_string());
    }

    let size = parse_qr_size_input(&app.qr_size_input)?;
    app.qr_size = size;
    app.qr_size_input = size.to_string();

    let icon_bytes = match app.qr_icon_mode {
        QrIconMode::None => None,
        QrIconMode::Default => Some(include_bytes!("../../../../../assets/logo.png").to_vec()),
        QrIconMode::Upload => Some(
            app.qr_icon_bytes
                .clone()
                .ok_or_else(|| "请先选择上传图标".to_string())?,
        ),
    };

    Ok(QrRenderRequest {
        data,
        size,
        level: app.qr_level,
        color_hex: app.qr_color_hex.clone(),
        icon_bytes,
    })
}

fn parse_qr_size_input(input: &str) -> Result<u32, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("请输入二维码尺寸".to_string());
    }

    let value = trimmed.parse::<u32>().map_err(|_| "二维码尺寸必须是数字".to_string())?;
    if !(QR_MIN_SIZE..=QR_MAX_SIZE).contains(&value) {
        return Err(format!("二维码尺寸需在 {QR_MIN_SIZE}-{QR_MAX_SIZE}px 之间"));
    }

    Ok(value)
}

fn render_qr_png(request: &QrRenderRequest) -> Result<Vec<u8>, String> {
    use image::ImageEncoder;
    use qrcodegen::{QrCode, QrCodeEcc};

    let ecc = match request.level {
        QrEcLevel::L => QrCodeEcc::Low,
        QrEcLevel::M => QrCodeEcc::Medium,
        QrEcLevel::Q => QrCodeEcc::Quartile,
        QrEcLevel::H => QrCodeEcc::High,
    };

    let code = QrCode::encode_text(&request.data, ecc)
        .map_err(|_| "二维码内容过长或当前纠错等级不支持".to_string())?;
    let module_count = code.size();
    let scale = (request.size / module_count as u32).max(1);
    let margin = 4u32;
    let side = (module_count as u32 + margin * 2) * scale;
    let mut rgba = vec![255u8; (side * side * 4) as usize];
    let fg = parse_color(&request.color_hex).unwrap_or(Color::from_rgb8(0, 0, 0));
    let fg_rgb = (
        (fg.r * 255.0).round() as u8,
        (fg.g * 255.0).round() as u8,
        (fg.b * 255.0).round() as u8,
    );

    for y in 0..module_count {
        for x in 0..module_count {
            let is_dark = code.get_module(x, y);
            let base_x = (x as u32 + margin) * scale;
            let base_y = (y as u32 + margin) * scale;

            for dy in 0..scale {
                for dx in 0..scale {
                    let px = base_x + dx;
                    let py = base_y + dy;
                    let idx = ((py * side + px) * 4) as usize;

                    if is_dark {
                        rgba[idx] = fg_rgb.0;
                        rgba[idx + 1] = fg_rgb.1;
                        rgba[idx + 2] = fg_rgb.2;
                    } else {
                        rgba[idx] = 255;
                        rgba[idx + 1] = 255;
                        rgba[idx + 2] = 255;
                    }
                    rgba[idx + 3] = 255;
                }
            }
        }
    }

    if let Some(icon_bytes) = &request.icon_bytes {
        overlay_icon(&mut rgba, side, scale, icon_bytes)?;
    }

    let mut png = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png);
    let encoder = image::codecs::png::PngEncoder::new(&mut cursor);
    encoder
        .write_image(&rgba, side, side, image::ExtendedColorType::Rgba8)
        .map_err(|_| "二维码 PNG 编码失败".to_string())?;

    Ok(png)
}

fn overlay_icon(
    rgba: &mut [u8],
    side: u32,
    scale: u32,
    icon_bytes: &[u8],
) -> Result<(), String> {
    let icon = image::load_from_memory(icon_bytes)
        .map_err(|_| "无法读取图标图片".to_string())?
        .to_rgba8();
    let target_w = ((side as f32 * 0.25).round() as u32).max(1);
    let scale_icon = (target_w as f32 / icon.width() as f32).max(0.01);
    let target_h = ((icon.height() as f32 * scale_icon).round() as u32).max(1);
    let icon_resized = image::imageops::resize(
        &icon,
        target_w,
        target_h,
        image::imageops::FilterType::Lanczos3,
    );
    let icon_w = icon_resized.width();
    let icon_h = icon_resized.height();
    let left = side / 2 - icon_w / 2;
    let top = side / 2 - icon_h / 2;
    let pad = (scale * 2).max(8);
    let clear_left = left.saturating_sub(pad);
    let clear_top = top.saturating_sub(pad);
    let clear_right = (left + icon_w + pad).min(side);
    let clear_bottom = (top + icon_h + pad).min(side);

    for py in clear_top..clear_bottom {
        for px in clear_left..clear_right {
            let idx = ((py * side + px) * 4) as usize;
            rgba[idx] = 255;
            rgba[idx + 1] = 255;
            rgba[idx + 2] = 255;
            rgba[idx + 3] = 255;
        }
    }

    for iy in 0..icon_h {
        for ix in 0..icon_w {
            let dst_x = left + ix;
            let dst_y = top + iy;
            let dst_idx = ((dst_y * side + dst_x) * 4) as usize;
            let src_idx = ((iy * icon_w + ix) * 4) as usize;
            let sr = icon_resized.as_raw()[src_idx];
            let sg = icon_resized.as_raw()[src_idx + 1];
            let sb = icon_resized.as_raw()[src_idx + 2];
            let sa = icon_resized.as_raw()[src_idx + 3] as f32 / 255.0;
            let dr = rgba[dst_idx] as f32;
            let dg = rgba[dst_idx + 1] as f32;
            let db = rgba[dst_idx + 2] as f32;

            rgba[dst_idx] = (sr as f32 * sa + dr * (1.0 - sa)).round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 1] =
                (sg as f32 * sa + dg * (1.0 - sa)).round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 2] =
                (sb as f32 * sa + db * (1.0 - sa)).round().clamp(0.0, 255.0) as u8;
            rgba[dst_idx + 3] = 255;
        }
    }

    Ok(())
}

fn clear_notification_task() -> Task<Message> {
    crate::app::message::after(
        std::time::Duration::from_secs(2),
        Message::QrTool(QrToolMessage::ClearNotification),
    )
}

fn notify_success(app: &mut App, message: &str) {
    app.qr_notification = Some(message.to_string());
    app.qr_notification_is_error = false;
}

fn notify_error(app: &mut App, message: &str) {
    app.qr_notification = Some(message.to_string());
    app.qr_notification_is_error = true;
}

fn visible_line_count(app: &App) -> f32 {
    let line_height = app.current_line_height.max(1.0);
    (app.qr_viewport_height / line_height).floor().max(1.0)
}

fn max_scroll_top_line(app: &App) -> f32 {
    let total_lines = app.qr_editor.line_count().max(1) as f32;
    (total_lines - visible_line_count(app)).max(0.0)
}

fn apply_scroll_lines(app: &mut App, delta_lines: i32) {
    let max_scroll = max_scroll_top_line(app);
    app.qr_scroll_top_line = (app.qr_scroll_top_line + delta_lines as f32).clamp(0.0, max_scroll);
}

#[cfg(test)]
#[path = "qr_tool_tests.rs"]
mod tests;
