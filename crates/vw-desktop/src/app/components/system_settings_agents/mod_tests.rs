#[test]
fn agents_view_builds_default_loading_and_error_states() {
    let (mut app, _) = crate::app::App::new();

    let _: iced::Element<'_, crate::app::Message> = super::view(&app);

    app.agents_settings.loading = true;
    app.agents_settings.save_error = Some("unable to save".to_string());
    let _: iced::Element<'_, crate::app::Message> = super::view(&app);
}

#[test]
fn agents_view_builds_empty_entries_fallback() {
    let (mut app, _) = crate::app::App::new();
    app.agents_settings.entries.clear();
    app.agents_settings.selected_agent = "missing".to_string();

    let _: iced::Element<'_, crate::app::Message> = super::view(&app);
}

#[test]
fn agents_view_routes_each_detail_tab() {
    let (mut app, _) = crate::app::App::new();

    for tab in [
        crate::app::state::AGENT_DETAIL_BASIC_TAB,
        crate::app::state::AGENT_DETAIL_IDENTITY_TAB,
        crate::app::state::AGENT_DETAIL_TOOLS_TAB,
        crate::app::state::AGENT_DETAIL_SKILLS_TAB,
    ] {
        app.agents_settings.active_detail_tab = tab.to_string();
        let _: iced::Element<'_, crate::app::Message> = super::view(&app);
    }
}
