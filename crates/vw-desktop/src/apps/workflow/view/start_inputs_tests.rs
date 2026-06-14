use iced::widget::text_editor;
use serde_yaml::Value;

use super::*;

fn variable_with_type(input_type: &str) -> super::super::state::WorkflowStartVariableDraft {
    super::super::state::WorkflowStartVariableDraft {
        raw_variable: Value::Mapping(Default::default()),
        label: "新变量".to_string(),
        variable: "input".to_string(),
        input_type: input_type.to_string(),
        required: true,
        hidden: false,
        options: Vec::new(),
        allowed_file_types: Vec::new(),
        allowed_file_extensions: Vec::new(),
        allowed_file_extensions_input: String::new(),
        allowed_file_upload_methods: Vec::new(),
        default_value: String::new(),
        default_file_values: Vec::new(),
        placeholder: String::new(),
        hint: String::new(),
        max_length_input: "48".to_string(),
    }
}

fn editor_with_variable(
    variable: super::super::state::WorkflowStartVariableDraft,
) -> super::super::state::WorkflowStartVariableEditorDraft {
    let default_value_editor = text_editor::Content::with_text(&variable.default_value);
    let default_file_url_input = variable.default_value.clone();

    super::super::state::WorkflowStartVariableEditorDraft {
        mode: super::super::state::WorkflowStartVariableEditorMode::Create,
        variable,
        default_value_editor,
        default_file_url_input,
        show_default_file_url_input: false,
    }
}

#[test]
fn start_variable_file_type_options_returns_stable_options() {
    assert_eq!(
        start_variable_file_type_options(),
        [
            ("document", "文档", "pdf, doc, docx, txt, md"),
            ("image", "图片", "png, jpg, jpeg, webp, gif"),
            ("audio", "音频", "mp3, wav, m4a"),
            ("video", "视频", "mp4, mov, avi"),
            ("custom", "自定义", "手动填写扩展名"),
        ]
    );
}

#[test]
fn build_start_variable_option_editor_handles_empty_and_existing_options() {
    let empty_variable = variable_with_type("select");
    let _empty_element = build_start_variable_option_editor(&empty_variable);

    let mut variable = variable_with_type("select");
    variable.options = vec!["alpha".to_string(), " beta ".to_string()];

    let _element = build_start_variable_option_editor(&variable);
}

#[test]
fn build_start_variable_select_default_field_handles_empty_matching_and_missing_defaults() {
    let empty_variable = variable_with_type("select");
    let _empty_element = build_start_variable_select_default_field(&empty_variable);

    let mut matching_variable = variable_with_type("select");
    matching_variable.options = vec!["alpha".to_string(), " ".to_string(), "beta".to_string()];
    matching_variable.default_value = "beta".to_string();
    let _matching_element = build_start_variable_select_default_field(&matching_variable);

    let mut missing_variable = matching_variable.clone();
    missing_variable.default_value = "gamma".to_string();
    let _missing_element = build_start_variable_select_default_field(&missing_variable);
}

#[test]
fn build_start_variable_buttons_handle_selected_and_unselected_states() {
    let _selected_file_type =
        build_start_variable_file_type_button("document", "文档", "pdf, doc", true);
    let _unselected_file_type =
        build_start_variable_file_type_button("image", "图片", "png, jpg", false);

    let _selected_upload_method =
        build_start_variable_upload_method_button("local_file", "本地上传", true);
    let _unselected_upload_method =
        build_start_variable_upload_method_button("remote_url", "URL", false);

    let _default_file_button = build_start_variable_default_file_button(
        Icon::Link,
        "粘贴文件链接",
        WorkflowMessage::NodeEditorStartVariableEditorOpenDefaultFileUrlInput,
    );
}

