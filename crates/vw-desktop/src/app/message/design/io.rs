use crate::app::message::{DesignMessage, NotificationMessage};
#[cfg(not(target_arch = "wasm32"))]
use crate::app::views::design::import::{
    count_figma_pages, figma_to_design_doc_with_base_dir_and_progress,
};
use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::state::DesignState;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::views::design::state::{FigmaProgressStage, FigmaProgressState};
use crate::app::{App, AppTab, Message, Screen};
use iced::Task;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use crate::app::views::design::export::{
    generate_element_html, generate_element_svg, generate_html,
};

pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match message {
        DesignMessage::New => {
            let doc =
                DesignDoc { version: "1.0".to_string(), children: vec![], ..Default::default() };

            let tab_id = app
                .active_tab_id
                .clone()
                .and_then(|id| {
                    let is_design_tab = app
                        .open_tabs
                        .iter()
                        .any(|t| t.id == id && matches!(t.screen, Screen::Design));
                    if is_design_tab && !app.design_states.contains_key(&id) {
                        Some(id)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "design".to_string());

            let mut state = DesignState::new(doc);
            state.file_path = None;
            app.design_states.insert(tab_id.clone(), state);

            if let Some(tab) = app.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                tab.title = "设计".to_string();
                tab.screen = Screen::Design;
                tab.project_path = None;
            } else {
                app.open_tabs.push(AppTab {
                    id: tab_id.clone(),
                    title: "设计".to_string(),
                    screen: Screen::Design,
                    project_path: None,
                });
            }

            app.active_tab_id = Some(tab_id);
            app.screen = Screen::Design;
            Task::none()
        }
        DesignMessage::ExportHtml => {
            if let Some(state) = app.active_design_state() {
                let doc = &state.doc;
                let html = generate_html(doc);
                Task::perform(
                    async move {
                        let file = rfd::AsyncFileDialog::new()
                            .set_file_name("export.html")
                            .save_file()
                            .await;
                        if let Some(file) = file {
                            let _ = file.write(html.as_bytes()).await;
                            #[cfg(not(target_arch = "wasm32"))]
                            let _ = open::that(file.path());
                        }
                    },
                    |_| Message::Design(DesignMessage::ToggleVariables),
                )
            } else {
                Task::none()
            }
        }
        DesignMessage::ExportElementHtml(id) => {
            if let Some(state) = app.active_design_state() {
                let doc = &state.doc;
                if let Some(html) = generate_element_html(doc, &id) {
                    Task::perform(
                        async move {
                            let file = rfd::AsyncFileDialog::new()
                                .set_file_name("export_element.html")
                                .save_file()
                                .await;
                            if let Some(file) = file {
                                let _ = file.write(html.as_bytes()).await;
                                #[cfg(not(target_arch = "wasm32"))]
                                let _ = open::that(file.path());
                            }
                        },
                        |_| Message::Design(DesignMessage::ToggleVariables),
                    )
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        DesignMessage::ExportElementSvg(id) => {
            if let Some(state) = app.active_design_state() {
                let doc = &state.doc;
                if let Some(_svg) = generate_element_svg(doc, &id) {
                    Task::perform(
                        async move {
                            let file = rfd::AsyncFileDialog::new()
                                .set_file_name("export.svg")
                                .save_file()
                                .await;
                            if let Some(file) = file {
                                let _ = file.write(_svg.as_bytes()).await;
                                #[cfg(not(target_arch = "wasm32"))]
                                let _ = open::that(file.path());
                            }
                        },
                        |_| Message::Design(DesignMessage::ToggleVariables),
                    )
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        DesignMessage::ExportElementPng(id) => {
            if let Some(state) = app.active_design_state() {
                let doc = &state.doc;
                if let Some(_svg) = generate_element_svg(doc, &id) {
                    Task::perform(
                        async move {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let file = rfd::AsyncFileDialog::new()
                                    .set_file_name("export.png")
                                    .save_file()
                                    .await;

                                if let Some(file) = file {
                                    // Render SVG to PNG
                                    if let Some(png_data) = render_svg_to_png(&_svg) {
                                        let _ = file.write(&png_data).await;
                                        let _ = open::that(file.path());
                                    }
                                }
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                // Not supported in WASM easily without additional libs
                            }
                        },
                        |_| Message::Design(DesignMessage::ToggleVariables),
                    )
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        DesignMessage::ExportElementJpeg(id) => {
            if let Some(state) = app.active_design_state() {
                let doc = &state.doc;
                if let Some(_svg) = generate_element_svg(doc, &id) {
                    Task::perform(
                        async move {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let file = rfd::AsyncFileDialog::new()
                                    .set_file_name("export.jpg")
                                    .save_file()
                                    .await;

                                if let Some(file) = file {
                                    // Render SVG to PNG then convert to JPEG
                                    if let Some(png_data) = render_svg_to_png(&_svg) {
                                        // Convert PNG bytes to JPEG using image crate
                                        if let Ok(img) = image::load_from_memory(&png_data) {
                                            let mut jpeg_data = std::io::Cursor::new(Vec::new());
                                            // Convert to RGB8 to drop alpha
                                            let rgb_img =
                                                image::DynamicImage::ImageRgba8(img.to_rgba8())
                                                    .into_rgb8();

                                            // Use JpegEncoder for quality control
                                            let mut encoder =
                                                image::codecs::jpeg::JpegEncoder::new_with_quality(
                                                    &mut jpeg_data,
                                                    90,
                                                );
                                            if let Ok(_) = encoder.encode(
                                                rgb_img.as_raw(),
                                                rgb_img.width(),
                                                rgb_img.height(),
                                                image::ExtendedColorType::Rgb8,
                                            ) {
                                                let _ = file.write(&jpeg_data.into_inner()).await;
                                                let _ = open::that(file.path());
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        |_| Message::Design(DesignMessage::ToggleVariables),
                    )
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        DesignMessage::Open => Task::perform(
            async {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("Design", &["json", "pen"])
                    .pick_file()
                    .await;

                if let Some(file) = file {
                    let data = file.read().await;
                    match serde_json::from_slice::<DesignDoc>(&data) {
                        Ok(mut doc) => {
                            doc.normalize_fill_flags();
                            #[cfg(not(target_arch = "wasm32"))]
                            let path = Some(file.path().to_path_buf());
                            #[cfg(target_arch = "wasm32")]
                            let path = None;
                            Ok((doc, path))
                        }
                        Err(e) => Err(format!("Failed to parse file: {}", e)),
                    }
                } else {
                    Err("Cancelled".to_string())
                }
            },
            |res| Message::Design(DesignMessage::FileOpened(res)),
        ),
        DesignMessage::ParseFigma => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (progress_tx, progress_rx) = std::sync::mpsc::channel();
                if let Some(state) = app.active_design_state_mut() {
                    state.figma_progress = Some(FigmaProgressState::new(
                        FigmaProgressStage::Parsing,
                        0,
                        1,
                        "等待选择 Figma 文件与输出目录…",
                    ));
                    state.figma_progress_rx = Some(progress_rx);
                }
                Task::perform(
                    async move {
                        let file = rfd::AsyncFileDialog::new()
                            .add_filter("Figma", &["fig", "zip"])
                            .pick_file()
                            .await
                            .ok_or_else(|| "Cancelled".to_string())?;
                        let output_dir = rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .ok_or_else(|| "Cancelled".to_string())?;
                        let input_path = file.path().to_path_buf();
                        let target_dir = output_dir.path().to_path_buf();
                        let bytes = file.read().await;
                        parse_figma_file_to_ai_folder(
                            &input_path,
                            &bytes,
                            &target_dir,
                            |current, total, detail| {
                                let _ = progress_tx.send(FigmaProgressState::new(
                                    FigmaProgressStage::Parsing,
                                    current,
                                    total,
                                    detail,
                                ));
                            },
                        )
                        .map_err(|error| error.to_string())?;
                        Ok(Some(target_dir))
                    },
                    |result: Result<Option<PathBuf>, String>| {
                        Message::Design(DesignMessage::FigmaParseCompleted(result))
                    },
                )
            }
            #[cfg(target_arch = "wasm32")]
            {
                Task::none()
            }
        }
        DesignMessage::FigmaParseCompleted(result) => {
            if let Some(state) = app.active_design_state_mut() {
                state.figma_progress = None;
                state.figma_progress_rx = None;
            }
            let message = match result {
                Ok(Some(path)) => {
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = open::that(&path);
                    Message::Notification(NotificationMessage::Add(format!(
                        "Figma 已解析到文件夹：{}",
                        path.display()
                    )))
                }
                Ok(None) => Message::None,
                Err(error) if error == "Cancelled" => Message::None,
                Err(error) => Message::Notification(NotificationMessage::Add(format!(
                    "解析 Figma 失败：{}",
                    error
                ))),
            };
            Task::done(message)
        }
        DesignMessage::Save => {
            if let Some(state) = app.active_design_state()
                && let Ok(json) = serde_json::to_string_pretty(&state.doc)
            {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(path) = &state.file_path {
                    let path = path.clone();
                    return Task::perform(
                        async move {
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                use std::io::Write;
                                match std::fs::File::create(&path) {
                                    Ok(mut file) => match file.write_all(json.as_bytes()) {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(e.to_string()),
                                    },
                                    Err(e) => Err(e.to_string()),
                                }
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                Ok(())
                            }
                        },
                        |res: Result<(), String>| match res {
                            Ok(_) => Message::Design(DesignMessage::ToggleVariables), // Just a dummy update or show notification
                            Err(e) => {
                                eprintln!("Error saving file: {}", e);
                                Message::Design(DesignMessage::ToggleVariables)
                            }
                        },
                    );
                } else {
                    // Save As logic if no path
                    return update(app, DesignMessage::SaveAs);
                }

                #[cfg(target_arch = "wasm32")]
                {
                    // For WASM, we might trigger a download or similar.
                    // rfd::AsyncFileDialog::new().save_file() might work.
                    return Task::perform(
                        async move {
                            let file = rfd::AsyncFileDialog::new()
                                .set_file_name("design.json")
                                .save_file()
                                .await;
                            if let Some(file) = file {
                                let _ = file.write(json.as_bytes()).await;
                            }
                        },
                        |_| Message::Design(DesignMessage::ToggleVariables),
                    );
                }
            }
            Task::none()
        }
        DesignMessage::SaveAs => {
            if let Some(state) = app.active_design_state()
                && let Ok(json) = serde_json::to_string_pretty(&state.doc)
            {
                Task::perform(
                    async move {
                        let file = rfd::AsyncFileDialog::new()
                            .set_file_name("design.json")
                            .save_file()
                            .await;

                        if let Some(file) = file {
                            if let Err(e) = file.write(json.as_bytes()).await {
                                return Err(e.to_string());
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            return Ok(Some(file.path().to_path_buf()));
                            #[cfg(target_arch = "wasm32")]
                            return Ok(None);
                        }
                        Ok(None)
                    },
                    |res| Message::Design(DesignMessage::FileSaved(res.ok().flatten())),
                )
            } else {
                Task::none()
            }
        }
        DesignMessage::FileOpened(res) => {
            match res {
                Ok((doc, path)) => {
                    let image_tasks = super::load_image_tasks_from_document(&doc);
                    let mut state = DesignState::new(doc);
                    state.file_path = path.clone();

                    let entry_tab_id = app.active_tab_id.clone().and_then(|id| {
                        let is_design_tab = app
                            .open_tabs
                            .iter()
                            .any(|t| t.id == id && matches!(t.screen, Screen::Design));
                        if is_design_tab && !app.design_states.contains_key(&id) {
                            Some(id)
                        } else {
                            None
                        }
                    });

                    let tab_title = if let Some(p) = &path {
                        p.file_name().unwrap_or_default().to_string_lossy().to_string()
                    } else {
                        format!("Design {}", app.design_states.len() + 1)
                    };

                    if let Some(tab_id) = entry_tab_id {
                        app.design_states.insert(tab_id.clone(), state);
                        if let Some(tab) = app.open_tabs.iter_mut().find(|t| t.id == tab_id) {
                            tab.title = tab_title;
                            tab.screen = Screen::Design;
                            tab.project_path = None;
                        }
                        app.active_tab_id = Some(tab_id);
                        app.screen = Screen::Design;
                    } else {
                        let tab_id_base = tab_title.clone();

                        let mut unique_id = tab_id_base.clone();
                        let mut count = 1;
                        while app.design_states.contains_key(&unique_id) {
                            unique_id = format!("{} ({})", tab_id_base, count);
                            count += 1;
                        }
                        let tab_id = unique_id;

                        app.design_states.insert(tab_id.clone(), state);

                        app.open_tabs.push(AppTab {
                            id: tab_id.clone(),
                            title: tab_title,
                            screen: Screen::Design,
                            project_path: None,
                        });

                        app.active_tab_id = Some(tab_id);
                        app.screen = Screen::Design;
                    }
                    return Task::batch(image_tasks);
                }
                Err(e) => {
                    eprintln!("Open failed: {}", e);
                    if e != "Cancelled" {
                        let error_msg = format!("打开设计文件失败：{}", e);
                        app.error_message = Some(error_msg.clone());
                        return Task::done(Message::Notification(NotificationMessage::Add(
                            error_msg.to_string(),
                        )));
                    }
                }
            }
            Task::none()
        }
        DesignMessage::FileSaved(path) => {
            if let Some(p) = path
                && let Some(state) = app.active_design_state_mut() {
                    state.file_path = Some(p);
                }
            Task::none()
        }
        _ => Task::none(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_figma_file_to_ai_folder<F>(
    input_path: &Path,
    bytes: &[u8],
    target_dir: &Path,
    mut on_progress: F,
) -> anyhow::Result<()>
where
    F: FnMut(usize, usize, String),
{
    use anyhow::{Context, anyhow, bail};

    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create output directory: {}", target_dir.display()))?;

    if vw_figma_json::parser::is_zip_container(bytes) {
        vw_figma_json::parser::extract_zip_to_directory(bytes, target_dir)
            .context("Failed to extract Figma ZIP archive")?;
        let fig_files = find_fig_files(target_dir)?;
        if fig_files.is_empty() {
            bail!("No .fig files found in extracted archive");
        }
        let mut fig_file_entries = Vec::with_capacity(fig_files.len());
        let mut total_pages = 0usize;
        for fig_path in fig_files {
            let fig_path: PathBuf = fig_path;
            let fig_bytes = fs::read(fig_path.as_path()).with_context(|| {
                format!("Failed to read extracted file: {}", fig_path.display())
            })?;
            let base_dir = fig_path
                .parent()
                .ok_or_else(|| anyhow!("Invalid extracted Figma path: {}", fig_path.display()))?
                .to_path_buf();
            let figma_json =
                vw_figma_json::convert(&fig_bytes, Some(&base_dir)).with_context(|| {
                    format!("Failed to export Figma JSON for: {}", fig_path.display())
                })?;
            total_pages = total_pages.saturating_add(count_figma_pages(&figma_json));
            fig_file_entries.push((fig_path, base_dir, figma_json));
        }
        let total_pages = total_pages.max(1);
        let mut processed_pages = 0usize;
        for (fig_path, base_dir, figma_json) in fig_file_entries {
            let fig_bytes = fs::read(fig_path.as_path()).with_context(|| {
                format!("Failed to read extracted file: {}", fig_path.display())
            })?;
            let file_total_pages = count_figma_pages(&figma_json);
            let json = figma_to_design_doc_with_base_dir_and_progress(
                &fig_bytes,
                Some(&base_dir),
                |progress| {
                    on_progress(
                        processed_pages.saturating_add(progress.completed_pages),
                        total_pages,
                        format!(
                            "正在解析 {} · {}",
                            fig_path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("Figma 文件"),
                            progress.detail
                        ),
                    );
                },
            )
            .with_context(|| format!("Failed to convert extracted file: {}", fig_path.display()))?;
            let raw_json = vw_figma_json::convert_raw(&fig_bytes)
                .with_context(|| format!("Failed to generate raw JSON: {}", fig_path.display()))?;
            fs::write(
                fig_path.with_extension("figma.json"),
                serde_json::to_string_pretty(&figma_json)?,
            )
            .with_context(|| format!("Failed to write Figma JSON for: {}", fig_path.display()))?;
            fs::write(fig_path.with_extension("json"), serde_json::to_string_pretty(&json)?)
                .with_context(|| {
                    format!("Failed to write parsed JSON for: {}", fig_path.display())
                })?;
            fs::write(
                fig_path.with_extension("raw.json"),
                serde_json::to_string_pretty(&raw_json)?,
            )
            .with_context(|| format!("Failed to write raw JSON for: {}", fig_path.display()))?;
            processed_pages = processed_pages.saturating_add(file_total_pages);
        }
        return Ok(());
    }

    let figma_json = vw_figma_json::convert(bytes, Some(target_dir))
        .with_context(|| format!("Failed to export Figma JSON for: {}", input_path.display()))?;
    let total_pages = count_figma_pages(&figma_json).max(1);
    let json =
        figma_to_design_doc_with_base_dir_and_progress(bytes, Some(target_dir), |progress| {
            on_progress(progress.completed_pages, total_pages, progress.detail);
        })
        .with_context(|| format!("Failed to convert Figma file: {}", input_path.display()))?;
    let raw_json = vw_figma_json::convert_raw(bytes)
        .with_context(|| format!("Failed to generate raw JSON: {}", input_path.display()))?;
    fs::write(target_dir.join("canvas_figma.json"), serde_json::to_string_pretty(&figma_json)?)
        .with_context(|| {
            format!("Failed to write {}", target_dir.join("canvas_figma.json").display())
        })?;
    fs::write(target_dir.join("canvas.json"), serde_json::to_string_pretty(&json)?)
        .with_context(|| format!("Failed to write {}", target_dir.join("canvas.json").display()))?;
    fs::write(target_dir.join("canvas.raw.json"), serde_json::to_string_pretty(&raw_json)?)
        .with_context(|| {
            format!("Failed to write {}", target_dir.join("canvas.raw.json").display())
        })?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn find_fig_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    fn visit_dir(dir: &Path, fig_files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                visit_dir(&path, fig_files)?;
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("fig") {
                fig_files.push(path);
            }
        }
        Ok(())
    }

    let mut fig_files = Vec::new();
    visit_dir(dir, &mut fig_files)?;
    Ok(fig_files)
}

#[cfg(not(target_arch = "wasm32"))]
fn render_svg_to_png(svg_data: &str) -> Option<Vec<u8>> {
    use resvg::usvg::{self};
    use tiny_skia::{Pixmap, Transform};

    let mut opt = usvg::Options::default();
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    opt.fontdb = std::sync::Arc::new(fontdb);

    // Parse SVG
    let tree = usvg::Tree::from_str(svg_data, &opt).ok()?;

    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height())?;

    let mut pm = pixmap.as_mut();
    resvg::render(&tree, Transform::default(), &mut pm);

    pixmap.encode_png().ok()
}

