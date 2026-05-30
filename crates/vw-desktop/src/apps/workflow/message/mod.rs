//! # Workflow 消息模块
//!
//! 该模块定义 workflow 的消息类型，并将 UI、文件、导出和编辑操作分发到状态更新逻辑。

use super::model::{
    LoadedWorkflow, WorkflowAppMeta, WorkflowConnectionEndpoint, create_blank_workflow,
    load_builtin_workflow, load_document_from_path, load_document_from_value,
    suggested_workflow_file_name,
};
#[cfg(not(target_arch = "wasm32"))]
use super::model::{WorkflowViewport, load_document_from_text, serialize_workflow_yaml};
use super::state::{
    WorkflowAppEditorMode, WorkflowCanvasContextMenuTarget, WorkflowNodeEditorTab,
    WorkflowVariablePanelKind,
};
use crate::app::{App, Message};
use iced::widget::text_editor;
use iced::{Point, Task, Vector};

mod app;
mod canvas;
mod document;
mod node;

#[cfg(test)]
#[path = "app_tests.rs"]
mod app_tests;
#[cfg(test)]
#[path = "canvas_tests.rs"]
mod canvas_tests;
#[cfg(test)]
#[path = "document_tests.rs"]
mod document_tests;
#[cfg(test)]
#[path = "node_tests.rs"]
mod node_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub enum WorkflowMessage {
    LoadSample,
    OpenFile,
    OpenFileFinished(Result<Option<LoadedWorkflow>, String>),
    SelectApp(String),
    OpenCreateAppEditor,
    OpenEditAppEditor(Option<String>),
    CloseAppEditor,
    AppEditorNameChanged(String),
    AppEditorDescriptionChanged(String),
    AppEditorIconChanged(String),
    AppEditorUseIconAsAnswerIconChanged(bool),
    AppEditorMaxActiveRequestsChanged(String),
    SubmitAppEditor,
    ToggleQuickInsertPanel,
    InsertSuggestedNode(String),
    OpenCreateNodeEditor(String),
    OpenCreateNodeEditorAt(String, Point),
    CreateContextNode(String),
    OpenDownstreamNodePicker(String),
    OpenEditNodeEditor(Option<String>),
    CloseNodeEditor,
    NodeEditorTabSelected(WorkflowNodeEditorTab),
    NodeEditorTitleChanged(String),
    NodeEditorDescriptionChanged(String),
    NodeEditorDescriptionAction(text_editor::Action),
    NodeEditorStartAddVariable,
    NodeEditorStartVariableHovered(Option<usize>),
    NodeEditorStartRemoveVariable(usize),
    NodeEditorStartSelectVariable(usize),
    NodeEditorStartCloseVariableEditor,
    NodeEditorStartSubmitVariableEditor,
    NodeEditorStartVariableEditorLabelChanged(String),
    NodeEditorStartVariableEditorNameChanged(String),
    NodeEditorStartVariableEditorTypeChanged(String),
    NodeEditorStartVariableEditorRequiredChanged(bool),
    NodeEditorStartVariableEditorHiddenChanged(bool),
    NodeEditorStartVariableEditorDefaultChanged(String),
    NodeEditorStartVariableEditorDefaultAction(text_editor::Action),
    NodeEditorStartVariableEditorMaxLengthChanged(String),
    NodeEditorStartVariableEditorAddOption,
    NodeEditorStartVariableEditorRemoveOption(usize),
    NodeEditorStartVariableEditorOptionChanged(usize, String),
    NodeEditorStartVariableEditorToggleFileType(String),
    NodeEditorStartVariableEditorExtensionsChanged(String),
    NodeEditorStartVariableEditorUploadMethodChanged(String),
    NodeEditorStartVariableEditorPickDefaultFile,
    NodeEditorStartVariableEditorPickDefaultFileFinished(Result<Option<String>, String>),
    NodeEditorStartVariableEditorRemoveDefaultFile(usize),
    NodeEditorStartVariableEditorOpenDefaultFileUrlInput,
    NodeEditorStartVariableEditorCloseDefaultFileUrlInput,
    NodeEditorStartVariableEditorDefaultFileUrlChanged(String),
    NodeEditorStartVariableEditorSubmitDefaultFileUrl,
    NodeEditorStartVariableLabelChanged(usize, String),
    NodeEditorStartVariableNameChanged(usize, String),
    NodeEditorStartVariableTypeChanged(usize, String),
    NodeEditorStartVariableRequiredChanged(usize, bool),
    NodeEditorStartVariableDefaultChanged(usize, String),
    NodeEditorStartVariablePlaceholderChanged(usize, String),
    NodeEditorStartVariableHintChanged(usize, String),
    NodeEditorStartVariableMaxLengthChanged(usize, String),
    InsertDownstreamNode(String, String),
    InsertDownstreamNodeFromHandle(String, String, String),
    NodeEditorShowRawDataEditorChanged(bool),
    NodeEditorIfElseAddCase,
    NodeEditorIfElseCaseLogicalOperatorChanged(usize, String),
    NodeEditorIfElseAddCondition(usize),
    NodeEditorIfElseRemoveCondition(usize, usize),
    NodeEditorIfElseConditionSelectorChanged(usize, usize, String),
    NodeEditorIfElseConditionOperatorChanged(usize, usize, String),
    NodeEditorIfElseConditionValueChanged(usize, usize, String),
    NodeEditorIfElseConditionVarTypeChanged(usize, usize, String),
    NodeEditorKnowledgeQuerySelectorChanged(String),
    NodeEditorKnowledgeQueryAttachmentSelectorChanged(String),
    NodeEditorKnowledgeDatasetIdsChanged(String),
    NodeEditorKnowledgeRetrievalModeChanged(String),
    NodeEditorKnowledgeTopKChanged(String),
    NodeEditorKnowledgeScoreThresholdEnabledChanged(bool),
    NodeEditorKnowledgeScoreThresholdChanged(String),
    NodeEditorKnowledgeRerankingEnabledChanged(bool),
    NodeEditorKnowledgeSingleModelProviderChanged(String),
    NodeEditorKnowledgeSingleModelNameChanged(String),
    NodeEditorKnowledgeSingleModelModeChanged(String),
    NodeEditorToolProviderIdChanged(String),
    NodeEditorToolProviderTypeChanged(String),
    NodeEditorToolProviderNameChanged(String),
    NodeEditorToolNameChanged(String),
    NodeEditorToolLabelChanged(String),
    NodeEditorToolDescriptionChanged(String),
    NodeEditorToolCredentialIdChanged(String),
    NodeEditorToolPluginUniqueIdentifierChanged(String),
    NodeEditorToolParametersAction(text_editor::Action),
    NodeEditorToolConfigurationsAction(text_editor::Action),
    NodeEditorAgentStrategyProviderChanged(String),
    NodeEditorAgentStrategyNameChanged(String),
    NodeEditorAgentStrategyLabelChanged(String),
    NodeEditorAgentPluginUniqueIdentifierChanged(String),
    NodeEditorAgentOutputSchemaAction(text_editor::Action),
    NodeEditorAgentParametersAction(text_editor::Action),
    NodeEditorAgentMemoryEnabledChanged(bool),
    NodeEditorAgentMemoryWindowSizeChanged(String),
    NodeEditorAgentMemoryPromptAction(text_editor::Action),
    NodeEditorLlmProviderChanged(String),
    NodeEditorLlmModelNameChanged(String),
    NodeEditorLlmModelModeChanged(String),
    NodeEditorLlmEnableThinkingChanged(bool),
    NodeEditorLlmContextEnabledChanged(bool),
    NodeEditorLlmContextSelectorChanged(String),
    NodeEditorLlmSystemPromptAction(text_editor::Action),
    NodeEditorLlmUserPromptAction(text_editor::Action),
    NodeEditorLlmVisionEnabledChanged(bool),
    NodeEditorAnswerAction(text_editor::Action),
    NodeEditorCodeLanguageChanged(String),
    NodeEditorCodeAddInputVariable,
    NodeEditorCodeRemoveInputVariable(usize),
    NodeEditorCodeInputVariableNameChanged(usize, String),
    NodeEditorCodeInputVariableSelectorChanged(usize, String, String),
    NodeEditorCodeAddOutputVariable,
    NodeEditorCodeRemoveOutputVariable(usize),
    NodeEditorCodeOutputNameChanged(usize, String),
    NodeEditorCodeOutputTypeChanged(usize, String),
    NodeEditorCodeRetryEnabledChanged(bool),
    NodeEditorCodeRetryMaxRetriesChanged(u8),
    NodeEditorCodeRetryIntervalChanged(u16),
    NodeEditorCodeErrorStrategyChanged(String),
    NodeEditorCodeAction(text_editor::Action),
    NodeEditorCodeDefaultValueAction(text_editor::Action),
    NodeEditorDataAction(text_editor::Action),
    SubmitNodeEditor,
    OpenVariablePanel(WorkflowVariablePanelKind),
    CloseVariablePanel,
    OpenCreateEnvironmentVariableEditor,
    OpenEditEnvironmentVariableEditor(String),
    OpenCreateConversationVariableEditor,
    OpenEditConversationVariableEditor(String),
    CloseVariableEditor,
    VariableEditorNameChanged(String),
    VariableEditorDescriptionChanged(String),
    VariableEditorTypeChanged(String),
    VariableEditorValueAction(text_editor::Action),
    SubmitVariableEditor,
    DeleteEnvironmentVariable(String),
    DeleteConversationVariable(String),
    ToggleActionMenu,
    CloseFloatingPanels,
    SaveActiveApp,
    SaveActiveAppAs,
    SaveActiveAppFinished(Result<Option<String>, String>),
    ExportPng,
    ExportJpeg,
    ExportSvg,
    ExportFinished(Result<(), String>),
    Reload,
    SelectNode(String),
    SelectEdge(String),
    ClearSelection,
    PanBy(Vector),
    Zoom(f32, Option<Point>),
    ZoomSet(f32),
    ZoomFit,
    ToggleZoomMenu,
    NodeDragStart(String),
    NodeDragged(String, Vector),
    FinishNodeDrag,
    StartConnection(WorkflowConnectionEndpoint, Point),
    UpdateConnectionCursor(Point),
    FinishConnection(WorkflowConnectionEndpoint),
    CancelConnection,
    CancelInteraction,
    OpenCanvasContextMenu(WorkflowCanvasContextMenuTarget, Point, Point),
    CloseCanvasContextMenu,
    DuplicateSelectedNode,
    DeleteSelectedNode,
    DeleteSelectedEdge,
    DeleteNodeById(String),
    DeleteEdgeById(String),
    JumpToNode(String),
    Undo,
    Redo,
    DismissError,
}

