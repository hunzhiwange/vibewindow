use super::*;
use serde_yaml::Value;

fn validation_with_errors(paths: &[&str]) -> state::WorkflowNodeEditorValidation {
    state::WorkflowNodeEditorValidation {
        field_errors: paths
            .iter()
            .map(|path| state::WorkflowNodeValidationError {
                path: (*path).to_string(),
                message: format!("error for {path}"),
            })
            .collect(),
    }
}

fn if_else_case(
    conditions: Vec<state::WorkflowIfElseConditionDraft>,
) -> state::WorkflowIfElseCaseDraft {
    state::WorkflowIfElseCaseDraft {
        raw_case: Value::Null,
        case_id: "case-a".to_string(),
        logical_operator: "and".to_string(),
        conditions,
    }
}

fn if_else_condition() -> state::WorkflowIfElseConditionDraft {
    state::WorkflowIfElseConditionDraft {
        raw_condition: Value::Null,
        variable_selector_input: "start.text".to_string(),
        comparison_operator: "contains".to_string(),
        compare_value: "orders".to_string(),
        var_type: "string".to_string(),
    }
}

#[test]
fn visual_section_handles_empty_cases() {
    let cases = Vec::new();
    let validation = state::WorkflowNodeEditorValidation::default();

    let _element = build_if_else_visual_section(&cases, &validation);
}

#[test]
fn visual_section_renders_case_list() {
    let cases = vec![if_else_case(vec![if_else_condition()])];
    let validation = state::WorkflowNodeEditorValidation::default();

    let _element = build_if_else_visual_section(&cases, &validation);
}

#[test]
fn visual_section_renders_multiple_cases_with_mixed_conditions() {
    let cases = vec![
        if_else_case(vec![if_else_condition()]),
        state::WorkflowIfElseCaseDraft {
            raw_case: Value::Null,
            case_id: "case-b".to_string(),
            logical_operator: "or".to_string(),
            conditions: Vec::new(),
        },
    ];
    let validation = validation_with_errors(&["if_else.cases[1].conditions"]);

    let _element = build_if_else_visual_section(&cases, &validation);
}

#[test]
fn case_card_handles_empty_conditions_without_validation_error() {
    let case = if_else_case(Vec::new());
    let validation = state::WorkflowNodeEditorValidation::default();

    let _element = build_if_else_case_card(0, &case, &validation);
}

#[test]
fn case_card_renders_validation_errors_and_conditions() {
    let case = if_else_case(vec![if_else_condition()]);
    let validation = validation_with_errors(&[
        "if_else.cases[2].logical_operator",
        "if_else.cases[2].conditions",
        "if_else.cases[2].conditions[0].var_type",
        "if_else.cases[2].conditions[0].operator",
        "if_else.cases[2].conditions[0].selector",
        "if_else.cases[2].conditions[0].value",
    ]);

    let _element = build_if_else_case_card(2, &case, &validation);
}

#[test]
fn condition_card_renders_without_validation_errors() {
    let condition = if_else_condition();
    let validation = state::WorkflowNodeEditorValidation::default();

    let _element = build_if_else_condition_card(1, 3, &condition, &validation);
}

#[test]
fn condition_card_renders_field_validation_errors() {
    let condition = if_else_condition();
    let validation = validation_with_errors(&[
        "if_else.cases[1].conditions[3].var_type",
        "if_else.cases[1].conditions[3].operator",
        "if_else.cases[1].conditions[3].selector",
        "if_else.cases[1].conditions[3].value",
    ]);

    let _element = build_if_else_condition_card(1, 3, &condition, &validation);
}
