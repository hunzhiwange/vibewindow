use super::*;
use crate::app::App;
use crate::app::state::QueryClassificationRuleInput;

fn app() -> App {
    App::new().0
}

#[test]
fn validates_and_updates_query_classification_rules() {
    let mut app = app();
    app.query_classification_settings.rules = vec![QueryClassificationRuleInput::default()];
    app.query_classification_settings.rules[0].priority_input.clear();
    assert!(validate(&app).is_ok());
    app.query_classification_settings.rules[0].category = "code".to_string();
    assert!(validate(&app).unwrap_err().contains("pattern"));
    app.query_classification_settings.rules[0].pattern = "rust".to_string();
    app.query_classification_settings.rules[0].category.clear();
    assert!(validate(&app).unwrap_err().contains("category"));
    app.query_classification_settings.rules[0].category = "code".to_string();
    app.query_classification_settings.rules[0].priority_input = "bad".to_string();
    assert!(validate(&app).unwrap_err().contains("priority"));

    let _ =
        update(&mut app, SettingsMessage::QueryClassification(QueryClassificationMessage::Refresh));
    let _ =
        update(&mut app, SettingsMessage::QueryClassification(QueryClassificationMessage::AddRule));
    let idx = app.query_classification_settings.rules.len() - 1;
    let _ = update(
        &mut app,
        SettingsMessage::QueryClassification(QueryClassificationMessage::CategoryChanged(
            idx,
            "code".to_string(),
        )),
    );
    assert!(
        app.query_classification_settings.save_error.as_deref().unwrap_or("").contains("pattern")
    );
    let _ = update(
        &mut app,
        SettingsMessage::QueryClassification(QueryClassificationMessage::PatternChanged(
            idx,
            " rust ".to_string(),
        )),
    );
    let _ = update(
        &mut app,
        SettingsMessage::QueryClassification(QueryClassificationMessage::PriorityChanged(
            idx,
            "3".to_string(),
        )),
    );
    assert_eq!(app.query_classification_settings.rules[idx].priority_input, "3");
    let _ = update(
        &mut app,
        SettingsMessage::QueryClassification(QueryClassificationMessage::EnabledToggled(true)),
    );
    assert!(app.query_classification_settings.enabled);
    let _ = update(
        &mut app,
        SettingsMessage::QueryClassification(QueryClassificationMessage::RemoveRule(idx)),
    );
}
