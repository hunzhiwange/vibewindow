//! 处理任务看板状态更新分支，将 UI 消息转换为应用状态变更和异步任务。
//!
//! 注释说明当前文件的职责边界，帮助调用方理解数据流与错误传播，
//! 不改变任何运行时行为。

use super::*;

/// 执行 update 对应的领域操作。
///
/// 参数由调用方提供，函数在当前模块的状态边界内完成处理。
/// 返回值表达处理结果；失败时保留错误信息给上层界面或调度逻辑。
pub(super) fn update(
    app: &mut crate::app::App,
    message: TaskBoardMessage,
) -> iced::Task<crate::app::Message> {
    dispatch_task_board_messages!(message,
TaskBoardMessage::ToggleImportMode(enabled) => {
    app.task_board_is_import_mode = enabled;
    iced::Task::none()
}
TaskBoardMessage::SetImportPromptFormat(format) => {
    app.task_board_import_prompt_format = format;
    iced::Task::none()
}
TaskBoardMessage::ToggleImportPromptCollapsed => {
    app.task_board_import_prompt_collapsed = !app.task_board_import_prompt_collapsed;
    iced::Task::none()
}
TaskBoardMessage::CopyImportPromptTemplate => {
    let selected_priority = parse_priority_or_default(
        &app.task_board_draft.priority,
        app.task_board_settings.default_priority,
    );
    let template = import_prompt_template(
        app.task_board_import_prompt_format,
        selected_priority,
        &app.task_board_draft.model,
        app.task_board_draft.acp_agent.as_deref(),
    );
    app.push_notification("已复制导入提示词模板".to_string());
    iced::clipboard::write(template).map(|_: ()| Message::None)
}
TaskBoardMessage::ImportEditorAction(action) => {
    app.task_board_import_editor.perform(action);
    iced::Task::none()
}
TaskBoardMessage::ImportFilePick => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        iced::Task::perform(
            async move {
                rfd::AsyncFileDialog::new()
                    .add_filter("导入文件", &["json", "csv", "tsv"])
                    .pick_file()
                    .await
                    .map(|handle| handle.path().to_string_lossy().to_string())
            },
            |picked| Message::TaskBoard(TaskBoardMessage::ImportFilePicked(picked)),
        )
    }
    #[cfg(target_arch = "wasm32")]
    {
        app.push_notification("当前平台暂不支持文件选择，请粘贴内容导入".to_string());
        iced::Task::none()
    }
}
TaskBoardMessage::ImportFilePicked(picked) => {
    if let Some(path) = picked {
        return iced::Task::perform(
            async move { std::fs::read_to_string(&path).map_err(|e| format!("读取导入文件失败: {}", e)) },
            |result| Message::TaskBoard(TaskBoardMessage::ImportFileLoaded(result)),
        );
    }
    iced::Task::none()
}
TaskBoardMessage::ImportFileLoaded(result) => {
    match result {
        Ok(content) => {
            app.task_board_import_editor = iced::widget::text_editor::Content::with_text(&content);
            app.push_notification("已将文件内容填入导入表单".to_string());
        }
        Err(err) => {
            app.push_notification(err);
        }
    }
    iced::Task::none()
}
TaskBoardMessage::InsertDemoData(template) => {
    if let Some(content) = import_demo_content(&template) {
        app.task_board_import_editor = iced::widget::text_editor::Content::with_text(content);
    }
    iced::Task::none()
}
TaskBoardMessage::ClearImportEditor => {
    app.task_board_import_editor = iced::widget::text_editor::Content::new();
    iced::Task::none()
}
TaskBoardMessage::ImportTasksSubmitted => {
    let content = app.task_board_import_editor.text().to_string();
    if content.trim().is_empty() {
        return iced::Task::none();
    }

    let mut tasks_to_create = Vec::new();
    let default_priority = app.task_board_settings.default_priority;

    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) {
        if let Some(arr) = json_val.as_array() {
            for item in arr {
                let priority = item
                    .get("priority")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u32)
                    .unwrap_or(default_priority);
                let prompt = item.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let mut task = Task::new(priority);
                task.prompt = prompt;
                if let Some(model) = item.get("model").and_then(|v| v.as_str()) {
                    task.model = model.to_string();
                } else {
                    task.model = app.task_board_draft.model.clone();
                }
                task.acp_agent = item
                    .get("acp_agent")
                    .and_then(|value| value.as_str())
                    .and_then(normalize_task_acp_agent_input)
                    .or_else(|| app.task_board_draft.acp_agent.clone());

                tasks_to_create.push(task);
            }
        }
    } else {
        let lines: Vec<&str> = content.lines().collect();
        if !lines.is_empty() {
            let header = lines[0].to_lowercase();
            let delimiter = if header.contains('\t') { '\t' } else { ',' };
            let headers: Vec<&str> =
                header.split(delimiter).map(|s| s.trim().trim_matches('"')).collect();

            let prompt_idx = headers.iter().position(|&h| h == "prompt" || h == "提示词");
            let priority_idx = headers.iter().position(|&h| h == "priority" || h == "优先级");
            let model_idx = headers.iter().position(|&h| h == "model" || h == "模型");
            let acp_agent_idx = headers.iter().position(|&h| {
                h == "acp_agent" || h == "智能体" || h == "acp智能体"
            });

            if prompt_idx.is_some() {
                for line in lines.iter().skip(1) {
                    if line.trim().is_empty() {
                        continue;
                    }
                    let parts: Vec<&str> =
                        line.split(delimiter).map(|s| s.trim().trim_matches('"')).collect();

                    let prompt = if let Some(idx) = prompt_idx {
                        parts.get(idx).unwrap_or(&"").to_string()
                    } else {
                        "".to_string()
                    };

                    let priority = if let Some(idx) = priority_idx {
                        parts
                            .get(idx)
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(default_priority)
                    } else {
                        default_priority
                    };

                    let mut task = Task::new(priority);
                    task.prompt = prompt;
                    task.model = model_idx
                        .and_then(|idx| parts.get(idx).copied())
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(str::to_string)
                        .unwrap_or_else(|| app.task_board_draft.model.clone());
                    task.acp_agent = acp_agent_idx
                        .and_then(|idx| parts.get(idx).copied())
                        .and_then(normalize_task_acp_agent_input)
                        .or_else(|| app.task_board_draft.acp_agent.clone());
                    tasks_to_create.push(task);
                }
            }
        }
    }

    if tasks_to_create.is_empty() {
        return iced::Task::none();
    }

    if let Some(project_path) = &app.project_path {
        let path = project_path.clone();
        let mut tasks = Vec::new();
        for task in tasks_to_create {
            let p = path.clone();
            tasks.push(iced::Task::perform(
                async move { crate::app::task::create_task(&p, task) },
                move |result| match result {
                    Ok(created_task) => Message::TaskBoard(TaskBoardMessage::TaskCreated(created_task)),
                    Err(e) => {
                        eprintln!("Failed to create task: {}", e);
                        Message::None
                    }
                },
            ));
        }
        return iced::Task::batch(tasks);
    }
    iced::Task::none()
}
    )
}
#[cfg(test)]
#[path = "import_tests.rs"]
mod import_tests;
