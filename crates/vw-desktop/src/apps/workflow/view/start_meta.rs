//! 工作流起始节点元数据视图模块，提供内置变量、变量类型选项和校验提示。

use super::*;

/// 提供 start builtin variables 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn start_builtin_variables(state: &WorkflowState) -> Vec<StartBuiltinVariableItem> {
    let is_chat_mode = state.active_meta().map(|meta| meta.mode.contains("chat")).unwrap_or(true);

    let mut items = Vec::new();
    if is_chat_mode {
        items.push(StartBuiltinVariableItem {
            name: "userinput.query",
            value_type: "String",
            description: "用户当前这一轮输入的问题文本。",
            legacy: false,
        });
    }
    items.push(StartBuiltinVariableItem {
        name: "userinput.files",
        value_type: "Array[File]",
        description: "用户上传的文件列表，可作为文件型输入或后续节点的附件来源。",
        legacy: !is_chat_mode,
    });

    items
}

/// 提供 start variable badge 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn start_variable_badge(label: &'static str) -> Element<'static, Message> {
    container(text(label).size(11)).padding([4, 8]).style(value_card_style).into()
}

/// 提供 start variable value type label 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn start_variable_value_type_label(input_type: &str) -> &'static str {
    match input_type {
        "number" => "number",
        "checkbox" => "boolean",
        "file" => "file",
        "file-list" => "array[file]",
        _ => "string",
    }
}

/// 提供 start variable summary error 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn start_variable_summary_error(
    index: usize,
    validation: &super::state::WorkflowNodeEditorValidation,
) -> Option<String> {
    let mut errors = Vec::new();

    for path in [
        format!("start.variables[{index}].label"),
        format!("start.variables[{index}].variable"),
        format!("start.variables[{index}].max_length"),
        format!("start.variables[{index}].options"),
        format!("start.variables[{index}].default"),
        format!("start.variables[{index}].allowed_file_types"),
        format!("start.variables[{index}].allowed_file_extensions"),
        format!("start.variables[{index}].allowed_file_upload_methods"),
    ] {
        if let Some(error) = validation.first_error_for(&path) {
            errors.push(error.to_string());
        }
    }

    if errors.is_empty() { None } else { Some(errors.join(" · ")) }
}

/// 提供 start variable default error 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn start_variable_default_error(
    variable: &super::state::WorkflowStartVariableDraft,
) -> Option<&'static str> {
    if variable.input_type == "number"
        && !super::state::is_valid_start_variable_number_default_value(&variable.default_value)
    {
        Some("数字类型默认值必须是数字")
    } else {
        None
    }
}

/// 提供 start variable advanced hint 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn start_variable_advanced_hint(input_type: &str) -> Option<&'static str> {
    match input_type {
        _ => None,
    }
}

/// 提供 start variable type options 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn start_variable_type_options() -> Vec<StartVariableTypeOption> {
    vec![
        StartVariableTypeOption { input_type: "text-input", label: "文本", value_type: "string" },
        StartVariableTypeOption { input_type: "paragraph", label: "段落", value_type: "string" },
        StartVariableTypeOption {
            input_type: "select", label: "下拉选项", value_type: "string"
        },
        StartVariableTypeOption { input_type: "number", label: "数字", value_type: "number" },
        StartVariableTypeOption {
            input_type: "checkbox", label: "复选框", value_type: "boolean"
        },
        StartVariableTypeOption { input_type: "file", label: "单文件", value_type: "file" },
        StartVariableTypeOption {
            input_type: "file-list",
            label: "文件列表",
            value_type: "array[file]",
        },
    ]
}

/// 提供 selected start variable type option 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
pub(super) fn selected_start_variable_type_option(
    input_type: &str,
) -> Option<StartVariableTypeOption> {
    start_variable_type_options().into_iter().find(|option| option.input_type == input_type)
}

/// 构建 start variable type selector 对应的界面元素。
///
/// 参数由当前工作流状态或编辑草稿提供；返回值是可直接嵌入 iced 视图树的元素。
pub(super) fn build_start_variable_type_selector(input_type: &str) -> Element<'static, Message> {
    let options = start_variable_type_options();
    let selected = selected_start_variable_type_option(input_type);

    pick_list(options, selected, |option| {
        Message::WorkflowTool(WorkflowMessage::NodeEditorStartVariableEditorTypeChanged(
            option.input_type.to_string(),
        ))
    })
    .padding([10, 12])
    .text_size(13)
    .style(settings_pick_list_style)
    .menu_style(settings_pick_list_menu_style)
    .width(Length::Fill)
    .into()
}

/// 提供 start variable file type options 功能。
///
/// 参数和返回值遵循调用方所在模块的工作流约定，错误会显式向上传递或由 UI 状态承载。
#[allow(dead_code)]
pub(super) fn start_variable_file_type_options() -> [(&'static str, &'static str, &'static str); 5]
{
    [
        ("document", "文档", "PDF, DOCX"),
        ("image", "图片", "PNG, JPG"),
        ("audio", "音频", "MP3, WAV"),
        ("video", "视频", "MP4, MOV"),
        ("custom", "自定义", "手动填写扩展名"),
    ]
}

#[cfg(test)]
#[path = "start_meta_tests.rs"]
mod start_meta_tests;