pub fn update(app: &mut App, message: WorkflowMessage) -> Task<Message> {
    app::handle(app, message.clone())
        .or_else(|| node::handle(app, message.clone()))
        .or_else(|| document::handle(app, message.clone()))
        .or_else(|| canvas::handle(app, message))
        .unwrap_or_else(Task::none)
}

pub(super) fn apply_loaded(app: &mut App, loaded: LoadedWorkflow) {
    app.workflow_state.apply_loaded(loaded, app.window_size);
    crate::apps::workflow::sync_top_tab(app);
}

pub(super) fn save_active_app(app: &mut App, force_picker: bool) -> Task<Message> {
    let Some(entry) = app.workflow_state.active_entry_snapshot() else {
        return Task::none();
    };

    save_entry(entry, force_picker)
}

pub(super) fn suggested_new_node_position(app: &App) -> Point {
    let zoom = app.workflow_state.zoom.max(0.0001);
    let center_screen = Point::new(app.window_size.0 * 0.52, app.window_size.1 * 0.5);
    let offset = (app.workflow_state.document.nodes.len() % 6) as f32 * 28.0;
    Point::new(
        (center_screen.x - app.workflow_state.pan.x) / zoom + offset,
        (center_screen.y - app.workflow_state.pan.y) / zoom + offset,
    )
}

