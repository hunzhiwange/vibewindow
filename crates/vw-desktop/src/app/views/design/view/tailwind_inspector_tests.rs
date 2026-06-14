#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("tailwind_inspector_tests"));
}

#[test]
fn tailwind_inspector_builds_empty_and_tailwind_states() {
    let (app, _task) = crate::app::App::new();
    let mut doc = crate::app::views::design::models::DesignDoc::default();
    let mut state = crate::app::views::design::state::DesignState::new(doc.clone());

    {
        let _empty = super::render_tailwind_inspector_panel(&app, &state);
    }

    doc.children.push(crate::app::views::design::models::DesignElement {
        id: "tw".to_string(),
        kind: "tailwind".to_string(),
        content: Some("<div><span>Hello</span></div>".to_string()),
        ..Default::default()
    });
    state = crate::app::views::design::state::DesignState::new(doc);
    state.selected_element_id = Some("tw".to_string());
    {
        let _expanded = super::render_tailwind_inspector_panel(&app, &state);
    }

    state.tailwind_inspector_collapsed = true;
    {
        let _collapsed = super::render_tailwind_inspector_panel(&app, &state);
    }

    assert_eq!(super::tailwind_collapse_key("tw", &[0, 1, 2]), "tw|0.1.2");
}
