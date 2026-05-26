use crate::app::{App, Message};
use crate::apps::mindmap::canvas::export_svg as render_svg;
#[cfg(not(target_arch = "wasm32"))]
use crate::apps::mindmap::canvas::render_svg_to_png;
use crate::apps::mindmap::state::MindMapTab;
use iced::Task;

use super::super::super::types::MindMapMessage;

fn prepare_export(tab: &mut MindMapTab) -> String {
    tab.show_action_menu = false;
    tab.show_export_menu = false;
    render_svg(tab)
}

/// 导出为 SVG 图片
///
/// 将当前思维导图渲染为矢量图并保存。
/// 导出完成后自动打开文件。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
///
/// # 返回
///
/// 异步导出任务，完成时发送 `ExportFinished` 消息
pub(crate) fn export_svg(app: &mut App) -> Task<Message> {
    let Some(tab) = app.active_mindmap_tab_mut() else {
        return Task::none();
    };
    let svg = prepare_export(tab);
    #[cfg(target_arch = "wasm32")]
    let _ = &svg;
    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file =
                    rfd::AsyncFileDialog::new().set_file_name("mindmap.svg").save_file().await;
                if let Some(file) = file {
                    file.write(svg.as_bytes()).await.map_err(|e| e.to_string())?;
                    let _ = open::that(file.path());
                }
                Ok(())
            }
            #[cfg(target_arch = "wasm32")]
            {
                Ok(())
            }
        },
        |res: Result<(), String>| Message::MindMapTool(MindMapMessage::ExportFinished(res)),
    )
}

/// 导出为 PNG 图片
///
/// 将当前思维导图先渲染为 SVG，再转换为 PNG 位图。
/// 导出完成后自动打开文件。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
///
/// # 返回
///
/// 异步导出任务，完成时发送 `ExportFinished` 消息
pub(crate) fn export_png(app: &mut App) -> Task<Message> {
    let Some(tab) = app.active_mindmap_tab_mut() else {
        return Task::none();
    };
    let svg = prepare_export(tab);
    #[cfg(target_arch = "wasm32")]
    let _ = &svg;
    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file =
                    rfd::AsyncFileDialog::new().set_file_name("mindmap.png").save_file().await;
                if let Some(file) = file {
                    let png =
                        render_svg_to_png(&svg).ok_or_else(|| "Render PNG failed".to_string())?;
                    file.write(&png).await.map_err(|e| e.to_string())?;
                    let _ = open::that(file.path());
                }
                Ok(())
            }
            #[cfg(target_arch = "wasm32")]
            {
                Ok(())
            }
        },
        |res: Result<(), String>| Message::MindMapTool(MindMapMessage::ExportFinished(res)),
    )
}

/// 导出为 JPEG 图片
///
/// 将当前思维导图先渲染为 PNG，再转换为 JPEG 格式。
/// 使用 90% 的 JPEG 质量设置。
/// 导出完成后自动打开文件。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
///
/// # 返回
///
/// 异步导出任务，完成时发送 `ExportFinished` 消息
pub(crate) fn export_jpeg(app: &mut App) -> Task<Message> {
    let Some(tab) = app.active_mindmap_tab_mut() else {
        return Task::none();
    };
    let svg = prepare_export(tab);
    #[cfg(target_arch = "wasm32")]
    let _ = &svg;
    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file =
                    rfd::AsyncFileDialog::new().set_file_name("mindmap.jpg").save_file().await;
                if let Some(file) = file {
                    let png =
                        render_svg_to_png(&svg).ok_or_else(|| "Render PNG failed".to_string())?;
                    let img = image::load_from_memory(&png)
                        .map_err(|e| format!("Decode PNG failed: {e}"))?;
                    let rgb_img = image::DynamicImage::ImageRgba8(img.to_rgba8()).into_rgb8();
                    let mut jpeg_data = std::io::Cursor::new(Vec::new());
                    let mut encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, 90);
                    encoder
                        .encode(
                            rgb_img.as_raw(),
                            rgb_img.width(),
                            rgb_img.height(),
                            image::ExtendedColorType::Rgb8,
                        )
                        .map_err(|e| format!("Encode JPEG failed: {e}"))?;
                    file.write(&jpeg_data.into_inner()).await.map_err(|e| e.to_string())?;
                    let _ = open::that(file.path());
                }
                Ok(())
            }
            #[cfg(target_arch = "wasm32")]
            {
                Ok(())
            }
        },
        |res: Result<(), String>| Message::MindMapTool(MindMapMessage::ExportFinished(res)),
    )
}

/// 处理导出完成结果
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
/// - `res`: 导出操作结果
///
/// # 返回
///
/// 空的 Task
pub(crate) fn export_finished(app: &mut App, res: Result<(), String>) -> Task<Message> {
    if let Err(error) = res {
        app.error_message = Some(error);
    }
    Task::none()
}
