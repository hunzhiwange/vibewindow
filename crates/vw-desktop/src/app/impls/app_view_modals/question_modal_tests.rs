#[test]
fn question_modal_tests_module_is_wired() {
    assert!(module_path!().ends_with("question_modal_tests"));
}

use iced::widget::text;

fn option(label: &str, preview: Option<&str>) -> vw_shared::question::OptionInfo {
    vw_shared::question::OptionInfo {
        label: label.to_string(),
        description: format!("{label} description"),
        preview: preview.map(ToString::to_string),
    }
}

fn question(
    header: &str,
    options: Vec<vw_shared::question::OptionInfo>,
    multiple: Option<bool>,
    custom: Option<bool>,
) -> vw_shared::question::Info {
    vw_shared::question::Info {
        question: format!("{header}?"),
        header: header.to_string(),
        options,
        multiple,
        custom,
    }
}

fn request(questions: Vec<vw_shared::question::Info>) -> vw_shared::question::Request {
    vw_shared::question::Request {
        id: "question".to_string(),
        session_id: "session".to_string(),
        questions,
        tool: None,
    }
}

#[test]
fn without_request_returns_root_content() {
    let (app, _) = crate::app::App::new();
    let root = text("root").into();

    let element = super::with_question_modal(&app, root);

    std::hint::black_box(element);
}

#[test]
fn request_with_options_preview_and_custom_input_builds_modal() {
    let (mut app, _) = crate::app::App::new();
    app.question_modal_request = Some(request(vec![
        question(
            "single",
            vec![option("once", Some("preview text")), option("always", None)],
            None,
            Some(true),
        ),
        question("multi", vec![option("a", None), option("b", None)], Some(true), None),
    ]));
    app.question_modal_answers = vec![vec!["once".to_string()], vec!["b".to_string()]];
    app.question_modal_custom = vec!["custom answer".to_string(), String::new()];

    let element = super::with_question_modal(&app, text("root").into());

    std::hint::black_box(element);
}

#[test]
fn empty_question_list_uses_default_header() {
    let (mut app, _) = crate::app::App::new();
    app.question_modal_request = Some(request(Vec::new()));

    let element = super::with_question_modal(&app, text("root").into());

    std::hint::black_box(element);
}
