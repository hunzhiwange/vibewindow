#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("handlers_tests"));
}

use super::handlers::update;
use crate::app::App;
use crate::app::message::DesignMessage;
use crate::app::views::design::models::{DesignDoc, DesignElement};
use crate::app::views::design::state::{
    DesignGenerationDevice, DesignGenerationModule, DesignGenerationPage, DesignGenerationStatus,
    DesignState, DesignStyle,
};

fn text_element(id: &str, content: &str) -> DesignElement {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "type": "text",
        "content": content
    }))
    .unwrap()
}

fn app_with_design() -> App {
    let mut app = App::new().0;
    let tab_id = "design-tab".to_string();
    app.active_tab_id = Some(tab_id.clone());
    app.design_states.insert(
        tab_id,
        DesignState::new(DesignDoc {
            children: vec![text_element("title", "Old")],
            ..DesignDoc::default()
        }),
    );
    app
}

#[test]
fn update_dispatches_edit_messages_to_editing_handlers() {
    let mut app = app_with_design();

    let _ = update(&mut app, DesignMessage::EditStart("title".to_string(), "Old".to_string()));
    let _ = update(&mut app, DesignMessage::EditContentChanged("New".to_string()));
    let _ = update(&mut app, DesignMessage::EditSubmit);

    let state = app.active_design_state().unwrap();
    assert_eq!(state.doc.find_element("title").unwrap().content.as_deref(), Some("New"));
    assert!(state.editing_id.is_none());
}

#[test]
fn update_dispatches_generation_control_messages() {
    let mut app = app_with_design();

    let _ = update(&mut app, DesignMessage::ToggleDesignGenerationExecutorPopover);
    assert!(app.active_design_state().unwrap().design_generation_executor_popover);

    let _ = update(&mut app, DesignMessage::DesignGenerationModelChanged("model-a".to_string()));
    let _ = update(&mut app, DesignMessage::DesignGenerationStyleSelected(DesignStyle::Tech));
    let _ = update(
        &mut app,
        DesignMessage::DesignGenerationDeviceSelected(DesignGenerationDevice::Tablet),
    );
    let state = app.active_design_state().unwrap();
    assert_eq!(state.design_generation_model, "model-a");
    assert_eq!(state.design_generation_style, DesignStyle::Tech);
    assert_eq!(state.design_generation_device, DesignGenerationDevice::Tablet);
}

#[test]
fn update_dispatches_target_frame_selection() {
    let mut app = app_with_design();
    app.active_design_state_mut().unwrap().design_generation_pages = vec![DesignGenerationPage {
        frame_id: "page".to_string(),
        title: "Page".to_string(),
        objective: String::new(),
        status: DesignGenerationStatus::Queued,
        modules: vec![DesignGenerationModule {
            module_id: "hero".to_string(),
            title: "Hero".to_string(),
            description: String::new(),
            status: DesignGenerationStatus::Queued,
            target_frame_id: "old".to_string(),
            target_frame_options: vec!["old".to_string()],
            generated_doc: None,
            is_generating: false,
            logs: Vec::new(),
        }],
    }];

    let _ = update(
        &mut app,
        DesignMessage::SetDesignPageTargetFrame(
            "page".to_string(),
            "hero".to_string(),
            r#"{ "id": "new" }"#.to_string(),
        ),
    );

    let module = &app.active_design_state().unwrap().design_generation_pages[0].modules[0];
    assert_eq!(module.target_frame_id, "new");
}
