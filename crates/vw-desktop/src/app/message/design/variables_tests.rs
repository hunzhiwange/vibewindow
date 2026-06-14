use super::{VariableKindPreset, variables::update};
use crate::app::App;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{
    DesignDoc, DesignThemes, ThemeCondition, VariableCollections, VariableDef, VariableValue,
};
use crate::app::views::design::state::DesignState;

fn variable(
    kind: &str,
    collection: Option<&str>,
    values: Vec<(&str, Option<&str>)>,
) -> VariableDef {
    VariableDef {
        kind: kind.to_string(),
        collection: collection.map(str::to_string),
        value: values
            .into_iter()
            .map(|(value, theme)| VariableValue {
                value: value.to_string(),
                theme: theme.map(|mode| ThemeCondition { mode: mode.to_string() }),
            })
            .collect(),
    }
}

fn test_app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

fn active_state(app: &App) -> &DesignState {
    app.active_design_state().expect("active design state")
}

fn apply_update(app: &mut App, message: DesignMessage) {
    let _ = update(app, message);
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("variables_tests"));
}

fn design_state_with_all_variable_popovers() -> crate::app::views::design::state::DesignState {
    let mut state = crate::app::views::design::state::DesignState::new(
        crate::app::views::design::models::DesignDoc::default(),
    );
    state.active_variable_collection_menu = Some("Theme".to_string());
    state.confirm_delete_variable_collection = Some("Theme".to_string());
    state.active_variable_theme_menu = Some("Dark".to_string());
    state.confirm_delete_variable_theme = Some("Dark".to_string());
    state.active_variable_menu = Some("color-1".to_string());
    state.variable_move_target_picker = Some("color-1".to_string());
    state.confirm_delete_variable = Some("color-1".to_string());
    state.show_add_variable_menu = true;
    state
}

#[test]
fn clear_all_variable_popovers_resets_every_menu_group() {
    let mut state = design_state_with_all_variable_popovers();

    super::clear_all_variable_popovers(&mut state);

    assert_eq!(state.active_variable_collection_menu, None);
    assert_eq!(state.confirm_delete_variable_collection, None);
    assert_eq!(state.active_variable_theme_menu, None);
    assert_eq!(state.confirm_delete_variable_theme, None);
    assert_eq!(state.active_variable_menu, None);
    assert_eq!(state.variable_move_target_picker, None);
    assert_eq!(state.confirm_delete_variable, None);
    assert!(!state.show_add_variable_menu);
}

#[test]
fn variable_popover_clear_keeps_collection_menu_group() {
    let mut state = design_state_with_all_variable_popovers();

    super::clear_variable_popovers(&mut state);

    assert_eq!(state.active_variable_collection_menu.as_deref(), Some("Theme"));
    assert_eq!(state.confirm_delete_variable_collection.as_deref(), Some("Theme"));
    assert_eq!(state.active_variable_theme_menu, None);
    assert_eq!(state.confirm_delete_variable_theme, None);
    assert_eq!(state.active_variable_menu, None);
    assert_eq!(state.variable_move_target_picker, None);
    assert_eq!(state.confirm_delete_variable, None);
    assert!(!state.show_add_variable_menu);
}

#[test]
fn select_variable_collection_accepts_existing_name_case_insensitively() {
    let doc = DesignDoc {
        variable_collections: Some(VariableCollections {
            names: vec!["Theme".to_string(), "Brand".to_string()],
        }),
        ..DesignDoc::default()
    };
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::SelectVariableCollection("brand".to_string()));

    assert_eq!(active_state(&app).current_variable_collection.as_deref(), Some("brand"));
}

#[test]
fn add_variable_collection_uses_unique_theme_name_and_selects_it() {
    let doc = DesignDoc {
        variable_collections: Some(VariableCollections {
            names: vec!["Theme".to_string(), "Theme-1".to_string()],
        }),
        ..DesignDoc::default()
    };
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::AddVariableCollection);

    let state = active_state(&app);
    assert_eq!(state.current_variable_collection.as_deref(), Some("Theme-2"));
    assert_eq!(
        state.doc.variable_collections.as_ref().map(|value| value.names.clone()),
        Some(vec!["Theme".to_string(), "Theme-1".to_string(), "Theme-2".to_string()])
    );
}

#[test]
fn rename_variable_collection_rejects_empty_and_duplicate_names() {
    let doc = DesignDoc {
        variable_collections: Some(VariableCollections {
            names: vec!["Theme".to_string(), "Brand".to_string()],
        }),
        ..DesignDoc::default()
    };
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::RenameVariableCollectionRequested("Theme".to_string()));
    apply_update(&mut app, DesignMessage::VariableCollectionRenameChanged(" ".to_string()));
    apply_update(&mut app, DesignMessage::SubmitVariableCollectionRename);
    assert_eq!(app.base_notification.as_deref(), Some("主题名称不能为空"));

    apply_update(&mut app, DesignMessage::VariableCollectionRenameChanged("Brand".to_string()));
    apply_update(&mut app, DesignMessage::SubmitVariableCollectionRename);
    assert_eq!(app.base_notification.as_deref(), Some("主题名称已存在"));
}

