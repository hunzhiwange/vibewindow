//! 工作流应用级消息处理，负责应用元数据编辑和工作流文件生命周期操作。

use super::*;

/// 构建或更新 handle 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn handle(app: &mut App, message: WorkflowMessage) -> Option<Task<Message>> {
    Some(match message {
        WorkflowMessage::LoadSample => {
            match load_builtin_workflow() {
                Ok(loaded) => apply_loaded(app, loaded),
                Err(error) => app.workflow_state.set_error(error),
            }
            Task::none()
        }
        WorkflowMessage::OpenFile => {
            app.workflow_state.close_floating_panels();
            open_file()
        }
        WorkflowMessage::OpenFileFinished(result) => {
            match result {
                Ok(Some(loaded)) => apply_loaded(app, loaded),
                Ok(None) => {}
                Err(error) => app.workflow_state.set_error(error),
            }
            Task::none()
        }
        WorkflowMessage::SelectApp(id) => {
            if app.workflow_state.select_app(&id) {
                crate::apps::workflow::sync_top_tab(app);
            }
            Task::none()
        }
        WorkflowMessage::OpenCreateAppEditor => {
            app.workflow_state.open_create_editor();
            Task::none()
        }
        WorkflowMessage::OpenEditAppEditor(id) => {
            app.workflow_state.open_edit_editor(id.as_deref());
            Task::none()
        }
        WorkflowMessage::CloseAppEditor => {
            app.workflow_state.close_editor();
            Task::none()
        }
        WorkflowMessage::AppEditorNameChanged(value) => {
            app.workflow_state.set_editor_name(value);
            Task::none()
        }
        WorkflowMessage::AppEditorDescriptionChanged(value) => {
            app.workflow_state.set_editor_description(value);
            Task::none()
        }
        WorkflowMessage::AppEditorIconChanged(value) => {
            app.workflow_state.set_editor_icon(value);
            Task::none()
        }
        WorkflowMessage::AppEditorUseIconAsAnswerIconChanged(value) => {
            app.workflow_state.set_editor_use_icon_as_answer_icon(value);
            Task::none()
        }
        WorkflowMessage::AppEditorMaxActiveRequestsChanged(value) => {
            app.workflow_state.set_editor_max_active_requests_input(value);
            Task::none()
        }
        WorkflowMessage::SubmitAppEditor => {
            let mode = app.workflow_state.app_editor.as_ref().map(|draft| draft.mode.clone());
            let Some(mode) = mode else {
                return Some(Task::none());
            };

            let template = match mode {
                WorkflowAppEditorMode::Create | WorkflowAppEditorMode::Edit(_) => {
                    match create_blank_workflow(WorkflowAppMeta::default()) {
                        Ok(loaded) => loaded,
                        Err(error) => {
                            app.workflow_state.set_error(error);
                            return Some(Task::none());
                        }
                    }
                }
            };

            if let Err(error) = app.workflow_state.submit_editor(app.window_size, template) {
                app.workflow_state.set_error(error);
            } else {
                crate::apps::workflow::sync_top_tab(app);
            }
            Task::none()
        }
        _ => return None,
    })
}
