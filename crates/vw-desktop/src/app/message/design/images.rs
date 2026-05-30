//! 处理设计编辑器中的图片导入、加载和资源状态更新。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{DesignMessage, ImageImportPayload};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::views::design::canvas::creation::create_image_element;
use crate::app::views::design::canvas::creation::create_sticky_note_element;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::views::design::models::DesignElement;
use crate::app::views::design::models::{DesignDoc, StickyNoteKind};
use crate::app::views::design::state::ImageImportTarget;
use crate::app::{App, Message};
#[cfg(not(target_arch = "wasm32"))]
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE as BASE64_URL_SAFE},
};
#[cfg(not(target_arch = "wasm32"))]
use iced::widget::image::Handle;
use iced::{Point, Task};
#[cfg(not(target_arch = "wasm32"))]
use image::GenericImageView;
#[cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
fn guess_image_size(bytes: &[u8]) -> Option<(u32, u32)> {
    image::load_from_memory(bytes).ok().map(|img| img.dimensions())
}

#[cfg(not(target_arch = "wasm32"))]
fn bytes_look_like_svg(bytes: &[u8]) -> bool {
    let trimmed = String::from_utf8_lossy(bytes);
    let trimmed = trimmed.trim_start_matches('\u{feff}').trim_start();
    trimmed.starts_with("<svg") || trimmed.starts_with("<?xml") || trimmed.contains("<svg")
}

#[cfg(not(target_arch = "wasm32"))]
fn source_looks_like_svg(source: &str) -> bool {
    let lowered = source.trim().to_ascii_lowercase();
    lowered.ends_with(".svg")
        || lowered.contains(".svg?")
        || lowered.starts_with("data:image/svg+xml")
}

#[cfg(not(target_arch = "wasm32"))]
fn render_svg_bytes_to_png(bytes: &[u8]) -> Result<(Vec<u8>, (u32, u32)), String> {
    use resvg::usvg;
    use tiny_skia::{Pixmap, Transform};

    let svg_data = std::str::from_utf8(bytes).map_err(|err| err.to_string())?;
    let mut opt = usvg::Options::default();
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    opt.fontdb = std::sync::Arc::new(fontdb);

    let tree = usvg::Tree::from_str(svg_data, &opt).map_err(|err| err.to_string())?;
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height())
        .ok_or_else(|| "无法为 SVG 创建位图".to_string())?;
    let mut pixmap_mut = pixmap.as_mut();
    resvg::render(&tree, Transform::default(), &mut pixmap_mut);

    let png_bytes = pixmap.encode_png().map_err(|err| err.to_string())?;
    Ok((png_bytes, (size.width(), size.height())))
}