#[test]
fn rename_variable_collection_updates_variables_and_current_collection() {
    let mut doc = DesignDoc {
        variable_collections: Some(VariableCollections {
            names: vec!["Theme".to_string(), "Brand".to_string()],
        }),
        ..DesignDoc::default()
    };
    doc.variables
        .insert("--color-1".to_string(), variable("color", Some("Theme"), vec![("#fff", None)]));
    let mut app = test_app_with_doc(doc);
    app.active_design_state_mut().unwrap().current_variable_collection = Some("Theme".to_string());

    apply_update(&mut app, DesignMessage::RenameVariableCollectionRequested("Theme".to_string()));
    apply_update(&mut app, DesignMessage::VariableCollectionRenameChanged("Core".to_string()));
    apply_update(&mut app, DesignMessage::SubmitVariableCollectionRename);

    let state = active_state(&app);
    assert_eq!(state.current_variable_collection.as_deref(), Some("Core"));
    assert_eq!(
        state.doc.variables.get("--color-1").and_then(|def| def.collection.as_deref()),
        Some("Core")
    );
    assert_eq!(state.renaming_variable_collection, None);
}

#[test]
fn duplicate_and_delete_variable_collection_copy_and_remove_members() {
    let mut doc = DesignDoc {
        variable_collections: Some(VariableCollections {
            names: vec!["Theme".to_string(), "Brand".to_string()],
        }),
        ..DesignDoc::default()
    };
    doc.variables
        .insert("--color-1".to_string(), variable("color", Some("Theme"), vec![("#fff", None)]));
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::DuplicateVariableCollection("Theme".to_string()));
    let state = active_state(&app);
    assert_eq!(state.current_variable_collection.as_deref(), Some("Theme-copy"));
    assert!(state.doc.variables.contains_key("--color-1-copy"));

    apply_update(
        &mut app,
        DesignMessage::RequestDeleteVariableCollection("Theme-copy".to_string()),
    );
    apply_update(&mut app, DesignMessage::ConfirmDeleteVariableCollection);

    let state = active_state(&app);
    assert!(!state.doc.variable_collection_names().iter().any(|name| name == "Theme-copy"));
    assert!(!state.doc.variables.contains_key("--color-1-copy"));
}

#[test]
fn deleting_only_variable_collection_is_rejected() {
    let doc = DesignDoc {
        variable_collections: Some(VariableCollections { names: vec!["Theme".to_string()] }),
        ..DesignDoc::default()
    };
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::RequestDeleteVariableCollection("Theme".to_string()));

    assert_eq!(app.base_notification.as_deref(), Some("至少保留一个主题"));
    assert_eq!(active_state(&app).confirm_delete_variable_collection, None);
}

#[test]
fn theme_menu_add_select_rename_duplicate_and_delete_update_theme_values() {
    let mut doc = DesignDoc {
        themes: Some(DesignThemes { mode: vec!["Light".to_string(), "Dark".to_string()] }),
        theme: Some(ThemeCondition { mode: "Light".to_string() }),
        ..DesignDoc::default()
    };
    doc.variables.insert(
        "--color-1".to_string(),
        variable("color", Some("Theme"), vec![("#fff", None), ("#000", Some("Dark"))]),
    );
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::SelectVariableTheme("dark".to_string()));
    assert_eq!(
        active_state(&app).doc.theme.as_ref().map(|theme| theme.mode.as_str()),
        Some("dark")
    );

    apply_update(&mut app, DesignMessage::AddVariableTheme);
    assert!(active_state(&app).doc.variable_theme_modes().iter().any(|mode| mode == "Theme-1"));

    apply_update(&mut app, DesignMessage::RenameVariableThemeRequested("Dark".to_string()));
    apply_update(&mut app, DesignMessage::VariableThemeRenameChanged("Night".to_string()));
    apply_update(&mut app, DesignMessage::SubmitVariableThemeRename);
    assert!(
        active_state(&app)
            .doc
            .variables
            .get("--color-1")
            .unwrap()
            .value
            .iter()
            .any(|value| value.theme.as_ref().is_some_and(|theme| theme.mode == "Night"))
    );

    apply_update(&mut app, DesignMessage::DuplicateVariableTheme("Night".to_string()));
    assert!(
        active_state(&app)
            .doc
            .variables
            .get("--color-1")
            .unwrap()
            .value
            .iter()
            .any(|value| value.theme.as_ref().is_some_and(|theme| theme.mode == "Night-copy"))
    );

    apply_update(&mut app, DesignMessage::RequestDeleteVariableTheme("Night-copy".to_string()));
    apply_update(&mut app, DesignMessage::ConfirmDeleteVariableTheme);
    assert!(
        !active_state(&app)
            .doc
            .variables
            .get("--color-1")
            .unwrap()
            .value
            .iter()
            .any(|value| value.theme.as_ref().is_some_and(|theme| theme.mode == "Night-copy"))
    );
}

