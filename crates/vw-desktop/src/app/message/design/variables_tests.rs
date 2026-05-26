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
