#[path = "handlers/editing.rs"]
mod editing;
#[path = "handlers/generation_controls.rs"]
mod generation_controls;
#[path = "handlers/generation_plan.rs"]
mod generation_plan;
#[path = "handlers/page_generation.rs"]
mod page_generation;
#[path = "handlers/persistence.rs"]
mod persistence;
#[path = "handlers/preview.rs"]
mod preview;

use crate::app::message::DesignMessage;
use crate::app::{App, Message};
use iced::Task;

/// 处理设计编辑器相关的消息并更新应用状态。
pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match message {
        DesignMessage::ViewElementHtml(id) => preview::view_element_html(app, id),
        DesignMessage::HtmlPreviewAction(action) => preview::html_preview_action(app, action),
        DesignMessage::CloseHtmlPreview => preview::close_html_preview(app),
        DesignMessage::DesignGenerationPromptAction(action) => {
            preview::design_generation_prompt_action(app, action)
        }
        DesignMessage::DesignGenerationLogEditorAction(action) => {
            preview::design_generation_log_editor_action(app, action)
        }
        DesignMessage::DesignGenerationCopyChatMessage(index) => {
            preview::design_generation_copy_chat_message(app, index)
        }
        DesignMessage::DesignGenerationSelectChatMessage(index) => {
            preview::design_generation_select_chat_message(app, index)
        }
        DesignMessage::DesignGenerationClearChatSelection => {
            preview::design_generation_clear_chat_selection(app)
        }
        DesignMessage::DesignGenerationShowAllLogs => preview::design_generation_show_all_logs(app),
        DesignMessage::DesignGenerationLoadLogFiles => preview::design_generation_load_log_files(app),
        DesignMessage::DesignGenerationLogFilesLoaded(files) => {
            preview::design_generation_log_files_loaded(app, files)
        }
        DesignMessage::ToggleDesignGenerationExecutorPopover => {
            generation_controls::toggle_design_generation_executor_popover(app)
        }
        DesignMessage::CloseDesignGenerationExecutorPopover => {
            generation_controls::close_design_generation_executor_popover(app)
        }
        DesignMessage::DesignGenerationAcpAgentSelected(agent) => {
            generation_controls::design_generation_acp_agent_selected(app, agent)
        }
        DesignMessage::ToggleDesignGenerationModelPopover => {
            generation_controls::toggle_design_generation_model_popover(app)
        }
        DesignMessage::CloseDesignGenerationModelPopover => {
            generation_controls::close_design_generation_model_popover(app)
        }
        DesignMessage::DesignGenerationModelSelected(model) => {
            generation_controls::design_generation_model_selected(app, model)
        }
        DesignMessage::DesignGenerationStyleSelected(style) => {
            generation_controls::design_generation_style_selected(app, style)
        }
        DesignMessage::DesignGenerationDeviceSelected(device) => {
            generation_controls::design_generation_device_selected(app, device)
        }
        DesignMessage::DesignGenerationModelChanged(model) => {
            generation_controls::design_generation_model_changed(app, model)
        }
        DesignMessage::DesignGenerationParallelPagesChanged(value) => {
            generation_controls::design_generation_parallel_pages_changed(app, value)
        }
        DesignMessage::DesignGenerationThemeSelected(theme) => {
            generation_controls::design_generation_theme_selected(app, theme)
        }
        DesignMessage::ToggleDesignGenerationThemePopover => {
            generation_controls::toggle_design_generation_theme_popover(app)
        }
        DesignMessage::CloseDesignGenerationThemePopover => {
            generation_controls::close_design_generation_theme_popover(app)
        }
        DesignMessage::ToggleDesignGenerationDevicePopover => {
            generation_controls::toggle_design_generation_device_popover(app)
        }
        DesignMessage::CloseDesignGenerationDevicePopover => {
            generation_controls::close_design_generation_device_popover(app)
        }
        DesignMessage::ToggleDesignGenerationStylePopover => {
            generation_controls::toggle_design_generation_style_popover(app)
        }
        DesignMessage::CloseDesignGenerationStylePopover => {
            generation_controls::close_design_generation_style_popover(app)
        }
        DesignMessage::DesignGenerationStreamTick => {
            generation_controls::design_generation_stream_tick(app)
        }

        DesignMessage::DesignGenerationCancel => generation_controls::design_generation_cancel(app),
        DesignMessage::DesignGenerationSubmit => generation_plan::design_generation_submit(app),
        DesignMessage::DesignGenerationCompleted(result) => {
            generation_plan::design_generation_completed(app, result)
        }
        DesignMessage::ToggleDesignPlannerPanelCollapsed => {
            generation_controls::toggle_design_planner_panel_collapsed(app)
        }
        DesignMessage::DesignPlannerSelectTab(tab) => {
            generation_controls::design_planner_select_tab(app, tab)
        }
        DesignMessage::OpenDesignPlannerQuickMenu => {
            generation_controls::open_design_planner_quick_menu(app)
        }
        DesignMessage::CloseDesignPlannerQuickMenu => {
            generation_controls::close_design_planner_quick_menu(app)
        }
        DesignMessage::DesignPlannerSetCorner(corner) => {
            generation_controls::design_planner_set_corner(app, corner)
        }
        DesignMessage::DesignPlannerNewChatSession => {
            generation_controls::design_planner_new_chat_session(app)
        }
        DesignMessage::DesignPlannerSelectChatSession(index) => {
            generation_controls::design_planner_select_chat_session(app, index)
        }
        DesignMessage::DesignGenerationApplyPartialRegenerate => {
            generation_controls::design_generation_apply_partial_regenerate(app)
        }
        DesignMessage::GenerateDesignPage(page_frame_id, module_id) => {
            page_generation::generate_design_page(app, page_frame_id, module_id)
        }
        DesignMessage::RegenerateDesignPage(page_frame_id, module_id) => {
            page_generation::regenerate_design_page(app, page_frame_id, module_id)
        }
        DesignMessage::SetDesignPageTargetFrame(page_frame_id, module_id, target_frame_id) => {
            generation_controls::set_design_page_target_frame(app, page_frame_id, module_id, target_frame_id)
        }
        DesignMessage::DesignPageGenerated {
            page_frame_id,
            page_task_id,
            result,
        } => page_generation::design_page_generated(app, page_frame_id, page_task_id, result),
        DesignMessage::AggregateDesignPage(page_frame_id, module_id) => {
            page_generation::aggregate_design_page(app, page_frame_id, module_id)
        }
        DesignMessage::SaveDesignProjectPen => persistence::save_design_project_pen(app),
        DesignMessage::DesignProjectPenSaved(result) => {
            persistence::design_project_pen_saved(app, result)
        }
        DesignMessage::SaveGeneratedPageAsPen(page_frame_id, module_id) => {
            persistence::save_generated_page_as_pen(app, page_frame_id, module_id)
        }
        DesignMessage::GeneratedPagePenSaved(result) => {
            persistence::generated_page_pen_saved(app, result)
        }
        DesignMessage::ImportGeneratedPenToPage(page_frame_id, module_id) => {
            persistence::import_generated_pen_to_page(page_frame_id, module_id)
        }
        DesignMessage::GeneratedPagePenImported {
            page_frame_id,
            page_task_id,
            result,
        } => persistence::generated_page_pen_imported(app, page_frame_id, page_task_id, result),
        DesignMessage::EditStart(id, content) => editing::edit_start(app, id, content),
        DesignMessage::EditContentChanged(content) => editing::edit_content_changed(app, content),
        DesignMessage::EditEditorAction(action) => editing::edit_editor_action(app, action),
        DesignMessage::EditSubmit => editing::edit_submit(app),
        DesignMessage::EditCancel => editing::edit_cancel(app),

        _ => Task::none(),
    }
}


#[cfg(test)]
#[path = "handlers/editing_tests.rs"]
mod editing_tests;

#[cfg(test)]
#[path = "handlers/generation_controls_tests.rs"]
mod generation_controls_tests;

#[cfg(test)]
#[path = "handlers/generation_plan_tests.rs"]
mod generation_plan_tests;

#[cfg(test)]
#[path = "handlers/page_generation_tests.rs"]
mod page_generation_tests;

#[cfg(test)]
#[path = "handlers/persistence_tests.rs"]
mod persistence_tests;

#[cfg(test)]
#[path = "handlers/preview_tests.rs"]
mod preview_tests;