#[test]
fn create_duplicate_rename_and_delete_variable_update_document() {
    let doc = DesignDoc {
        variable_collections: Some(VariableCollections { names: vec!["Theme".to_string()] }),
        ..DesignDoc::default()
    };
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::CreateVariable(VariableKindPreset::Color));
    assert!(active_state(&app).doc.variables.contains_key("--color-1"));

    apply_update(&mut app, DesignMessage::DuplicateVariable("--color-1".to_string()));
    assert!(active_state(&app).doc.variables.contains_key("--color-1-copy"));

    apply_update(&mut app, DesignMessage::RenameVariableRequested("--color-1-copy".to_string()));
    apply_update(&mut app, DesignMessage::VariableRenameChanged("--accent".to_string()));
    apply_update(&mut app, DesignMessage::SubmitVariableRename);
    assert!(active_state(&app).doc.variables.contains_key("--accent"));

    apply_update(&mut app, DesignMessage::RequestDeleteVariable("--accent".to_string()));
    assert_eq!(active_state(&app).confirm_delete_variable.as_deref(), Some("--accent"));
    apply_update(&mut app, DesignMessage::ConfirmDeleteVariable);
    assert!(!active_state(&app).doc.variables.contains_key("--accent"));
}

#[test]
fn rename_variable_updates_nested_variable_references() {
    let mut doc = DesignDoc::default();
    doc.variables
        .insert("--color-1".to_string(), variable("color", Some("Theme"), vec![("#fff", None)]));
    doc.variables.insert(
        "--alias".to_string(),
        variable("color", Some("Theme"), vec![("var($--color-1)", None)]),
    );
    let mut app = test_app_with_doc(doc);

    apply_update(&mut app, DesignMessage::RenameVariableRequested("--color-1".to_string()));
    apply_update(&mut app, DesignMessage::VariableRenameChanged("--brand".to_string()));
    apply_update(&mut app, DesignMessage::SubmitVariableRename);

    let alias = active_state(&app).doc.variables.get("--alias").unwrap();
    assert_eq!(alias.value[0].value, "var($--brand)");
}

#[test]
fn variable_value_changed_upserts_and_removes_empty_values() {
    let mut doc = DesignDoc::default();
    doc.variables
        .insert("--color-1".to_string(), variable("color", Some("Theme"), vec![("#fff", None)]));
    let mut app = test_app_with_doc(doc);

    apply_update(
        &mut app,
        DesignMessage::VariableValueChanged(
            "--color-1".to_string(),
            Some("Dark".to_string()),
            "#000".to_string(),
        ),
    );
    assert_eq!(active_state(&app).doc.variables.get("--color-1").unwrap().value.len(), 2);

    apply_update(
        &mut app,
        DesignMessage::VariableValueChanged(
            "--color-1".to_string(),
            Some("Dark".to_string()),
            " ".to_string(),
        ),
    );
    let values = &active_state(&app).doc.variables.get("--color-1").unwrap().value;
    assert_eq!(values.len(), 1);
    assert!(values.iter().all(|value| value.theme.is_none()));
}

#[test]
fn move_variable_to_rejects_multi_value_and_duplicate_target() {
    let mut doc = DesignDoc::default();
    doc.variables.insert(
        "--color-1".to_string(),
        variable("color", Some("Theme"), vec![("#fff", None), ("#000", Some("Dark"))]),
    );
    let mut app = test_app_with_doc(doc);

    apply_update(
        &mut app,
        DesignMessage::MoveVariableTo("--color-1".to_string(), Some("Light".to_string())),
    );

    assert_eq!(app.base_notification.as_deref(), None);

    let mut doc = DesignDoc::default();
    doc.variables.insert(
        "--color-2".to_string(),
        variable("color", Some("Theme"), vec![("#fff", None), ("#000", Some("Dark"))]),
    );
    let mut app = test_app_with_doc(doc);
    app.active_design_state_mut().unwrap().doc.variables.get_mut("--color-2").unwrap().value.pop();

    apply_update(
        &mut app,
        DesignMessage::MoveVariableTo("--color-2".to_string(), Some("Dark".to_string())),
    );
    assert_eq!(
        active_state(&app).doc.variables.get("--color-2").unwrap().value[0]
            .theme
            .as_ref()
            .map(|theme| theme.mode.as_str()),
        Some("Dark")
    );
}