pub(super) fn save_entry(
    entry: super::state::WorkflowAppEntry,
    force_picker: bool,
) -> Task<Message> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (entry, force_picker);
        Task::perform(
            async { Err("Web 平台暂不支持保存本地工作流文件".to_string()) },
            |res| Message::WorkflowTool(WorkflowMessage::SaveActiveAppFinished(res)),
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Task::perform(
            async move {
                let default_name = suggested_workflow_file_name(&entry.meta.name);
                let path = if force_picker || entry.source_path.is_none() {
                    let file = rfd::AsyncFileDialog::new()
                        .add_filter("Dify Workflow", &["yml", "yaml"])
                        .set_file_name(&default_name)
                        .save_file()
                        .await;

                    let Some(file) = file else {
                        return Ok(None);
                    };

                    file.path().to_string_lossy().to_string()
                } else {
                    entry.source_path.clone().unwrap_or(default_name)
                };

                let content = serialize_workflow_yaml(
                    &entry.meta,
                    &entry.document,
                    &entry.environment_variables,
                    &entry.conversation_variables,
                    &entry.raw_root,
                    WorkflowViewport { x: entry.pan.x, y: entry.pan.y, zoom: entry.zoom },
                )?;
                std::fs::write(&path, content)
                    .map_err(|error| format!("写入工作流文件失败: {error}"))?;
                Ok(Some(path))
            },
            |res| Message::WorkflowTool(WorkflowMessage::SaveActiveAppFinished(res)),
        )
    }
}

pub(super) fn open_file() -> Task<Message> {
    #[cfg(target_arch = "wasm32")]
    {
        Task::perform(
            async { Err("Web 平台暂不支持本地工作流文件选择".to_string()) },
            |res| Message::WorkflowTool(WorkflowMessage::OpenFileFinished(res)),
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Task::perform(
            async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("Dify Workflow", &["yml", "yaml", "json"])
                    .pick_file()
                    .await;

                let Some(file) = file else {
                    return Ok(None);
                };

                let path = file.path().to_string_lossy().to_string();
                let bytes = file.read().await;
                let text = String::from_utf8_lossy(&bytes).to_string();
                let loaded = load_document_from_text(Some(path), text)?;

                Ok(Some(loaded))
            },
            |res| Message::WorkflowTool(WorkflowMessage::OpenFileFinished(res)),
        )
    }
}

