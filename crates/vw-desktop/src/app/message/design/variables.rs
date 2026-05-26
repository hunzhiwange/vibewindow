//! 设计变量面板消息处理。
//!
//! 本模块负责变量面板的交互状态与文档修改。

use iced::Task;

use crate::app::message::DesignMessage;
use crate::app::message::design::VariableKindPreset;
use crate::app::views::design::models::{DesignThemes, ThemeCondition, VariableDef, VariableValue};
use crate::app::{App, Message};

fn contains_mode(modes: &[String], target: &str) -> bool {
    modes.iter().any(|mode| mode.eq_ignore_ascii_case(target))
}

fn set_current_theme(
    state: &mut crate::app::views::design::state::DesignState,
    mode: impl Into<String>,
) {
    state.doc.theme = Some(ThemeCondition { mode: mode.into() });
    state.canvas_cache.clear();
}

fn unique_theme_name(modes: &[String]) -> String {
    for index in 1..=999 {
        let candidate = format!("Theme-{index}");
        if !contains_mode(modes, &candidate) {
            return candidate;
        }
    }
    format!("Theme-{}", modes.len() + 1)
}

fn unique_collection_name(names: &[String]) -> String {
    if !contains_mode(names, "Theme") {
        return "Theme".to_string();
    }

    for index in 1..=999 {
        let candidate = format!("Theme-{index}");
        if !contains_mode(names, &candidate) {
            return candidate;
        }
    }

    format!("Theme-{}", names.len() + 1)
}

fn unique_copy_name(base: &str, modes: &[String]) -> String {
    let first = format!("{base}-copy");
    if !contains_mode(modes, &first) {
        return first;
    }

    for index in 2..=999 {
        let candidate = format!("{base}-copy-{index}");
        if !contains_mode(modes, &candidate) {
            return candidate;
        }
    }

    format!("{base}-copy-{}", modes.len() + 1)
}

fn current_variable_collection_name(
    state: &crate::app::views::design::state::DesignState,
) -> String {
    state
        .current_variable_collection
        .clone()
        .or_else(|| state.doc.variable_collection_names().first().cloned())
        .unwrap_or_else(|| "Theme".to_string())
}

fn set_current_variable_collection(
    state: &mut crate::app::views::design::state::DesignState,
    name: impl Into<String>,
) {
    state.current_variable_collection = Some(name.into());
}

fn unique_variable_name(
    variables: &std::collections::HashMap<String, VariableDef>,
    kind: VariableKindPreset,
) -> String {
    let prefix = kind.default_name_prefix();
    for index in 1..=999 {
        let candidate = format!("{prefix}-{index}");
        if !variables.contains_key(&candidate) {
            return candidate;
        }
    }

    format!("{prefix}-{}", variables.len() + 1)
}

fn unique_variable_copy_name(
    variables: &std::collections::HashMap<String, VariableDef>,
    base: &str,
) -> String {
    let first = format!("{base}-copy");
    if !variables.contains_key(&first) {
        return first;
    }

    for index in 2..=999 {
        let candidate = format!("{base}-copy-{index}");
        if !variables.contains_key(&candidate) {
            return candidate;
        }
    }

    format!("{base}-copy-{}", variables.len() + 1)
}

pub(super) fn clear_variable_popovers(state: &mut crate::app::views::design::state::DesignState) {
    state.active_variable_theme_menu = None;
    state.confirm_delete_variable_theme = None;
    state.active_variable_menu = None;
    state.variable_move_target_picker = None;
    state.confirm_delete_variable = None;
    state.show_add_variable_menu = false;
}

fn clear_variable_collection_popovers(state: &mut crate::app::views::design::state::DesignState) {
    state.active_variable_collection_menu = None;
    state.confirm_delete_variable_collection = None;
}

pub(super) fn clear_all_variable_popovers(
    state: &mut crate::app::views::design::state::DesignState,
) {
    clear_variable_popovers(state);
    clear_variable_collection_popovers(state);
}