#[cfg(not(target_arch = "wasm32"))]
fn prepare_image_bytes_for_canvas(
    source_hint: &str,
    raw_bytes: Vec<u8>,
) -> Result<(Vec<u8>, Option<(u32, u32)>), String> {
    if source_looks_like_svg(source_hint) || bytes_look_like_svg(&raw_bytes) {
        let (png_bytes, size) = render_svg_bytes_to_png(&raw_bytes)?;
        return Ok((png_bytes, Some(size)));
    }

    let size = guess_image_size(&raw_bytes);
    Ok((raw_bytes, size))
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_data_url(source: &str) -> Result<Vec<u8>, String> {
    let (header, payload) = source.split_once(',').ok_or_else(|| "无效的 data URL".to_string())?;
    if header.contains(";base64") {
        decode_base64_payload(payload)
    } else {
        Ok(payload.as_bytes().to_vec())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_base64_payload(payload: &str) -> Result<Vec<u8>, String> {
    let compact: String = payload.chars().filter(|ch| !ch.is_whitespace()).collect();
    BASE64_STANDARD
        .decode(&compact)
        .or_else(|_| BASE64_URL_SAFE.decode(&compact))
        .map_err(|err| err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn file_path_from_source(source: &str) -> Option<PathBuf> {
    let trimmed = source.trim();
    if let Some(path) = trimmed.strip_prefix("file://localhost/") {
        return Some(PathBuf::from(format!("/{}", path.trim_start_matches('/'))));
    }
    if let Some(path) = trimmed.strip_prefix("file://") {
        return Some(PathBuf::from(path));
    }
    let path = Path::new(trimmed);
    path.exists().then(|| path.to_path_buf())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_non_network_image_payload(source: &str) -> Result<(Vec<u8>, Option<(u32, u32)>), String> {
    let trimmed = source.trim();
    let raw_bytes = if trimmed.starts_with("data:") {
        decode_data_url(trimmed)?
    } else if let Some(path) = file_path_from_source(trimmed) {
        std::fs::read(path).map_err(|err| err.to_string())?
    } else {
        decode_base64_payload(trimmed).map_err(|_| {
            "不支持的图片来源，需为本地路径、file://、base64、data URL 或网络 URL".to_string()
        })?
    };

    prepare_image_bytes_for_canvas(trimmed, raw_bytes)
}

#[cfg(not(target_arch = "wasm32"))]
fn message_from_loaded_image(
    source: String,
    result: Result<(Vec<u8>, Option<(u32, u32)>), String>,
) -> Message {
    let result = result.map(|(bytes, size_opt)| (Handle::from_bytes(bytes), size_opt));
    Message::Design(DesignMessage::ImageLoaded(source, result))
}

#[cfg(not(target_arch = "wasm32"))]
async fn load_design_image_bytes_async(
    source: String,
) -> Result<(Vec<u8>, Option<(u32, u32)>), String> {
    let trimmed = source.trim().to_string();
    if trimmed.is_empty() {
        return Err("请输入图片 URL、本地路径、data URL 或 base64".to_string());
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let client = reqwest::Client::new();
        let response = client
            .get(&trimmed)
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) VibeWindow/1.0",
            )
            .header(
                reqwest::header::ACCEPT,
                "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
            )
            .send()
            .await
            .map_err(|err| err.to_string())?;
        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }
        let bytes = response.bytes().await.map_err(|err| err.to_string())?.to_vec();
        return prepare_image_bytes_for_canvas(&trimmed, bytes);
    }

    tokio::task::spawn_blocking(move || load_non_network_image_payload(&trimmed))
        .await
        .map_err(|err| err.to_string())?
}

#[cfg(not(target_arch = "wasm32"))]
fn load_design_image_task(source: String) -> Task<Message> {
    let trimmed = source.trim().to_string();
    if trimmed.is_empty() {
        return Task::none();
    }

    // 耗时或平台相关操作交给异步任务，避免阻塞界面消息循环。

    Task::perform(load_design_image_bytes_async(trimmed), move |result| {
        message_from_loaded_image(source, result)
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_image_sources_from_fill_value(value: &Value, sources: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("image")
                && map.get("enabled").and_then(Value::as_bool).unwrap_or(true)
                && let Some(url) = map.get("url").and_then(Value::as_str)
                && !url.trim().is_empty()
            {
                sources.push(url.trim().to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_image_sources_from_fill_value(item, sources);
            }
        }
        _ => {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// load_image_tasks_from_fill_value 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn load_image_tasks_from_fill_value(fill: &Value) -> Vec<Task<Message>> {
    let mut sources = Vec::new();
    collect_image_sources_from_fill_value(fill, &mut sources);
    let mut seen = std::collections::HashSet::new();
    sources
        .into_iter()
        .filter(|source| seen.insert(source.clone()))
        .map(load_design_image_task)
        .collect()
}

#[cfg(target_arch = "wasm32")]
/// load_image_tasks_from_fill_value 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn load_image_tasks_from_fill_value(_fill: &serde_json::Value) -> Vec<Task<Message>> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn collect_image_sources_from_element(element: &DesignElement, sources: &mut Vec<String>) {
    if let Some(fill) = &element.fill {
        collect_image_sources_from_fill_value(fill, sources);
    }
    for child in &element.children {
        collect_image_sources_from_element(child, sources);
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// load_image_tasks_from_document 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn load_image_tasks_from_document(doc: &DesignDoc) -> Vec<Task<Message>> {
    let mut sources = Vec::new();
    for element in &doc.children {
        collect_image_sources_from_element(element, &mut sources);
    }
    let mut seen = std::collections::HashSet::new();
    sources
        .into_iter()
        .filter(|source| seen.insert(source.clone()))
        .map(load_design_image_task)
        .collect()
}

#[cfg(target_arch = "wasm32")]
/// load_image_tasks_from_document 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn load_image_tasks_from_document(_doc: &DesignDoc) -> Vec<Task<Message>> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_image_import_payload(app: &mut App, payload: ImageImportPayload) -> Task<Message> {
    use crate::app::views::design::properties::fill::types::{FillItem, FillObject};
    use serde_json::json;

    let Some(state) = app.active_design_state_mut() else {
        return Task::none();
    };

    match payload.target {
        ImageImportTarget::Element => {
            let position = Point::new(
                (-state.pan.x + 220.0) / state.zoom,
                (-state.pan.y + 160.0) / state.zoom,
            );
            state.doc.images.insert(payload.source.clone(), Handle::from_bytes(payload.bytes));
            if let Some(size) = payload.size_opt {
                state.doc.image_sizes.insert(payload.source.clone(), size);
            }
            state.canvas_cache.clear();
            let element = create_image_element(position, payload.source, payload.size_opt);
            Task::done(Message::Design(DesignMessage::CreateElement {
                element,
                parent_id: None,
                start_editing: false,
            }))
        }
        ImageImportTarget::Fill { element_id, fill_index } => {
            if let Some(el) = state.doc.find_element(&element_id) {
                let fills = match &el.fill {
                    Some(v) => {
                        if let Ok(list) = serde_json::from_value::<Vec<FillItem>>(v.clone()) {
                            list
                        } else if let Ok(single) = serde_json::from_value::<FillItem>(v.clone()) {
                            vec![single]
                        } else {
                            Vec::new()
                        }
                    }
                    None => Vec::new(),
                };

                let mut new_fills = fills;
                if let Some(item) = new_fills.get_mut(fill_index)
                    && let FillItem::Object(FillObject::Image(img)) = item
                {
                    img.url = payload.source.clone();
                    img.mode = "fill_width".to_string();
                }

                state.doc.images.insert(payload.source.clone(), Handle::from_bytes(payload.bytes));
                if let Some(size) = payload.size_opt {
                    state.doc.image_sizes.insert(payload.source.clone(), size);
                }
                state.doc.update_property(&element_id, "fill", json!(new_fills));
                state.canvas_cache.clear();
            }

            Task::done(Message::Design(DesignMessage::Snapshot))
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn apply_image_import_payload(_app: &mut App, _payload: ImageImportPayload) -> Task<Message> {
    Task::none()
}

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(super) fn update(app: &mut App, message: DesignMessage) -> Option<Task<Message>> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        DesignMessage::ImageLoaded(url, result) => {
            if let Some(state) = app.active_design_state_mut() {
                match result {
                    Ok((handle, size_opt)) => {
                        println!("Image loaded successfully: {}", url);
                        let key = url.clone();
                        state.doc.images.insert(key.clone(), handle);
                        if let Some(size) = size_opt {
                            state.doc.image_sizes.insert(key, size);
                        }
                        state.canvas_cache.clear();
                    }
                    Err(error) => {
                        eprintln!("Failed to load image {}: {}", url, error);
                    }
                }
            }
            Some(Task::none())
        }
        DesignMessage::ImportImageElement => {
            if let Some(state) = app.active_design_state_mut() {
                state.image_import_target = Some(ImageImportTarget::Element);
                state.image_import_input.clear();
                state.image_import_error = None;
                state.image_import_loading = false;
            }
            Some(Task::none())
        }
        DesignMessage::ImportFillImage(element_id, fill_index) => {
            if let Some(state) = app.active_design_state_mut() {
                state.image_import_target =
                    Some(ImageImportTarget::Fill { element_id, fill_index });
                state.image_import_input.clear();
                state.image_import_error = None;
                state.image_import_loading = false;
            }
            Some(Task::none())
        }
        DesignMessage::CloseImageImportDialog => {
            if let Some(state) = app.active_design_state_mut() {
                state.image_import_target = None;
                state.image_import_input.clear();
                state.image_import_error = None;
                state.image_import_loading = false;
            }
            Some(Task::none())
        }
        DesignMessage::OpenStickyNoteDialog => {
            if let Some(state) = app.active_design_state_mut() {
                state.sticky_note_dialog_open = true;
                state.sticky_note_dialog_default_kind = StickyNoteKind::Note;
            }
            Some(Task::none())
        }
        DesignMessage::CloseStickyNoteDialog => {
            if let Some(state) = app.active_design_state_mut() {
                state.sticky_note_dialog_open = false;
            }
            Some(Task::none())
        }
        DesignMessage::CreateStickyNote(kind) => {
            let Some(state) = app.active_design_state_mut() else {
                return Some(Task::none());
            };

            state.sticky_note_dialog_open = false;
            state.sticky_note_dialog_default_kind = kind;
            let position = Point::new(
                (-state.pan.x + 220.0) / state.zoom,
                (-state.pan.y + 160.0) / state.zoom,
            );
            let element = create_sticky_note_element(position, kind);
            Some(Task::done(Message::Design(DesignMessage::CreateElement {
                element,
                parent_id: None,
                start_editing: false,
            })))
        }
        DesignMessage::ImageImportInputChanged(value) => {
            if let Some(state) = app.active_design_state_mut() {
                state.image_import_input = value;
                state.image_import_error = None;
            }
            Some(Task::none())
        }
        DesignMessage::PasteImageImportInput => {
            Some(iced::clipboard::read().map(|content| {
                Message::Design(DesignMessage::ImageImportClipboardReceived(content))
            }))
        }
        DesignMessage::ImageImportClipboardReceived(content) => {
            if let Some(state) = app.active_design_state_mut() {
                match content.map(|value| value.trim().to_string()) {
                    Some(value) if !value.is_empty() => {
                        state.image_import_input = value;
                        state.image_import_error = None;
                    }
                    _ => {
                        state.image_import_error =
                            Some("剪贴板为空，或不是可粘贴的图片文本".to_string());
                    }
                }
            }
            Some(Task::none())
        }
        DesignMessage::ChooseImageImportFile => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let Some(target) =
                    app.active_design_state().and_then(|state| state.image_import_target.clone())
                else {
                    return Some(Task::none());
                };

                if let Some(state) = app.active_design_state_mut() {
                    state.image_import_loading = true;
                    state.image_import_error = None;
                }

                Some(Task::perform(
                    async move {
                        tokio::task::spawn_blocking(
                            move || -> Result<Option<ImageImportPayload>, String> {
                                use rfd::FileDialog;

                                let Some(path) = FileDialog::new()
                                    .add_filter(
                                        "Images",
                                        &[
                                            "png", "jpg", "jpeg", "webp", "gif", "bmp", "tif",
                                            "tiff", "svg",
                                        ],
                                    )
                                    .pick_file()
                                else {
                                    return Ok(None);
                                };

                                let source = path.to_string_lossy().to_string();
                                let raw_bytes =
                                    std::fs::read(path.as_path()).map_err(|err| err.to_string())?;
                                let (bytes, size_opt) =
                                    prepare_image_bytes_for_canvas(&source, raw_bytes)?;
                                Ok(Some(ImageImportPayload { target, source, bytes, size_opt }))
                            },
                        )
                        .await
                        .map_err(|err| err.to_string())?
                    },
                    |result| Message::Design(DesignMessage::ImageImportFilePicked(result)),
                ))
            }
            #[cfg(target_arch = "wasm32")]
            {
                Some(Task::none())
            }
        }
        DesignMessage::ImageImportFilePicked(result) => {
            let Some(state) = app.active_design_state_mut() else {
                return Some(Task::none());
            };

            state.image_import_loading = false;
            match result {
                Ok(Some(payload)) => {
                    state.image_import_target = None;
                    state.image_import_input.clear();
                    state.image_import_error = None;
                    return Some(apply_image_import_payload(app, payload));
                }
                Ok(None) => {}
                Err(error) => {
                    state.image_import_error = Some(error);
                }
            }
            Some(Task::none())
        }
        DesignMessage::SubmitImageImport => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let Some((target, source)) = app.active_design_state().and_then(|state| {
                    state
                        .image_import_target
                        .clone()
                        .map(|target| (target, state.image_import_input.clone()))
                }) else {
                    return Some(Task::none());
                };

                if source.trim().is_empty() {
                    if let Some(state) = app.active_design_state_mut() {
                        state.image_import_error =
                            Some("请输入图片 URL、本地路径、data URL 或 base64".to_string());
                    }
                    return Some(Task::none());
                }

                if let Some(state) = app.active_design_state_mut() {
                    state.image_import_loading = true;
                    state.image_import_error = None;
                }

                Some(Task::perform(
                    async move {
                        let (bytes, size_opt) =
                            load_design_image_bytes_async(source.clone()).await?;
                        Ok(ImageImportPayload { target, source, bytes, size_opt })
                    },
                    |result| Message::Design(DesignMessage::ImageImportResolved(result)),
                ))
            }
            #[cfg(target_arch = "wasm32")]
            {
                Some(Task::none())
            }
        }
        DesignMessage::ImageImportResolved(result) => {
            let Some(state) = app.active_design_state_mut() else {
                return Some(Task::none());
            };

            state.image_import_loading = false;
            match result {
                Ok(payload) => {
                    state.image_import_target = None;
                    state.image_import_input.clear();
                    state.image_import_error = None;
                    return Some(apply_image_import_payload(app, payload));
                }
                Err(error) => {
                    state.image_import_error = Some(error);
                }
            }
            Some(Task::none())
        }
        _ => None,
    }
}
