use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use super::Info;

static ENV_LOCK: Mutex<()> = Mutex::new(());
static TEST_ROOT: OnceLock<PathBuf> = OnceLock::new();

struct EnvGuard {
    _lock: MutexGuard<'static, ()>,
    saved: Vec<(&'static str, Option<OsString>)>,
}

impl EnvGuard {
    fn set_test_home(home: &Path) -> Self {
        let lock = ENV_LOCK.lock().expect("env lock should not be poisoned");
        let saved = vec![("VIBEWINDOW_TEST_HOME", std::env::var_os("VIBEWINDOW_TEST_HOME"))];
        unsafe { std::env::set_var("VIBEWINDOW_TEST_HOME", home) };
        Self { _lock: lock, saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in &self.saved {
            match value {
                Some(value) => unsafe { std::env::set_var(key, value) },
                None => unsafe { std::env::remove_var(key) },
            }
        }
    }
}

fn unique_dir(prefix: &str) -> PathBuf {
    let root = TEST_ROOT
        .get_or_init(|| {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be valid")
                .as_nanos();
            std::env::temp_dir()
                .join(format!("vw-provider-auth-tests-{}-{nanos}", std::process::id()))
        })
        .clone();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    root.join(format!("{prefix}-{nanos}"))
}

fn assert_api_key(info: Option<Info>, expected: &str) {
    match info {
        Some(Info::Api(api)) => assert_eq!(api.key, expected),
        other => panic!("expected api info, got {other:?}"),
    }
}

#[test]
fn filepath_uses_global_paths_and_legacy_auth_location() {
    let home = unique_dir("home");
    let _guard = EnvGuard::set_test_home(&home);

    let path = super::filepath();

    assert!(path.ends_with(".vibewindow/auth.json") || path.ends_with("auth.json"));
}

#[test]
fn get_and_all_read_auth_records_from_resolved_file() {
    let home = unique_dir("records-home");
    let _guard = EnvGuard::set_test_home(&home);
    let path = super::filepath();
    if !path.starts_with(&home) {
        return;
    }
    let Some(parent) = path.parent() else {
        panic!("auth path should have a parent");
    };
    std::fs::create_dir_all(parent).expect("auth parent should be created");
    std::fs::write(
        &path,
        json!({
            "openai": {"type": "api", "key": "openai-key"},
            "bad": {"type": "api"}
        })
        .to_string(),
    )
    .expect("auth file should be written");

    assert_api_key(super::get("openai"), "openai-key");
    assert!(super::get("missing").is_none());
    let all = super::all();
    assert_eq!(all.len(), 1);
    assert_api_key(all.get("openai").cloned(), "openai-key");
}