fn clear_variable_rename(state: &mut crate::app::views::design::state::DesignState) {
    state.renaming_variable = None;
    state.variable_rename_value.clear();
}

fn clear_collection_rename(state: &mut crate::app::views::design::state::DesignState) {
    state.renaming_variable_collection = None;
    state.variable_collection_rename_value.clear();
}

fn clear_theme_rename(state: &mut crate::app::views::design::state::DesignState) {
    state.renaming_variable_theme = None;
    state.variable_theme_rename_value.clear();
}

fn variable_belongs_to_collection(def: &VariableDef, collection: &str) -> bool {
    def.collection.as_ref().map(|current| current.eq_ignore_ascii_case(collection)).unwrap_or(false)
}

fn replace_variable_references_in_value(
    value: &mut serde_json::Value,
    old_name: &str,
    new_name: &str,
) {
    let old_token = format!("${old_name}");
    let new_token = format!("${new_name}");
    match value {
        serde_json::Value::String(text) => {
            if *text == old_token {
                *text = new_token;
            } else {
                let old_var = format!("var({old_token})");
                let new_var = format!("var({new_token})");
                if text.contains(&old_var) {
                    *text = text.replace(&old_var, &new_var);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                replace_variable_references_in_value(item, old_name, new_name);
            }
        }
        serde_json::Value::Object(map) => {
            for nested in map.values_mut() {
                replace_variable_references_in_value(nested, old_name, new_name);
            }
        }
        _ => {}
    }
}

fn rename_variable_references(
    state: &mut crate::app::views::design::state::DesignState,
    old_name: &str,
    new_name: &str,
) {
    if let Ok(mut children_value) = serde_json::to_value(&state.doc.children) {
        replace_variable_references_in_value(&mut children_value, old_name, new_name);
        if let Ok(children) = serde_json::from_value(children_value) {
            state.doc.children = children;
        }
    }

    if let Ok(mut variables_value) = serde_json::to_value(&state.doc.variables) {
        replace_variable_references_in_value(&mut variables_value, old_name, new_name);
        if let Ok(variables) = serde_json::from_value(variables_value) {
            state.doc.variables = variables;
        }
    }
}

fn upsert_variable_value(values: &mut Vec<VariableValue>, mode: Option<&str>, new_value: String) {
    let target_mode = mode.map(str::trim).filter(|value| !value.is_empty());
    if let Some(existing) = values.iter_mut().find(|entry| match (&entry.theme, target_mode) {
        (None, None) => true,
        (Some(theme), Some(target)) => theme.mode.eq_ignore_ascii_case(target),
        _ => false,
    }) {
        existing.value = new_value;
    } else {
        values.push(VariableValue {
            value: new_value,
            theme: target_mode.map(|target| ThemeCondition { mode: target.to_string() }),
        });
    }

    values.retain(|entry| !entry.value.trim().is_empty());
}

fn movable_variable_value_index(values: &[VariableValue]) -> Option<usize> {
    if let Some(index) = values.iter().position(|entry| entry.theme.is_none()) {
        return Some(index);
    }
    (values.len() == 1).then_some(0)
}

pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match message {
        DesignMessage::ToggleVariables => Task::none(),

        DesignMessage::SelectVariableCollection(name) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };
            let names = state.doc.ensure_variable_collections();
            if contains_mode(&names, &name) {
                set_current_variable_collection(state, name);
            }
            clear_all_variable_popovers(state);
            Task::none()
        }

        DesignMessage::ToggleVariableCollectionMenu(name) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let reopen = !state
                .active_variable_collection_menu
                .as_ref()
                .is_some_and(|active| active.eq_ignore_ascii_case(&name));
            clear_all_variable_popovers(state);
            if reopen {
                state.active_variable_collection_menu = Some(name);
            }
            clear_collection_rename(state);
            Task::none()
        }

        DesignMessage::CloseVariableCollectionMenu => {
            if let Some(state) = app.active_design_state_mut() {
                clear_variable_collection_popovers(state);
            }
            Task::none()
        }

        DesignMessage::AddVariableCollection => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let mut names = state.doc.ensure_variable_collections();
            let new_name = unique_collection_name(&names);
            names.push(new_name.clone());
            state.doc.variable_collections =
                Some(crate::app::views::design::models::VariableCollections { names });
            set_current_variable_collection(state, new_name);
            clear_all_variable_popovers(state);
            clear_collection_rename(state);
            Task::none()
        }

        DesignMessage::RenameVariableCollectionRequested(current) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            clear_all_variable_popovers(state);
            state.renaming_variable_collection = Some(current.clone());
            state.variable_collection_rename_value = current;
            Task::none()
        }

        DesignMessage::VariableCollectionRenameChanged(value) => {
            if let Some(state) = app.active_design_state_mut() {
                state.variable_collection_rename_value = value;
            }
            Task::none()
        }

        DesignMessage::SubmitVariableCollectionRename => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let Some(current) = state.renaming_variable_collection.clone() else {
                    return Task::none();
                };

                let renamed = state.variable_collection_rename_value.trim().to_string();
                if renamed.is_empty() {
                    notification = Some("主题名称不能为空".to_string());
                } else {
                    let mut names = state.doc.ensure_variable_collections();
                    if !current.eq_ignore_ascii_case(&renamed) && contains_mode(&names, &renamed) {
                        notification = Some("主题名称已存在".to_string());
                    } else {
                        for name in &mut names {
                            if name.eq_ignore_ascii_case(&current) {
                                *name = renamed.clone();
                            }
                        }
                        state.doc.variable_collections =
                            Some(crate::app::views::design::models::VariableCollections { names });
                        for def in state.doc.variables.values_mut() {
                            if variable_belongs_to_collection(def, &current) {
                                def.collection = Some(renamed.clone());
                            }
                        }
                        if state
                            .current_variable_collection
                            .as_ref()
                            .is_some_and(|name| name.eq_ignore_ascii_case(&current))
                        {
                            set_current_variable_collection(state, renamed.clone());
                        }
                        clear_collection_rename(state);
                        state.canvas_cache.clear();
                    }
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::CancelVariableCollectionRename => {
            if let Some(state) = app.active_design_state_mut() {
                clear_collection_rename(state);
            }
            Task::none()
        }

        DesignMessage::DuplicateVariableCollection(current) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let mut names = state.doc.ensure_variable_collections();
            let cloned = unique_copy_name(&current, &names);
            let insert_at = names
                .iter()
                .position(|name| name.eq_ignore_ascii_case(&current))
                .map(|index| index + 1)
                .unwrap_or(names.len());
            names.insert(insert_at, cloned.clone());
            state.doc.variable_collections =
                Some(crate::app::views::design::models::VariableCollections { names });

            let to_duplicate = state
                .doc
                .variables
                .iter()
                .filter(|(_, def)| variable_belongs_to_collection(def, &current))
                .map(|(name, def)| (name.clone(), def.clone()))
                .collect::<Vec<_>>();
            for (name, mut def) in to_duplicate {
                let new_name = unique_variable_copy_name(&state.doc.variables, &name);
                def.collection = Some(cloned.clone());
                state.doc.variables.insert(new_name, def);
            }

            set_current_variable_collection(state, cloned);
            clear_all_variable_popovers(state);
            state.canvas_cache.clear();
            Task::none()
        }

        DesignMessage::RequestDeleteVariableCollection(current) => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let names = state.doc.ensure_variable_collections();
                if names.len() <= 1 {
                    notification = Some("至少保留一个主题".to_string());
                    state.active_variable_collection_menu = None;
                } else {
                    clear_all_variable_popovers(state);
                    state.active_variable_collection_menu = Some(current.clone());
                    state.confirm_delete_variable_collection = Some(current);
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::ConfirmDeleteVariableCollection => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let Some(target) = state.confirm_delete_variable_collection.clone() else {
                    return Task::none();
                };

                let mut names = state.doc.ensure_variable_collections();
                let Some(remove_index) =
                    names.iter().position(|name| name.eq_ignore_ascii_case(&target))
                else {
                    state.confirm_delete_variable_collection = None;
                    return Task::none();
                };

                if names.len() <= 1 {
                    notification = Some("至少保留一个主题".to_string());
                    state.confirm_delete_variable_collection = None;
                } else {
                    names.remove(remove_index);
                    state.doc.variable_collections =
                        Some(crate::app::views::design::models::VariableCollections {
                            names: names.clone(),
                        });
                    state
                        .doc
                        .variables
                        .retain(|_, def| !variable_belongs_to_collection(def, &target));

                    if state
                        .current_variable_collection
                        .as_ref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(&target))
                    {
                        let fallback_index = remove_index.min(names.len().saturating_sub(1));
                        if let Some(next_name) = names.get(fallback_index).cloned() {
                            set_current_variable_collection(state, next_name);
                        }
                    }

                    state.confirm_delete_variable_collection = None;
                    state.active_variable_collection_menu = None;
                    clear_collection_rename(state);
                    state.canvas_cache.clear();
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::CancelDeleteVariableCollection => {
            if let Some(state) = app.active_design_state_mut() {
                state.confirm_delete_variable_collection = None;
            }
            Task::none()
        }

        DesignMessage::SelectVariableTheme(mode) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };
            let modes = state.doc.ensure_variable_themes();
            if contains_mode(&modes, &mode) {
                set_current_theme(state, mode);
            }
            clear_all_variable_popovers(state);
            Task::none()
        }

        DesignMessage::ToggleVariableThemeMenu(mode) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let reopen = !state
                .active_variable_theme_menu
                .as_ref()
                .is_some_and(|active| active.eq_ignore_ascii_case(&mode));
            clear_all_variable_popovers(state);
            if reopen {
                state.active_variable_theme_menu = Some(mode);
            }
            clear_variable_rename(state);
            Task::none()
        }

        DesignMessage::CloseVariableThemeMenu => {
            if let Some(state) = app.active_design_state_mut() {
                state.active_variable_theme_menu = None;
                state.confirm_delete_variable_theme = None;
            }
            Task::none()
        }

        DesignMessage::ToggleAddVariableMenu => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let next = !state.show_add_variable_menu;
            clear_all_variable_popovers(state);
            state.show_add_variable_menu = next;
            clear_theme_rename(state);
            clear_variable_rename(state);
            Task::none()
        }

        DesignMessage::CloseAddVariableMenu => {
            if let Some(state) = app.active_design_state_mut() {
                state.show_add_variable_menu = false;
            }
            Task::none()
        }

        DesignMessage::AddVariableTheme => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let mut modes = state.doc.ensure_variable_themes();
            let new_mode = unique_theme_name(&modes);
            modes.push(new_mode.clone());
            state.doc.themes = Some(DesignThemes { mode: modes });
            if state.doc.theme.is_none() {
                set_current_theme(state, new_mode);
            }
            clear_all_variable_popovers(state);
            clear_theme_rename(state);
            clear_variable_rename(state);
            Task::none()
        }

        DesignMessage::CreateVariable(kind) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let name = unique_variable_name(&state.doc.variables, kind);
            let collection = current_variable_collection_name(state);
            state.doc.variables.insert(
                name,
                VariableDef {
                    kind: kind.as_kind().to_string(),
                    collection: Some(collection),
                    value: vec![VariableValue {
                        value: kind.default_value().to_string(),
                        theme: None,
                    }],
                },
            );
            state.canvas_cache.clear();
            clear_all_variable_popovers(state);
            clear_variable_rename(state);
            Task::none()
        }

        DesignMessage::ToggleVariableMenu(name) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let was_open =
                state.active_variable_menu.as_ref().is_some_and(|active| active == &name)
                    && state.variable_move_target_picker.is_none()
                    && state.confirm_delete_variable.is_none();

            clear_all_variable_popovers(state);
            if !was_open {
                state.active_variable_menu = Some(name);
            }
            clear_variable_rename(state);
            clear_theme_rename(state);
            Task::none()
        }

        DesignMessage::CloseVariableMenu => {
            if let Some(state) = app.active_design_state_mut() {
                state.active_variable_menu = None;
                state.variable_move_target_picker = None;
                state.confirm_delete_variable = None;
            }
            Task::none()
        }

        DesignMessage::ToggleVariableMoveTargets(name) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let reopen =
                state.variable_move_target_picker.as_ref().is_none_or(|active| active != &name);
            clear_variable_collection_popovers(state);
            state.active_variable_menu = Some(name.clone());
            state.confirm_delete_variable = None;
            state.variable_move_target_picker = reopen.then_some(name);
            state.active_variable_theme_menu = None;
            state.confirm_delete_variable_theme = None;
            state.show_add_variable_menu = false;
            Task::none()
        }

        DesignMessage::RenameVariableRequested(current) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            clear_all_variable_popovers(state);
            clear_theme_rename(state);
            state.renaming_variable = Some(current.clone());
            state.variable_rename_value = current;
            Task::none()
        }

        DesignMessage::VariableRenameChanged(value) => {
            if let Some(state) = app.active_design_state_mut() {
                state.variable_rename_value = value;
            }
            Task::none()
        }

        DesignMessage::SubmitVariableRename => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let Some(current) = state.renaming_variable.clone() else {
                    return Task::none();
                };

                let renamed = state.variable_rename_value.trim().to_string();
                if renamed.is_empty() {
                    notification = Some("变量名称不能为空".to_string());
                } else if !current.eq_ignore_ascii_case(&renamed)
                    && state.doc.variables.contains_key(&renamed)
                {
                    notification = Some("变量名称已存在".to_string());
                } else if let Some(def) = state.doc.variables.remove(&current) {
                    state.doc.variables.insert(renamed.clone(), def);
                    rename_variable_references(state, &current, &renamed);
                    clear_variable_rename(state);
                    state.canvas_cache.clear();
                } else {
                    clear_variable_rename(state);
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::CancelVariableRename => {
            if let Some(state) = app.active_design_state_mut() {
                clear_variable_rename(state);
            }
            Task::none()
        }

        DesignMessage::DuplicateVariable(current) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };
            let Some(def) = state.doc.variables.get(&current).cloned() else {
                return Task::none();
            };

            let new_name = unique_variable_copy_name(&state.doc.variables, &current);
            state.doc.variables.insert(new_name, def);
            clear_all_variable_popovers(state);
            state.canvas_cache.clear();
            Task::none()
        }

        DesignMessage::MoveVariableTo(variable_name, target_mode) => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let Some(def) = state.doc.variables.get_mut(&variable_name) else {
                    return Task::none();
                };

                let Some(index) = movable_variable_value_index(&def.value) else {
                    notification = Some("该变量已有多个主题值，请直接编辑目标主题列".to_string());
                    app.base_notification = notification;
                    return Task::none();
                };

                let current_mode = def.value[index].theme.as_ref().map(|theme| theme.mode.as_str());
                let target_mode_trimmed =
                    target_mode.as_deref().map(str::trim).filter(|mode| !mode.is_empty());
                let same_target = match (current_mode, target_mode_trimmed) {
                    (None, None) => true,
                    (Some(current), Some(target)) => current.eq_ignore_ascii_case(target),
                    _ => false,
                };
                if same_target {
                    clear_all_variable_popovers(state);
                    return Task::none();
                }

                let target_exists = def.value.iter().enumerate().any(|(entry_index, entry)| {
                    if entry_index == index {
                        return false;
                    }
                    match (&entry.theme, target_mode_trimmed) {
                        (None, None) => true,
                        (Some(theme), Some(target)) => theme.mode.eq_ignore_ascii_case(target),
                        _ => false,
                    }
                });
                if target_exists {
                    notification = Some("目标主题已存在值".to_string());
                } else {
                    def.value[index].theme =
                        target_mode_trimmed.map(|mode| ThemeCondition { mode: mode.to_string() });
                    clear_all_variable_popovers(state);
                    state.canvas_cache.clear();
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::RequestDeleteVariable(current) => {
            if let Some(state) = app.active_design_state_mut() {
                clear_variable_collection_popovers(state);
                state.active_variable_menu = Some(current.clone());
                state.variable_move_target_picker = None;
                state.confirm_delete_variable = Some(current);
                state.active_variable_theme_menu = None;
                state.confirm_delete_variable_theme = None;
                state.show_add_variable_menu = false;
            }
            Task::none()
        }

        DesignMessage::ConfirmDeleteVariable => {
            if let Some(state) = app.active_design_state_mut()
                && let Some(target) = state.confirm_delete_variable.clone()
            {
                state.doc.variables.remove(&target);
                state.confirm_delete_variable = None;
                state.active_variable_menu = None;
                state.variable_move_target_picker = None;
                state.canvas_cache.clear();
            }
            Task::none()
        }

        DesignMessage::CancelDeleteVariable => {
            if let Some(state) = app.active_design_state_mut() {
                state.confirm_delete_variable = None;
                state.variable_move_target_picker = None;
            }
            Task::none()
        }

        DesignMessage::VariableValueChanged(variable_name, mode, value) => {
            if let Some(state) = app.active_design_state_mut()
                && let Some(def) = state.doc.variables.get_mut(&variable_name)
            {
                upsert_variable_value(&mut def.value, mode.as_deref(), value);
                state.canvas_cache.clear();
            }
            Task::none()
        }

        DesignMessage::RenameVariableThemeRequested(current) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            clear_all_variable_popovers(state);
            clear_variable_rename(state);
            state.variable_theme_rename_value = current.clone();
            state.renaming_variable_theme = Some(current);
            Task::none()
        }

        DesignMessage::VariableThemeRenameChanged(value) => {
            if let Some(state) = app.active_design_state_mut() {
                state.variable_theme_rename_value = value;
            }
            Task::none()
        }

        DesignMessage::SubmitVariableThemeRename => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let Some(current) = state.renaming_variable_theme.clone() else {
                    return Task::none();
                };

                let renamed = state.variable_theme_rename_value.trim().to_string();
                if renamed.is_empty() {
                    notification = Some("主题名称不能为空".to_string());
                } else {
                    let modes = state.doc.ensure_variable_themes();
                    if !current.eq_ignore_ascii_case(&renamed) && contains_mode(&modes, &renamed) {
                        notification = Some("主题名称已存在".to_string());
                    } else {
                        if let Some(themes) = state.doc.themes.as_mut() {
                            for mode in &mut themes.mode {
                                if mode.eq_ignore_ascii_case(&current) {
                                    *mode = renamed.clone();
                                }
                            }
                        }

                        if state
                            .doc
                            .theme
                            .as_ref()
                            .is_some_and(|theme| theme.mode.eq_ignore_ascii_case(&current))
                        {
                            set_current_theme(state, renamed.clone());
                        }

                        for def in state.doc.variables.values_mut() {
                            for value in &mut def.value {
                                if let Some(theme) = value.theme.as_mut()
                                    && theme.mode.eq_ignore_ascii_case(&current)
                                {
                                    theme.mode = renamed.clone();
                                }
                            }
                        }

                        clear_theme_rename(state);
                        state.canvas_cache.clear();
                    }
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::CancelVariableThemeRename => {
            if let Some(state) = app.active_design_state_mut() {
                clear_theme_rename(state);
            }
            Task::none()
        }

        DesignMessage::DuplicateVariableTheme(current) => {
            let Some(state) = app.active_design_state_mut() else {
                return Task::none();
            };

            let mut modes = state.doc.ensure_variable_themes();
            let cloned_mode = unique_copy_name(&current, &modes);
            let insert_at = modes
                .iter()
                .position(|mode| mode.eq_ignore_ascii_case(&current))
                .map(|index| index + 1)
                .unwrap_or(modes.len());
            modes.insert(insert_at, cloned_mode.clone());
            state.doc.themes = Some(DesignThemes { mode: modes });

            for def in state.doc.variables.values_mut() {
                let cloned_value = def
                    .value
                    .iter()
                    .find(|value| {
                        value
                            .theme
                            .as_ref()
                            .is_some_and(|theme| theme.mode.eq_ignore_ascii_case(&current))
                    })
                    .cloned()
                    .or_else(|| def.value.iter().find(|value| value.theme.is_none()).cloned());

                if let Some(mut value) = cloned_value {
                    value.theme = Some(ThemeCondition { mode: cloned_mode.clone() });
                    def.value.push(value);
                }
            }

            clear_all_variable_popovers(state);
            state.canvas_cache.clear();
            Task::none()
        }

        DesignMessage::RequestDeleteVariableTheme(current) => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let modes = state.doc.ensure_variable_themes();
                if modes.len() <= 1 {
                    notification = Some("至少保留一个主题".to_string());
                    state.active_variable_theme_menu = None;
                } else {
                    clear_variable_collection_popovers(state);
                    state.active_variable_theme_menu = Some(current.clone());
                    state.confirm_delete_variable_theme = Some(current);
                    state.active_variable_menu = None;
                    state.variable_move_target_picker = None;
                    state.confirm_delete_variable = None;
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::ConfirmDeleteVariableTheme => {
            let mut notification = None;
            if let Some(state) = app.active_design_state_mut() {
                let Some(target) = state.confirm_delete_variable_theme.clone() else {
                    return Task::none();
                };

                let mut modes = state.doc.ensure_variable_themes();
                let Some(remove_index) =
                    modes.iter().position(|mode| mode.eq_ignore_ascii_case(&target))
                else {
                    state.confirm_delete_variable_theme = None;
                    return Task::none();
                };

                if modes.len() <= 1 {
                    notification = Some("至少保留一个主题".to_string());
                    state.confirm_delete_variable_theme = None;
                } else {
                    modes.remove(remove_index);
                    state.doc.themes = Some(DesignThemes { mode: modes.clone() });

                    for def in state.doc.variables.values_mut() {
                        def.value.retain(|value| {
                            !value
                                .theme
                                .as_ref()
                                .is_some_and(|theme| theme.mode.eq_ignore_ascii_case(&target))
                        });
                    }

                    if state
                        .doc
                        .theme
                        .as_ref()
                        .is_some_and(|theme| theme.mode.eq_ignore_ascii_case(&target))
                    {
                        let fallback_index = remove_index.min(modes.len().saturating_sub(1));
                        if let Some(next_mode) = modes.get(fallback_index).cloned() {
                            set_current_theme(state, next_mode);
                        }
                    }

                    state.confirm_delete_variable_theme = None;
                    state.active_variable_theme_menu = None;
                    clear_theme_rename(state);
                    state.canvas_cache.clear();
                }
            }
            app.base_notification = notification;
            Task::none()
        }

        DesignMessage::CancelDeleteVariableTheme => {
            if let Some(state) = app.active_design_state_mut() {
                state.confirm_delete_variable_theme = None;
            }
            Task::none()
        }

        _ => Task::none(),
    }
}
