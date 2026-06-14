use super::detect::{
    biome_enabled, clang_format_enabled, has_dep, is_enabled, ocamlformat_enabled, pint_enabled,
    prettier_enabled, rlang_air_enabled, ruff_enabled, uvformat_enabled, which,
};
use super::state::{EnabledCheck, FormatterInfo, State};
use crate::app::agent::project::instance;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

struct PathGuard {
    old: Option<std::ffi::OsString>,
}

impl PathGuard {
    fn set(path: &Path) -> Self {
        let old = std::env::var_os("PATH");
        unsafe { std::env::set_var("PATH", path.as_os_str()) };
        Self { old }
    }
}

impl Drop for PathGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(value) => unsafe { std::env::set_var("PATH", value) },
            None => unsafe { std::env::remove_var("PATH") },
        }
    }
}

fn write_executable(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("chmod");
    }
}

async fn in_project<T: Send + 'static>(
    dir: &Path,
    f: impl FnOnce() -> crate::app::agent::project::BoxFuture<T> + Send + 'static,
) -> T {
    instance::provide(dir, None, f).await.expect("project context")
}

#[test]
fn which_returns_none_for_missing_program() {
    assert!(which("vibewindow-format-command-that-should-not-exist").is_none());
}

#[test]
fn which_finds_program_on_path_and_ignores_directories() {
    let _lock = env_lock();
    let temp = tempfile::tempdir().expect("tempdir");
    write_executable(temp.path(), "tool", "#!/bin/sh\nexit 0\n");
    std::fs::create_dir(temp.path().join("folder-tool")).expect("create dir");
    let _path = PathGuard::set(temp.path());

    assert_eq!(which("tool").as_deref(), Some(temp.path().join("tool").as_path()));
    assert!(which("folder-tool").is_none());
}

#[test]
fn has_dep_checks_dependencies_and_dev_dependencies_only() {
    assert!(has_dep(&json!({"dependencies": {"prettier": "^3"}}), "prettier"));
    assert!(has_dep(&json!({"devDependencies": {"ruff": "*"}}), "ruff"));
    assert!(!has_dep(&json!({"peerDependencies": {"prettier": "^3"}}), "prettier"));
    assert!(!has_dep(&json!({"dependencies": []}), "prettier"));
}

#[tokio::test]
async fn is_enabled_uses_cache_before_recomputing() {
    let state = Arc::new(State {
        enabled: Mutex::new(HashMap::from([("cached".to_string(), true)])),
        formatters: HashMap::new(),
    });
    let cached = FormatterInfo {
        name: "cached".to_string(),
        command: vec![],
        environment: HashMap::new(),
        extensions: vec![],
        enabled: EnabledCheck::Which("definitely-not-present"),
    };
    assert!(is_enabled(&state, &cached).await);

    let always = FormatterInfo {
        name: "always".to_string(),
        command: vec![],
        environment: HashMap::new(),
        extensions: vec![],
        enabled: EnabledCheck::Always,
    };
    assert!(is_enabled(&state, &always).await);
    assert_eq!(state.enabled.lock().unwrap().get("always"), Some(&true));
}

