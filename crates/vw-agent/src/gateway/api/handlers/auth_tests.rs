use super::*;
use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use std::path::{Path as FsPath, PathBuf};
use std::sync::OnceLock;

async fn env_lock() -> tokio::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(())).lock().await
}

fn test_home() -> &'static PathBuf {
    static HOME: OnceLock<PathBuf> = OnceLock::new();
    HOME.get_or_init(|| {
        let home = std::env::temp_dir()
            .join(format!("vw-agent-auth-handler-tests-{}", std::process::id()));
        let auth_file = vw_config_types::paths::home_config_dir(&home).join("auth.json");
        std::fs::create_dir_all(auth_file.parent().expect("auth.json should have parent"))
            .expect("should create auth test directory");
        std::fs::write(&auth_file, "{}").expect("should seed auth file");
        unsafe {
            std::env::set_var("VIBEWINDOW_TEST_HOME", &home);
        }
        home
    })
}

fn auth_file() -> PathBuf {
    let paths = crate::app::agent::global::paths();
    vw_shared::auth::store::resolve_filepath(&paths.home, &paths.data)
}

fn reset_auth_store() {
    let auth_file = auth_file();
    if auth_file.is_dir() {
        std::fs::remove_dir_all(&auth_file).expect("should remove auth directory");
    }
    std::fs::create_dir_all(auth_file.parent().expect("auth.json should have parent"))
        .expect("should create auth parent directory");
    #[cfg(unix)]
    set_mode(&auth_file, 0o600);
    std::fs::write(auth_file, "{}").expect("should reset auth file");
}

fn api_info(key: &str) -> auth::Info {
    auth::Info::Api(auth::ApiInfo { key: key.to_string() })
}

#[cfg(unix)]
fn set_mode(path: &FsPath, mode: u32) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = std::fs::metadata(path) {
        let mut permissions = metadata.permissions();
        permissions.set_mode(mode);
        std::fs::set_permissions(path, permissions).expect("should update file permissions");
    }
}

#[cfg(unix)]
fn replace_auth_file_with_directory() {
    let auth_file = auth_file();
    let _ = std::fs::remove_file(&auth_file);
    std::fs::create_dir_all(&auth_file).expect("should replace auth file with directory");
}

#[cfg(unix)]
fn restore_auth_file_from_directory() {
    let auth_file = auth_file();
    let _ = std::fs::remove_dir_all(&auth_file);
    reset_auth_store();
}

#[tokio::test]
async fn auth_set_persists_credentials_and_returns_true() {
    let _guard = env_lock().await;
    reset_auth_store();

    let response = auth_set(Path("provider.openai".to_string()), Json(api_info("sk-test-abc123")))
        .await
        .expect("setting auth should succeed");

    assert!(response.0);
    match auth::get("provider.openai") {
        Some(auth::Info::Api(api)) => assert_eq!(api.key, "sk-test-abc123"),
        _ => panic!("expected stored api credentials"),
    }
}

#[tokio::test]
async fn auth_set_overwrites_existing_provider_credentials() {
    let _guard = env_lock().await;
    reset_auth_store();
    auth::set("provider.openai", &api_info("sk-old")).expect("should seed existing auth");

    let response = auth_set(Path("provider.openai".to_string()), Json(api_info("sk-new")))
        .await
        .expect("updating auth should succeed");

    assert!(response.0);
    match auth::get("provider.openai") {
        Some(auth::Info::Api(api)) => assert_eq!(api.key, "sk-new"),
        _ => panic!("expected updated api credentials"),
    }
}

#[tokio::test]
async fn auth_remove_deletes_existing_provider_and_returns_true() {
    let _guard = env_lock().await;
    reset_auth_store();
    auth::set("provider.openai", &api_info("sk-remove")).expect("should seed auth");
    auth::set(
        "provider.github",
        &auth::Info::Wellknown(auth::WellKnownInfo {
            key: "wk-key".to_string(),
            token: "wk-token".to_string(),
        }),
    )
    .expect("should seed second provider auth");

    let response = auth_remove(Path("provider.openai".to_string()))
        .await
        .expect("removing auth should succeed");

    assert!(response.0);
    assert!(auth::get("provider.openai").is_none());
    assert!(auth::get("provider.github").is_some());
}

#[tokio::test]
async fn auth_remove_missing_provider_keeps_existing_credentials() {
    let _guard = env_lock().await;
    reset_auth_store();
    auth::set("provider.github", &api_info("sk-still-there")).expect("should seed auth");

    let response = auth_remove(Path("provider.missing".to_string()))
        .await
        .expect("removing missing auth should still succeed");

    assert!(response.0);
    match auth::get("provider.github") {
        Some(auth::Info::Api(api)) => assert_eq!(api.key, "sk-still-there"),
        _ => panic!("expected untouched provider auth"),
    }
}

#[cfg(unix)]
#[tokio::test]
async fn auth_set_maps_write_errors_to_bad_request() {
    let _guard = env_lock().await;
    reset_auth_store();
    replace_auth_file_with_directory();

    let err = auth_set(Path("provider.openai".to_string()), Json(api_info("sk-denied")))
        .await
        .expect_err("setting auth should fail when auth path is a directory");

    assert_eq!(err.status, StatusCode::BAD_REQUEST);
    assert!(
        err.message.to_ascii_lowercase().contains("directory"),
        "unexpected error message: {}",
        err.message
    );

    restore_auth_file_from_directory();
}

#[cfg(unix)]
#[tokio::test]
async fn auth_remove_maps_write_errors_to_bad_request() {
    let _guard = env_lock().await;
    reset_auth_store();
    auth::set("provider.openai", &api_info("sk-denied")).expect("should seed auth");
    replace_auth_file_with_directory();

    let err = auth_remove(Path("provider.openai".to_string()))
        .await
        .expect_err("removing auth should fail when auth path is a directory");

    assert_eq!(err.status, StatusCode::BAD_REQUEST);
    assert!(
        err.message.to_ascii_lowercase().contains("directory"),
        "unexpected error message: {}",
        err.message
    );

    restore_auth_file_from_directory();
}

#[test]
fn router_builds_without_panic() {
    let _r: Router<()> = router();
}