#[test]
fn build_start_variable_file_settings_handles_local_remote_custom_and_default_files() {
    let mut variable = variable_with_type("file");
    variable.allowed_file_types = vec!["document".to_string(), "custom".to_string()];
    variable.allowed_file_upload_methods = vec!["local_file".to_string(), "remote_url".to_string()];
    variable.allowed_file_extensions_input = ".csv, .json".to_string();
    variable.default_file_values = vec!["https://example.test/a.pdf".to_string()];
    let mut editor = editor_with_variable(variable);
    editor.show_default_file_url_input = true;
    editor.default_file_url_input = "https://example.test/b.pdf".to_string();

    let _element = build_start_variable_file_settings(&editor);
}

#[test]
fn build_start_variable_file_settings_handles_file_list_and_upload_method_edges() {
    let mut file_list_variable = variable_with_type("file-list");
    file_list_variable.allowed_file_types = vec!["image".to_string()];
    file_list_variable.allowed_file_upload_methods = vec!["local_file".to_string()];
    file_list_variable.max_length_input = "3".to_string();
    file_list_variable.default_file_values = vec!["one.png".to_string(), "two.png".to_string()];
    let file_list_editor = editor_with_variable(file_list_variable);
    let _file_list_element = build_start_variable_file_settings(&file_list_editor);

    let mut remote_only_variable = variable_with_type("file");
    remote_only_variable.allowed_file_upload_methods = vec!["remote_url".to_string()];
    let remote_only_editor = editor_with_variable(remote_only_variable);
    let _remote_only_element = build_start_variable_file_settings(&remote_only_editor);

    let no_upload_editor = editor_with_variable(variable_with_type("file"));
    let _no_upload_element = build_start_variable_file_settings(&no_upload_editor);
}

#[test]
fn build_start_variable_file_default_value_handles_hidden_overlay_and_unsupported_type() {
    let mut file_variable = variable_with_type("file");
    file_variable.allowed_file_upload_methods = vec!["local_file".to_string()];
    let file_editor = editor_with_variable(file_variable);
    let _hidden_overlay_element = build_start_variable_file_default_value(&file_editor);

    let unsupported_editor = editor_with_variable(variable_with_type("text-input"));
    let _unsupported_element = build_start_variable_file_settings(&unsupported_editor);
}

#[test]
fn build_start_variable_card_handles_select_checkbox_text_paragraph_number_and_files() {
    let mut select_variable = variable_with_type("select");
    select_variable.options = vec!["alpha".to_string(), "beta".to_string()];
    select_variable.default_value = "alpha".to_string();
    let select_editor = editor_with_variable(select_variable);
    let _select_card = build_start_variable_card(&select_editor);

    let mut checked_variable = variable_with_type("checkbox");
    checked_variable.default_value = "true".to_string();
    let checked_editor = editor_with_variable(checked_variable);
    let _checked_card = build_start_variable_card(&checked_editor);

    let mut unchecked_variable = variable_with_type("checkbox");
    unchecked_variable.default_value = "false".to_string();
    let unchecked_editor = editor_with_variable(unchecked_variable);
    let _unchecked_card = build_start_variable_card(&unchecked_editor);

    let mut unknown_checkbox_variable = variable_with_type("checkbox");
    unknown_checkbox_variable.default_value = "maybe".to_string();
    let unknown_checkbox_editor = editor_with_variable(unknown_checkbox_variable);
    let _unknown_checkbox_card = build_start_variable_card(&unknown_checkbox_editor);

    let mut paragraph_variable = variable_with_type("paragraph");
    paragraph_variable.default_value = "line one\nline two".to_string();
    let paragraph_editor = editor_with_variable(paragraph_variable);
    let _paragraph_card = build_start_variable_card(&paragraph_editor);

    let mut number_variable = variable_with_type("number");
    number_variable.default_value = "not-a-number".to_string();
    let number_editor = editor_with_variable(number_variable);
    let _number_card = build_start_variable_card(&number_editor);

    let text_editor = editor_with_variable(variable_with_type("text-input"));
    let _text_card = build_start_variable_card(&text_editor);

    let file_editor = editor_with_variable(variable_with_type("file"));
    let _file_card = build_start_variable_card(&file_editor);
}
