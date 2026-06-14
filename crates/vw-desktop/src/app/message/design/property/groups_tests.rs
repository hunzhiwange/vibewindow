use crate::app::App;
use crate::app::views::design::models::{DesignDoc, DesignElement, DesignGroup};
use crate::app::views::design::state::DesignState;

fn app_with_doc(doc: DesignDoc) -> App {
    let mut app = App::new().0;
    let tab_id = "design".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(tab_id, DesignState::new(doc));
    app
}

fn doc_with_groups() -> DesignDoc {
    DesignDoc {
        groups: vec![
            DesignGroup { id: 1, name: "One".to_string() },
            DesignGroup { id: 2, name: "Two".to_string() },
        ],
        children: vec![
            DesignElement { id: "a".to_string(), group_id: 1, ..Default::default() },
            DesignElement { id: "b".to_string(), group_id: 2, ..Default::default() },
        ],
        ..Default::default()
    }
}

#[test]
fn set_active_group_focuses_first_element_and_closes_page_menu() {
    let mut app = app_with_doc(doc_with_groups());
    let state = app.active_design_state_mut().unwrap();
    state.active_group_id = 1;
    state.active_page_menu = Some(2);
    state.page_menu_anchor = Some(iced::Point::new(1.0, 2.0));

    let _ = super::groups::set_active_group(&mut app, 2);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.active_group_id, 2);
    assert_eq!(state.selected_element_id.as_deref(), Some("b"));
    assert_eq!(state.active_page_menu, None);
    assert_eq!(state.page_menu_anchor, None);
}

#[test]
fn set_active_group_ignores_unknown_group() {
    let mut app = app_with_doc(doc_with_groups());
    app.active_design_state_mut().unwrap().active_group_id = 1;

    let _ = super::groups::set_active_group(&mut app, 99);

    assert_eq!(app.active_design_state().unwrap().active_group_id, 1);
}

#[test]
fn create_group_uses_trimmed_name_and_clears_selection() {
    let mut app = app_with_doc(doc_with_groups());
    let state = app.active_design_state_mut().unwrap();
    state.new_group_name = "  Landing  ".to_string();
    state.selected_element_id = Some("a".to_string());
    state.selected_element_ids.insert("a".to_string());
    state.selected_fill_index = Some(0);
    state.selected_effect_index = Some(1);

    let _ = super::groups::create_group(&mut app);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.active_group_id, 3);
    assert_eq!(state.doc.group_name(3), Some("Landing"));
    assert!(state.new_group_name.is_empty());
    assert_eq!(state.selected_element_id, None);
    assert!(state.selected_element_ids.is_empty());
    assert_eq!(state.selected_fill_index, None);
    assert_eq!(state.selected_effect_index, None);
}

#[test]
fn page_menu_toggle_opens_and_closes_same_group() {
    let mut app = app_with_doc(doc_with_groups());

    let _ = super::groups::toggle_page_menu(&mut app, 2, 8.0, 9.0);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.active_page_menu, Some(2));
    assert_eq!(state.page_menu_anchor, Some(iced::Point::new(8.0, 9.0)));

    let _ = super::groups::toggle_page_menu(&mut app, 2, 0.0, 0.0);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.active_page_menu, None);
    assert_eq!(state.page_menu_anchor, None);
}

#[test]
fn rename_submit_trims_name_and_empty_submit_keeps_existing_name() {
    let mut app = app_with_doc(doc_with_groups());

    let _ = super::groups::rename_page_requested(&mut app, 2);
    assert_eq!(app.active_design_state().unwrap().renaming_page_name, "Two");

    let _ = super::groups::page_rename_changed(&mut app, "  Renamed  ".to_string());
    let _ = super::groups::submit_page_rename(&mut app);
    assert_eq!(app.active_design_state().unwrap().doc.group_name(2), Some("Renamed"));

    let state = app.active_design_state_mut().unwrap();
    state.renaming_page_id = Some(2);
    state.renaming_page_name = "   ".to_string();
    let _ = super::groups::submit_page_rename(&mut app);
    assert_eq!(app.active_design_state().unwrap().doc.group_name(2), Some("Renamed"));
}

#[test]
fn duplicate_page_inserts_copy_after_source_and_focuses_clone() {
    let mut app = app_with_doc(doc_with_groups());

    let _ = super::groups::duplicate_page(&mut app, 1);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.groups[1].id, 3);
    assert_eq!(state.doc.groups[1].name, "One 副本");
    assert_eq!(state.active_group_id, 3);
    assert!(state.doc.children.iter().any(|child| child.group_id == 3));
    assert!(state.selected_element_id.as_deref().is_some_and(|id| id != "a"));
}

#[test]
fn delete_page_removes_group_elements_or_resets_last_page() {
    let mut app = app_with_doc(doc_with_groups());
    app.active_design_state_mut().unwrap().active_group_id = 2;

    let _ = super::groups::delete_page(&mut app, 2);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.groups.len(), 1);
    assert_eq!(state.active_group_id, 1);
    assert!(state.doc.children.iter().all(|child| child.group_id != 2));

    let _ = super::groups::delete_page(&mut app, 1);
    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.groups.len(), 1);
    assert_eq!(state.doc.groups[0].name, DesignDoc::default_group_name(1));
    assert!(state.doc.children.is_empty());
}

#[test]
fn move_page_up_and_down_swap_only_when_in_bounds() {
    let mut app = app_with_doc(doc_with_groups());

    let _ = super::groups::move_page_up(&mut app, 2);
    assert_eq!(
        app.active_design_state()
            .unwrap()
            .doc
            .groups
            .iter()
            .map(|group| group.id)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );

    let _ = super::groups::move_page_down(&mut app, 2);
    assert_eq!(
        app.active_design_state()
            .unwrap()
            .doc
            .groups
            .iter()
            .map(|group| group.id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
}