#[tokio::test]
async fn prettier_requires_bun_package_json_and_dependency() {
    let _lock = env_lock();
    let bin = tempfile::tempdir().expect("bin tempdir");
    write_executable(bin.path(), "bun", "#!/bin/sh\nexit 0\n");
    let _path = PathGuard::set(bin.path());

    let project = tempfile::tempdir().expect("project tempdir");
    assert!(!in_project(project.path(), || Box::pin(async { prettier_enabled().await })).await);

    std::fs::write(project.path().join("package.json"), r#"{"devDependencies":{"prettier":"3"}}"#)
        .expect("write package");
    assert!(in_project(project.path(), || Box::pin(async { prettier_enabled().await })).await);
}

#[tokio::test]
async fn prettier_is_disabled_without_bun_or_project_context() {
    let _lock = env_lock();
    let empty_bin = tempfile::tempdir().expect("bin tempdir");
    let _path = PathGuard::set(empty_bin.path());

    assert!(!prettier_enabled().await);
    let project = tempfile::tempdir().expect("project tempdir");
    std::fs::write(project.path().join("package.json"), r#"{"dependencies":{"prettier":"3"}}"#)
        .expect("write package");
    assert!(!in_project(project.path(), || Box::pin(async { prettier_enabled().await })).await);
}

#[tokio::test]
async fn biome_requires_bun_and_config_file() {
    let _lock = env_lock();
    let bin = tempfile::tempdir().expect("bin tempdir");
    write_executable(bin.path(), "bun", "#!/bin/sh\nexit 0\n");
    let _path = PathGuard::set(bin.path());
    let project = tempfile::tempdir().expect("project tempdir");

    assert!(!in_project(project.path(), || Box::pin(async { biome_enabled().await })).await);
    std::fs::write(project.path().join("biome.jsonc"), "{}").expect("write config");
    assert!(in_project(project.path(), || Box::pin(async { biome_enabled().await })).await);
}

#[tokio::test]
async fn clang_and_ocamlformat_require_binary_and_config() {
    let _lock = env_lock();
    let bin = tempfile::tempdir().expect("bin tempdir");
    write_executable(bin.path(), "clang-format", "#!/bin/sh\nexit 0\n");
    write_executable(bin.path(), "ocamlformat", "#!/bin/sh\nexit 0\n");
    let _path = PathGuard::set(bin.path());
    let project = tempfile::tempdir().expect("project tempdir");

    assert!(!in_project(project.path(), || Box::pin(async { clang_format_enabled().await })).await);
    assert!(!in_project(project.path(), || Box::pin(async { ocamlformat_enabled().await })).await);

    std::fs::write(project.path().join(".clang-format"), "BasedOnStyle: LLVM\n")
        .expect("write clang config");
    std::fs::write(project.path().join(".ocamlformat"), "profile = default\n")
        .expect("write ocaml config");

    assert!(in_project(project.path(), || Box::pin(async { clang_format_enabled().await })).await);
    assert!(in_project(project.path(), || Box::pin(async { ocamlformat_enabled().await })).await);
}

#[tokio::test]
async fn ruff_detects_config_or_dependency_mentions() {
    let _lock = env_lock();
    let bin = tempfile::tempdir().expect("bin tempdir");
    write_executable(bin.path(), "ruff", "#!/bin/sh\nexit 0\n");
    let _path = PathGuard::set(bin.path());

    let with_config = tempfile::tempdir().expect("project tempdir");
    std::fs::write(with_config.path().join("pyproject.toml"), "[tool.ruff]\n")
        .expect("write pyproject");
    assert!(in_project(with_config.path(), || Box::pin(async { ruff_enabled().await })).await);

    let with_dep = tempfile::tempdir().expect("project tempdir");
    std::fs::write(with_dep.path().join("requirements.txt"), "ruff==0.6.0\n")
        .expect("write requirements");
    assert!(in_project(with_dep.path(), || Box::pin(async { ruff_enabled().await })).await);

    let without_match = tempfile::tempdir().expect("project tempdir");
    std::fs::write(without_match.path().join("pyproject.toml"), "[project]\n")
        .expect("write pyproject");
    assert!(!in_project(without_match.path(), || Box::pin(async { ruff_enabled().await })).await);
}

#[tokio::test]
async fn uvformat_checks_uv_help_and_is_disabled_when_ruff_is_enabled() {
    let _lock = env_lock();
    let bin = tempfile::tempdir().expect("bin tempdir");
    write_executable(bin.path(), "uv", "#!/bin/sh\nexit 0\n");
    let _path = PathGuard::set(bin.path());

    let project = tempfile::tempdir().expect("project tempdir");
    assert!(in_project(project.path(), || Box::pin(async { uvformat_enabled().await })).await);

    write_executable(bin.path(), "ruff", "#!/bin/sh\nexit 0\n");
    std::fs::write(project.path().join("ruff.toml"), "").expect("write ruff config");
    assert!(!in_project(project.path(), || Box::pin(async { uvformat_enabled().await })).await);
}

#[tokio::test]
async fn pint_detects_laravel_pint_in_composer_sections() {
    let project = tempfile::tempdir().expect("project tempdir");
    assert!(!in_project(project.path(), || Box::pin(async { pint_enabled().await })).await);

    std::fs::write(
        project.path().join("composer.json"),
        r#"{"require-dev":{"laravel/pint":"^1.0"}}"#,
    )
    .expect("write composer");
    assert!(in_project(project.path(), || Box::pin(async { pint_enabled().await })).await);

    std::fs::write(project.path().join("composer.json"), r#"{"require":{"laravel/pint":"^1.0"}}"#)
        .expect("write composer");
    assert!(in_project(project.path(), || Box::pin(async { pint_enabled().await })).await);
}

#[tokio::test]
async fn rlang_air_validates_help_output_first_line() {
    let _lock = env_lock();
    let bin = tempfile::tempdir().expect("bin tempdir");
    write_executable(
        bin.path(),
        "air",
        "#!/bin/sh\nprintf 'R language formatter\\nmore help\\n'\n",
    );
    let _path = PathGuard::set(bin.path());

    assert!(rlang_air_enabled().await);

    write_executable(bin.path(), "air", "#!/bin/sh\nprintf 'unrelated tool\\n'\n");
    assert!(!rlang_air_enabled().await);
}

#[tokio::test]
async fn file_up_any_enabled_check_finds_nearest_target() {
    let state = Arc::new(State::default());
    let project = tempfile::tempdir().expect("project tempdir");
    std::fs::write(project.path().join("marker.toml"), "").expect("write marker");
    let item = FormatterInfo {
        name: "marker".to_string(),
        command: vec![],
        environment: HashMap::new(),
        extensions: vec![],
        enabled: EnabledCheck::FileUpAny(&["marker.toml"]),
    };

    assert!(
        in_project(project.path(), move || {
            let state = Arc::clone(&state);
            Box::pin(async move { is_enabled(&state, &item).await })
        })
        .await
    );
}
