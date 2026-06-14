use crate::app::App;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::state::{DesignSettingsTab, DesignState};

fn app_with_design_state() -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(DesignDoc::default()));
    app
}

#[test]
fn toggle_variables_flips_panel_and_clears_variable_popovers() {
    let mut app = app_with_design_state();
    let state = app.active_design_state_mut().unwrap();
    state.active_variable_theme_menu = Some("Dark".to_string());
    state.renaming_variable_theme = Some("Dark".to_string());
    state.variable_theme_rename_value = "Night".to_string();
    state.confirm_delete_variable_theme = Some("Dark".to_string());
    state.current_variable_collection = Some("Theme".to_string());
    state.active_variable_collection_menu = Some("Theme".to_string());
    state.renaming_variable_collection = Some("Theme".to_string());
    state.variable_collection_rename_value = "Brand".to_string();
    state.confirm_delete_variable_collection = Some("Theme".to_string());
    state.active_variable_menu = Some("color".to_string());
    state.variable_move_target_picker = Some("color".to_string());
    state.renaming_variable = Some("color".to_string());
    state.variable_rename_value = "name".to_string();
    state.confirm_delete_variable = Some("color".to_string());
    state.show_add_variable_menu = true;

    let _ = super::settings::update(&mut app, DesignMessage::ToggleVariables);

    let state = app.active_design_state().unwrap();
    assert!(app.show_design_variables);
    assert_eq!(state.active_variable_theme_menu, None);
    assert_eq!(state.renaming_variable_theme, None);
    assert!(state.variable_theme_rename_value.is_empty());
    assert_eq!(state.current_variable_collection, None);
    assert_eq!(state.active_variable_collection_menu, None);
    assert_eq!(state.active_variable_menu, None);
    assert_eq!(state.variable_move_target_picker, None);
    assert_eq!(state.renaming_variable, None);
    assert!(!state.show_add_variable_menu);
}

#[test]
fn simple_settings_messages_update_app_flags() {
    let mut app = app_with_design_state();

    let _ = super::settings::update(&mut app, DesignMessage::ToggleShortcuts);
    let _ = super::settings::update(&mut app, DesignMessage::ToggleSettings);
    let _ = super::settings::update(
        &mut app,
        DesignMessage::DesignSettingsSelectTab(DesignSettingsTab::Chat),
    );
    let _ = super::settings::update(&mut app, DesignMessage::ToggleMouseWheelZoom(false));
    let _ = super::settings::update(&mut app, DesignMessage::ToggleSlotContent(false));
    let _ = super::settings::update(&mut app, DesignMessage::ToggleSlotOverflow(false));
    let initial_properties_panel = app.show_properties_panel;
    let _ = super::settings::update(&mut app, DesignMessage::TogglePropertiesPanel);

    assert!(app.show_design_shortcuts);
    assert!(app.show_design_settings);
    assert_eq!(app.design_settings_active_tab, DesignSettingsTab::Chat);
    assert!(!app.mouse_wheel_zoom_enabled);
    assert!(!app.show_slot_content);
    assert!(!app.show_slot_overflow);
    assert_eq!(app.show_properties_panel, !initial_properties_panel);
}

#[test]
fn unhandled_settings_message_is_noop() {
    let mut app = app_with_design_state();

    let _ = super::settings::update(&mut app, DesignMessage::Snapshot);

    assert!(!app.show_design_shortcuts);
    assert!(!app.show_design_settings);
}
