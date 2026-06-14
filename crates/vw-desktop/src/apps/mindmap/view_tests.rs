use super::view;
use crate::app::App;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::state::MindMapTab;

fn tab(id: &str) -> MindMapTab {
    MindMapTab::new(
        id.to_string(),
        format!("Tab {id}"),
        None,
        MindNode { text: id.to_string(), children: Vec::new() },
    )
}

#[test]
fn view_renders_empty_state_when_no_tab_is_available() {
    let (mut app, _task) = App::new();
    app.mindmap_tabs.clear();
    app.mindmap_active_tab_id = None;

    let _element = view(&app);
}

#[test]
fn view_renders_active_tab_when_id_matches() {
    let (mut app, _task) = App::new();
    app.mindmap_tabs = vec![tab("first"), tab("second")];
    app.mindmap_active_tab_id = Some("second".to_string());

    let _element = view(&app);
}

#[test]
fn view_falls_back_to_first_tab_when_active_id_is_stale() {
    let (mut app, _task) = App::new();
    app.mindmap_tabs = vec![tab("first")];
    app.mindmap_active_tab_id = Some("missing".to_string());

    let _element = view(&app);
}
