use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn clamps_limits_and_toggles_remote_fetch() {
    let mut app = app();
    app.multimodal_settings.save_error = Some("old".to_string());

    let _ = update(&mut app, SettingsMessage::Multimodal(MultimodalMessage::MaxImagesChanged(0)));
    assert_eq!(app.multimodal_settings.max_images, 1);
    assert!(app.multimodal_settings.save_error.is_none());
    let _ = update(&mut app, SettingsMessage::Multimodal(MultimodalMessage::MaxImagesChanged(99)));
    assert_eq!(app.multimodal_settings.max_images, 16);

    let _ =
        update(&mut app, SettingsMessage::Multimodal(MultimodalMessage::MaxImageSizeMbChanged(0)));
    assert_eq!(app.multimodal_settings.max_image_size_mb, 1);
    let _ =
        update(&mut app, SettingsMessage::Multimodal(MultimodalMessage::MaxImageSizeMbChanged(99)));
    assert_eq!(app.multimodal_settings.max_image_size_mb, 20);

    let _ = update(
        &mut app,
        SettingsMessage::Multimodal(MultimodalMessage::AllowRemoteFetchToggled(true)),
    );
    assert!(app.multimodal_settings.allow_remote_fetch);
    let _ = update(&mut app, SettingsMessage::SchedulerHelpOpen);
    assert!(app.multimodal_settings.allow_remote_fetch);
}