pub(super) fn suggested_export_file_name(title: &str, extension: &str) -> String {
    let default_name = suggested_workflow_file_name(title);
    let stem = default_name.strip_suffix(".yml").unwrap_or(default_name.as_str());
    format!("{stem}.{extension}")
}

pub(super) fn export_svg(app: &mut App) -> Task<Message> {
    if app.workflow_state.active_app_id.is_none() {
        return Task::none();
    }

    app.workflow_state.close_floating_panels();
    let file_name = suggested_export_file_name(app.workflow_state.title(), "svg");
    let svg_data = crate::apps::workflow::canvas::export_svg(&app.workflow_state.document);

    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file = rfd::AsyncFileDialog::new().set_file_name(&file_name).save_file().await;
                if let Some(file) = file {
                    file.write(svg_data.as_bytes()).await.map_err(|error| error.to_string())?;
                    let _ = open::that(file.path());
                }
                Ok(())
            }

            #[cfg(target_arch = "wasm32")]
            {
                let _ = (file_name, svg_data);
                Err("Web 平台暂不支持导出工作流图片".to_string())
            }
        },
        |result| Message::WorkflowTool(WorkflowMessage::ExportFinished(result)),
    )
}

pub(super) fn export_png(app: &mut App) -> Task<Message> {
    if app.workflow_state.active_app_id.is_none() {
        return Task::none();
    }

    app.workflow_state.close_floating_panels();
    let file_name = suggested_export_file_name(app.workflow_state.title(), "png");
    let svg_data = crate::apps::workflow::canvas::export_svg(&app.workflow_state.document);

    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file = rfd::AsyncFileDialog::new().set_file_name(&file_name).save_file().await;
                if let Some(file) = file {
                    let png = crate::apps::mindmap::canvas::render_svg_to_png(&svg_data)
                        .ok_or_else(|| "导出 PNG 失败".to_string())?;
                    file.write(&png).await.map_err(|error| error.to_string())?;
                    let _ = open::that(file.path());
                }
                Ok(())
            }

            #[cfg(target_arch = "wasm32")]
            {
                let _ = (file_name, svg_data);
                Err("Web 平台暂不支持导出工作流图片".to_string())
            }
        },
        |result| Message::WorkflowTool(WorkflowMessage::ExportFinished(result)),
    )
}

pub(super) fn export_jpeg(app: &mut App) -> Task<Message> {
    if app.workflow_state.active_app_id.is_none() {
        return Task::none();
    }

    app.workflow_state.close_floating_panels();
    let file_name = suggested_export_file_name(app.workflow_state.title(), "jpg");
    let svg_data = crate::apps::workflow::canvas::export_svg(&app.workflow_state.document);

    Task::perform(
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let file = rfd::AsyncFileDialog::new().set_file_name(&file_name).save_file().await;
                if let Some(file) = file {
                    let png = crate::apps::mindmap::canvas::render_svg_to_png(&svg_data)
                        .ok_or_else(|| "导出 JPEG 失败".to_string())?;
                    let image = image::load_from_memory(&png)
                        .map_err(|error| format!("解码 PNG 失败: {error}"))?;
                    let rgb = image::DynamicImage::ImageRgba8(image.to_rgba8()).into_rgb8();
                    let mut jpeg_data = std::io::Cursor::new(Vec::new());
                    let mut encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_data, 90);
                    encoder
                        .encode(
                            rgb.as_raw(),
                            rgb.width(),
                            rgb.height(),
                            image::ExtendedColorType::Rgb8,
                        )
                        .map_err(|error| format!("编码 JPEG 失败: {error}"))?;
                    file.write(&jpeg_data.into_inner()).await.map_err(|error| error.to_string())?;
                    let _ = open::that(file.path());
                }
                Ok(())
            }

            #[cfg(target_arch = "wasm32")]
            {
                let _ = (file_name, svg_data);
                Err("Web 平台暂不支持导出工作流图片".to_string())
            }
        },
        |result| Message::WorkflowTool(WorkflowMessage::ExportFinished(result)),
    )
}

pub(super) fn export_finished(app: &mut App, result: Result<(), String>) {
    match result {
        Ok(()) => {
            app.workflow_state.status_message = Some("已导出工作流图片".to_string());
        }
        Err(error) => app.workflow_state.set_error(error),
    }
}
