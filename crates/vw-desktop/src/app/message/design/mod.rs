//! 汇总设计消息子模块，并提供设计工具消息分发入口。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use crate::app::views::design::models::DesignTool;
use crate::app::{App, Message};
use iced::Task;

pub mod canvas;
pub mod clipboard;
pub mod editor;
mod figma;
pub mod history;
mod images;
pub mod io;
pub mod layer;
pub mod property;
pub mod settings;
mod tailwind;
mod types;
pub mod variables;

pub use types::{
    CanvasContextMenuAction, DesignMessage, ImageImportPayload, LayerAction, PageAction,
    VariableKindPreset,
};
#[cfg(test)]
use variables::{clear_all_variable_popovers, clear_variable_popovers};
use images::{load_image_tasks_from_document, load_image_tasks_from_fill_value};

/// update 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match &message {
        DesignMessage::ImageLoaded(_, _)
        | DesignMessage::ImportImageElement
        | DesignMessage::ImportFillImage(_, _)
        | DesignMessage::CloseImageImportDialog
        | DesignMessage::OpenStickyNoteDialog
        | DesignMessage::CloseStickyNoteDialog
        | DesignMessage::CreateStickyNote(_)
        | DesignMessage::ImageImportInputChanged(_)
        | DesignMessage::PasteImageImportInput
        | DesignMessage::ImageImportClipboardReceived(_)
        | DesignMessage::ChooseImageImportFile
        | DesignMessage::ImageImportFilePicked(_)
        | DesignMessage::SubmitImageImport
        | DesignMessage::ImageImportResolved(_) => {
            return images::update(app, message).unwrap_or_else(Task::none);
        }
        DesignMessage::ConvertHtmlToLayers(_)
        | DesignMessage::UpdateTailwindNodeClass(_, _, _)
        | DesignMessage::TailwindNodeClassCommit(_, _)
        | DesignMessage::TailwindNodeTextCommit(_, _)
        | DesignMessage::UpdateTailwindNodeText(_, _, _)
        | DesignMessage::UpdateTailwindHtml(_, _)
        | DesignMessage::DeleteTailwindNode(_, _) => {
            return tailwind::update(app, message).unwrap_or_else(Task::none);
        }
        DesignMessage::ToolSelected(DesignTool::ImportFigma)
        | DesignMessage::FigmaImportFilePicked(_)
        | DesignMessage::FigmaProgressTick
        | DesignMessage::FigmaFileImported(_) => {
            return figma::update(app, message).unwrap_or_else(Task::none);
        }
        _ => {}
    }

    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。

    match message {
        DesignMessage::ImageLoaded(_, _)
        | DesignMessage::ImportImageElement
        | DesignMessage::ImportFillImage(_, _)
        | DesignMessage::CloseImageImportDialog
        | DesignMessage::OpenStickyNoteDialog
        | DesignMessage::CloseStickyNoteDialog
        | DesignMessage::CreateStickyNote(_)
        | DesignMessage::ImageImportInputChanged(_)
        | DesignMessage::PasteImageImportInput
        | DesignMessage::ImageImportClipboardReceived(_)
        | DesignMessage::ChooseImageImportFile
        | DesignMessage::ImageImportFilePicked(_)
        | DesignMessage::SubmitImageImport
        | DesignMessage::ImageImportResolved(_) => images::update(app, message).unwrap_or_else(Task::none),
        DesignMessage::ConvertHtmlToLayers(_)
        | DesignMessage::UpdateTailwindNodeClass(_, _, _)
        | DesignMessage::TailwindNodeClassCommit(_, _)
        | DesignMessage::TailwindNodeTextCommit(_, _)
        | DesignMessage::UpdateTailwindNodeText(_, _, _)
        | DesignMessage::UpdateTailwindHtml(_, _)
        | DesignMessage::DeleteTailwindNode(_, _) => tailwind::update(app, message).unwrap_or_else(Task::none),
        DesignMessage::ToolSelected(DesignTool::ImportFigma)
        | DesignMessage::FigmaImportFilePicked(_)
        | DesignMessage::FigmaProgressTick
        | DesignMessage::FigmaFileImported(_) => figma::update(app, message).unwrap_or_else(Task::none),
        DesignMessage::DesignGenerationPromptAction(_)
        | DesignMessage::DesignGenerationLogEditorAction(_)
        | DesignMessage::DesignGenerationCopyChatMessage(_)
        | DesignMessage::DesignGenerationSelectChatMessage(_)
        | DesignMessage::DesignGenerationClearChatSelection
        | DesignMessage::DesignGenerationShowAllLogs
        | DesignMessage::DesignGenerationLoadLogFiles
        | DesignMessage::DesignGenerationLogFilesLoaded(_)
        | DesignMessage::ToggleDesignGenerationExecutorPopover
        | DesignMessage::CloseDesignGenerationExecutorPopover
        | DesignMessage::DesignGenerationAcpAgentSelected(_)
        | DesignMessage::ToggleDesignGenerationThemePopover
        | DesignMessage::CloseDesignGenerationThemePopover
        | DesignMessage::ToggleDesignGenerationDevicePopover
        | DesignMessage::CloseDesignGenerationDevicePopover
        | DesignMessage::ToggleDesignGenerationStylePopover
        | DesignMessage::CloseDesignGenerationStylePopover
        | DesignMessage::DesignGenerationDeviceSelected(_)
        | DesignMessage::DesignGenerationStyleSelected(_)
        | DesignMessage::ToggleDesignGenerationModelPopover
        | DesignMessage::CloseDesignGenerationModelPopover
        | DesignMessage::DesignGenerationModelSelected(_)
        | DesignMessage::DesignGenerationModelChanged(_)
        | DesignMessage::DesignGenerationParallelPagesChanged(_)
        | DesignMessage::DesignGenerationThemeSelected(_)
        | DesignMessage::DesignGenerationSubmit
        | DesignMessage::DesignGenerationCancel
        | DesignMessage::ToggleDesignPlannerPanelCollapsed
        | DesignMessage::DesignPlannerSelectTab(_)
        | DesignMessage::OpenDesignPlannerQuickMenu
        | DesignMessage::CloseDesignPlannerQuickMenu
        | DesignMessage::DesignPlannerSetCorner(_)
        | DesignMessage::DesignPlannerNewChatSession
        | DesignMessage::DesignPlannerSelectChatSession(_)
        | DesignMessage::DesignGenerationApplyPartialRegenerate
        | DesignMessage::DesignGenerationStreamTick
        | DesignMessage::DesignGenerationCompleted(_)
        | DesignMessage::GenerateDesignPage(_, _)
        | DesignMessage::RegenerateDesignPage(_, _)
        | DesignMessage::SetDesignPageTargetFrame(_, _, _)
        | DesignMessage::AggregateDesignPage(_, _)
        | DesignMessage::SaveDesignProjectPen
        | DesignMessage::DesignProjectPenSaved(_)
        | DesignMessage::SaveGeneratedPageAsPen(_, _)
        | DesignMessage::GeneratedPagePenSaved(_)
        | DesignMessage::ImportGeneratedPenToPage(_, _)
        | DesignMessage::DesignPageGenerated { .. }
        | DesignMessage::GeneratedPagePenImported { .. } => editor::update(app, message),
        DesignMessage::ToolSelected(DesignTool::ImportImage) => {
            Task::done(Message::Design(DesignMessage::ImportImageElement))
        }
        DesignMessage::Pan(_)
        | DesignMessage::Zoom(_, _)
        | DesignMessage::ToolSelected(_)
        | DesignMessage::ZoomIn
        | DesignMessage::ZoomOut
        | DesignMessage::ZoomFit
        | DesignMessage::ZoomSet(_)
        | DesignMessage::ZoomPresetSelected(_)
        | DesignMessage::FitToElement(_)
        | DesignMessage::ToggleContextPopover(_)
        | DesignMessage::ContextShapeGroupHover(_)
        | DesignMessage::SetIconFilter(_)
        | DesignMessage::SetToolbarIconFamilyTab(_)
        | DesignMessage::SelectToolbarIcon { .. }
        | DesignMessage::SetBrushColor(_)
        | DesignMessage::SetBrushWidth(_)
        | DesignMessage::UpdateContextShape(_)
        | DesignMessage::UpdateContextFill(_)
        | DesignMessage::UpdateContextBorder(_)
        | DesignMessage::CanvasContextMenuOpen(_, _)
        | DesignMessage::CanvasContextMenuClose
        | DesignMessage::CanvasContextMenuAction(_)
        | DesignMessage::EraseBrushAt(_, _)
        | DesignMessage::ToggleZoomMenu => canvas::update(app, message),

        DesignMessage::CreateElement { .. } => {
            let task = canvas::update(app, message);
            Task::batch(vec![task, Task::done(Message::Design(DesignMessage::Snapshot))])
        }

        DesignMessage::ReparentElements(_, _) => {
            let task = canvas::update(app, message);
            Task::batch(vec![task, Task::done(Message::Design(DesignMessage::Snapshot))])
        }

        DesignMessage::MeshCurveChanged(_, _, _) => {
            Task::done(Message::Design(DesignMessage::Snapshot))
        }

        DesignMessage::ElementSelected(_)
        | DesignMessage::SelectTailwindNode(_, _)
        | DesignMessage::ToggleTailwindInspectorCollapsed
        | DesignMessage::ToggleTailwindTreeCollapsed(_, _)
        | DesignMessage::LayerRowPressed(_)
        | DesignMessage::MultiSelect(_)
        | DesignMessage::ToggleNode(_)
        | DesignMessage::ToggleLayerPanel
        | DesignMessage::LayerPanelResizing(_)
        | DesignMessage::LayerDragStart(_)
        | DesignMessage::LayerDragOver(_)
        | DesignMessage::LayerHover(_)
        | DesignMessage::LayerHoverLeave
        | DesignMessage::LayerMenuHover(_)
        | DesignMessage::LayerMenuLeave
        | DesignMessage::LayerMenuToggle(_, _, _)
        | DesignMessage::LayerMenuClose => layer::update(app, message),

        DesignMessage::ToggleVisible(_)
        | DesignMessage::MoveLayerItem(_, _)
        | DesignMessage::LayerActionSelected(_, _)
        | DesignMessage::LayerDrop => {
            let task = layer::update(app, message);
            Task::batch(vec![task, Task::done(Message::Design(DesignMessage::Snapshot))])
        }

        DesignMessage::PropertyUpdate(_, _, _)
        | DesignMessage::PropertiesUpdate(_, _)
        | DesignMessage::BatchPropertiesUpdate(_)
        | DesignMessage::IconFamilySelected { .. }
        | DesignMessage::CreateGroup
        | DesignMessage::PageActionSelected(_, PageAction::Duplicate)
        | DesignMessage::PageActionSelected(_, PageAction::Delete)
        | DesignMessage::PageActionSelected(_, PageAction::MoveUp)
        | DesignMessage::PageActionSelected(_, PageAction::MoveDown)
        | DesignMessage::PageRenameSubmit => {
            let task = property::update(app, message);
            Task::batch(vec![task, Task::done(Message::Design(DesignMessage::Snapshot))])
        }

        DesignMessage::SetActiveGroup(_)
        | DesignMessage::NewGroupNameChanged(_)
        | DesignMessage::PageMenuToggle(_, _, _)
        | DesignMessage::PageMenuClose
        | DesignMessage::PageActionSelected(_, PageAction::Rename)
        | DesignMessage::PageRenameChanged(_)
        | DesignMessage::PageRenameCancel => property::update(app, message),

        DesignMessage::SelectFill(_)
        | DesignMessage::SelectEffect(_)
        | DesignMessage::SetTailwindFilter(_)
        | DesignMessage::SetFontFilter(_)
        | DesignMessage::OpenTailwindClassPicker(_, _)
        | DesignMessage::CloseTailwindClassPicker
        | DesignMessage::TailwindClassInputChanged(_, _)
        | DesignMessage::TailwindClassInputSubmit(_)
        | DesignMessage::AddTailwindClassToken(_, _)
        | DesignMessage::TailwindInspectorHover(_)
        | DesignMessage::TailwindNodeClassInputChanged(_, _, _)
        | DesignMessage::TailwindNodeClassInputSubmit(_, _)
        | DesignMessage::TailwindNodeClassDropdownClose(_, _) => property::update(app, message),

        DesignMessage::PropertyUpdateTransient(_, _, _)
        | DesignMessage::PropertiesUpdateTransient(_, _)
        | DesignMessage::BatchPropertiesUpdateTransient(_) => property::update(app, message),

        DesignMessage::ContextEditorAction(_)
        | DesignMessage::ToggleContextEditor
        | DesignMessage::ContentEditorAction(_)
        | DesignMessage::TailwindHtmlEditorAction(_)
        | DesignMessage::TailwindNodeClassEditorAction(_)
        | DesignMessage::TailwindNodeTextEditorAction(_)
        | DesignMessage::OpenColorPicker(_, _, _)
        | DesignMessage::OpenFillPicker(_, _, _)
        | DesignMessage::CloseFillPicker
        | DesignMessage::OpenEffectPicker(_, _, _)
        | DesignMessage::CloseEffectPicker
        | DesignMessage::OpenFontPicker(_, _)
        | DesignMessage::CloseFontPicker
        | DesignMessage::OpenIconPicker(_, _)
        | DesignMessage::CloseIconPicker
        | DesignMessage::SetIconPickerFilter(_)
        | DesignMessage::SetIconPickerFamilyTab(_)
        | DesignMessage::FillPickerColorChange(_)
        | DesignMessage::FillPickerFormatChange(_)
        | DesignMessage::FillPickerEyedropper
        | DesignMessage::ColorPickerChange(_)
        | DesignMessage::ColorPickerFormatChange(_)
        | DesignMessage::ColorPickerEyedropper
        | DesignMessage::PickColor(_)
        | DesignMessage::CloseColorPicker
        | DesignMessage::FontPickerSelect(_, _)
        | DesignMessage::IconPickerSelect { .. }
        | DesignMessage::ShowHelpModal(_)
        | DesignMessage::CloseHelpModal => property::update(app, message),

        DesignMessage::EditSubmit => {
            let task = editor::update(app, message);
            Task::batch(vec![task, Task::done(Message::Design(DesignMessage::Snapshot))])
        }

        DesignMessage::EditStart(_, _)
        | DesignMessage::EditContentChanged(_)
        | DesignMessage::EditEditorAction(_)
        | DesignMessage::EditCancel
        | DesignMessage::ViewElementHtml(_)
        | DesignMessage::HtmlPreviewAction(_)
        | DesignMessage::CloseHtmlPreview => editor::update(app, message),

        DesignMessage::New
        | DesignMessage::Open
        | DesignMessage::ParseFigma
        | DesignMessage::FigmaParseCompleted(_)
        | DesignMessage::Save
        | DesignMessage::SaveAs
        | DesignMessage::ExportHtml
        | DesignMessage::ExportElementHtml(_)
        | DesignMessage::ExportElementSvg(_)
        | DesignMessage::ExportElementPng(_)
        | DesignMessage::ExportElementJpeg(_)
        | DesignMessage::FileOpened(_)
        | DesignMessage::FileSaved(_) => io::update(app, message),

        DesignMessage::AddVariableCollection
        | DesignMessage::AddVariableTheme
        | DesignMessage::CreateVariable(_)
        | DesignMessage::SubmitVariableCollectionRename
        | DesignMessage::DuplicateVariableCollection(_)
        | DesignMessage::ConfirmDeleteVariableCollection
        | DesignMessage::SubmitVariableRename
        | DesignMessage::DuplicateVariable(_)
        | DesignMessage::MoveVariableTo(_, _)
        | DesignMessage::ConfirmDeleteVariable
        | DesignMessage::VariableValueChanged(_, _, _)
        | DesignMessage::SubmitVariableThemeRename
        | DesignMessage::DuplicateVariableTheme(_)
        | DesignMessage::ConfirmDeleteVariableTheme => {
            let should_snapshot = matches!(
                message,
                DesignMessage::AddVariableCollection
                    | DesignMessage::AddVariableTheme
                    | DesignMessage::CreateVariable(_)
                    | DesignMessage::SubmitVariableCollectionRename
                    | DesignMessage::DuplicateVariableCollection(_)
                    | DesignMessage::ConfirmDeleteVariableCollection
                    | DesignMessage::SubmitVariableRename
                    | DesignMessage::DuplicateVariable(_)
                    | DesignMessage::MoveVariableTo(_, _)
                    | DesignMessage::ConfirmDeleteVariable
                    | DesignMessage::SubmitVariableThemeRename
                    | DesignMessage::DuplicateVariableTheme(_)
                    | DesignMessage::ConfirmDeleteVariableTheme
            );
            let task = variables::update(app, message);
            if should_snapshot {
                Task::batch(vec![task, Task::done(Message::Design(DesignMessage::Snapshot))])
            } else {
                task
            }
        }

        DesignMessage::SelectVariableCollection(_)
        | DesignMessage::ToggleVariableCollectionMenu(_)
        | DesignMessage::CloseVariableCollectionMenu
        | DesignMessage::RenameVariableCollectionRequested(_)
        | DesignMessage::VariableCollectionRenameChanged(_)
        | DesignMessage::CancelVariableCollectionRename
        | DesignMessage::RequestDeleteVariableCollection(_)
        | DesignMessage::CancelDeleteVariableCollection
        | DesignMessage::SelectVariableTheme(_)
        | DesignMessage::ToggleVariableThemeMenu(_)
        | DesignMessage::CloseVariableThemeMenu
        | DesignMessage::ToggleAddVariableMenu
        | DesignMessage::CloseAddVariableMenu
        | DesignMessage::ToggleVariableMenu(_)
        | DesignMessage::CloseVariableMenu
        | DesignMessage::ToggleVariableMoveTargets(_)
        | DesignMessage::RenameVariableRequested(_)
        | DesignMessage::VariableRenameChanged(_)
        | DesignMessage::CancelVariableRename
        | DesignMessage::RequestDeleteVariable(_)
        | DesignMessage::CancelDeleteVariable
        | DesignMessage::RenameVariableThemeRequested(_)
        | DesignMessage::VariableThemeRenameChanged(_)
        | DesignMessage::CancelVariableThemeRename
        | DesignMessage::RequestDeleteVariableTheme(_)
        | DesignMessage::CancelDeleteVariableTheme => variables::update(app, message),

        DesignMessage::ToggleVariables
        | DesignMessage::ToggleShortcuts
        | DesignMessage::ToggleSettings
        | DesignMessage::DesignSettingsSelectTab(_)
        | DesignMessage::ToggleMouseWheelZoom(_)
        | DesignMessage::ToggleSlotContent(_)
        | DesignMessage::ToggleSlotOverflow(_)
        | DesignMessage::TogglePropertiesPanel => settings::update(app, message),

        DesignMessage::Undo | DesignMessage::Redo | DesignMessage::Snapshot => {
            history::update(app, message)
        }

        DesignMessage::Cut
        | DesignMessage::Copy
        | DesignMessage::Paste
        | DesignMessage::ClipboardContentReceived(_) => clipboard::update(app, message),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "clipboard_tests.rs"]
mod clipboard_tests;

#[cfg(test)]
#[path = "figma_tests.rs"]
mod figma_tests;

#[cfg(test)]
#[path = "history_tests.rs"]
mod history_tests;

#[cfg(test)]
#[path = "images_tests.rs"]
mod images_tests;

#[cfg(test)]
#[path = "io_tests.rs"]
mod io_tests;

#[cfg(test)]
#[path = "layer_tests.rs"]
mod layer_tests;

#[cfg(test)]
#[path = "property_tests.rs"]
mod property_tests;

#[cfg(test)]
#[path = "settings_tests.rs"]
mod settings_tests;

#[cfg(test)]
#[path = "tailwind_tests.rs"]
mod tailwind_tests;

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

#[cfg(test)]
#[path = "variables_tests.rs"]
mod variables_tests;
