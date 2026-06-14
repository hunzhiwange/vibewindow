use super::*;
use crate::app::App;

fn app() -> App {
    App::new().0
}

#[test]
fn normalizes_and_updates_observability_settings() {
    assert_eq!(normalize_observability_backend(" OTEL "), "otel");
    assert_eq!(normalize_observability_backend("bad"), "none");
    assert_eq!(normalize_runtime_trace_mode(" FULL "), "full");
    assert_eq!(normalize_runtime_trace_mode("bad"), "none");

    let mut app = app();
    app.observability_settings.save_error = Some("old".to_string());
    let _ = update(&mut app, SettingsMessage::ObservabilityBackendChanged(" LOG ".to_string()));
    assert_eq!(app.observability_settings.backend, "log");
    assert!(app.observability_settings.save_error.is_none());
    let _ = update(
        &mut app,
        SettingsMessage::ObservabilityOtelEndpointChanged(" http://otel ".to_string()),
    );
    let _ =
        update(&mut app, SettingsMessage::ObservabilityOtelServiceNameChanged(" svc ".to_string()));
    let _ =
        update(&mut app, SettingsMessage::ObservabilityRuntimeTraceModeChanged("bad".to_string()));
    let _ = update(
        &mut app,
        SettingsMessage::ObservabilityRuntimeTracePathChanged(" trace ".to_string()),
    );
    let _ = update(&mut app, SettingsMessage::ObservabilityRuntimeTraceMaxEntriesChanged(0));
    assert_eq!(app.observability_settings.runtime_trace_mode, "none");
    assert_eq!(app.observability_settings.runtime_trace_max_entries, 1);
    let _ = update(&mut app, SettingsMessage::ObservabilityRuntimeTraceMaxEntriesChanged(200_000));
    assert_eq!(app.observability_settings.runtime_trace_max_entries, 100_000);
    let _ = update(&mut app, SettingsMessage::ObservabilityHelpOpen);
    assert!(app.observability_settings.show_help_modal);
    let _ = update(&mut app, SettingsMessage::ObservabilityHelpClose);
    assert!(!app.observability_settings.show_help_modal);
}
