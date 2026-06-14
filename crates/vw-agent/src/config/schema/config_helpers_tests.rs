use super::config_helpers::config_dir_creation_error;
use super::config_helpers::read_codex_openai_api_key;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn config_dir_creation_error_mentions_path_and_openrc_hint() {
    let message = config_dir_creation_error(std::path::Path::new("/restricted/vw"));

    assert!(message.contains("/restricted/vw"));
    assert!(message.contains("OpenRC"));
}

#[test]
fn read_codex_openai_api_key_returns_none_when_auth_file_is_absent() {
    let _guard = env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_home = std::env::var("HOME").ok();
    unsafe {
        std::env::set_var("HOME", tmp.path());
    }

    assert!(read_codex_openai_api_key().is_none());

    unsafe {
        if let Some(old_home) = old_home {
            std::env::set_var("HOME", old_home);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
