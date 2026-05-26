//! 网关客户端测试模块，覆盖端点拼接、SSE 分帧和流式事件归一化行为。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex, MutexGuard};

    static SHELL_ENV_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn shell_env_test_lock() -> MutexGuard<'static, ()> {
        SHELL_ENV_TEST_LOCK.lock().expect("shell env test lock should acquire")
    }

    struct EnvGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set_os(key: &'static str, value: &std::ffi::OsStr) -> Self {
            let original = std::env::var_os(key);
            unsafe { std::env::set_var(key, value) };
            Self { key, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => unsafe { std::env::set_var(self.key, value) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn extract_path_assignment_rhs_supports_export_and_plain_assignments() {
        assert_eq!(
            extract_path_assignment_rhs("export PATH=/opt/homebrew/bin:$PATH"),
            Some("/opt/homebrew/bin:$PATH")
        );
        assert_eq!(
            extract_path_assignment_rhs("PATH=/usr/local/bin:$PATH"),
            Some("/usr/local/bin:$PATH")
        );
        assert_eq!(extract_path_assignment_rhs("export HOME=/tmp/demo"), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn extract_env_assignment_rhs_supports_custom_variable_name() {
        assert_eq!(
            extract_env_assignment_rhs("export CLAUDE_BIN=/opt/homebrew/bin/claude", "CLAUDE_BIN"),
            Some("/opt/homebrew/bin/claude")
        );
        assert_eq!(
            extract_env_assignment_rhs("CLAUDE_BIN=/opt/homebrew/bin/claude", "CLAUDE_BIN"),
            Some("/opt/homebrew/bin/claude")
        );
        assert_eq!(extract_env_assignment_rhs("export PATH=/usr/bin", "CLAUDE_BIN"), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn brew_shellenv_dirs_detects_homebrew_prefixes() {
        let dirs = brew_shellenv_dirs("eval \"$(/opt/homebrew/bin/brew shellenv)\"")
            .expect("should detect /opt/homebrew brew shellenv");
        assert_eq!(
            dirs,
            vec![PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/opt/homebrew/sbin")]
        );

        let dirs = brew_shellenv_dirs("eval \"$(/usr/local/bin/brew shellenv)\"")
            .expect("should detect /usr/local brew shellenv");
        assert_eq!(dirs, vec![PathBuf::from("/usr/local/bin"), PathBuf::from("/usr/local/sbin")]);
    }

    #[cfg(not(windows))]
    #[test]
    fn infer_path_from_shell_profiles_reads_path_exports() {
        let home = tempfile::TempDir::new().expect("temp home should be created");
        let custom_bin = home.path().join("custom-bin");
        let local_bin = home.path().join(".local/bin");
        std::fs::create_dir_all(&custom_bin).expect("custom bin dir should be created");
        std::fs::create_dir_all(&local_bin).expect("local bin dir should be created");
        std::fs::write(
            home.path().join(".zshrc"),
            format!("export PATH={}:$PATH\nPATH=~/.local/bin:$PATH\n", custom_bin.display()),
        )
        .expect("profile should be written");

        let inferred = infer_path_from_shell_profiles(home.path())
            .expect("path should be inferred from profile");

        assert!(inferred.contains(&custom_bin.to_string_lossy().to_string()));
        assert!(inferred.contains(&local_bin.to_string_lossy().to_string()));
    }

    #[cfg(not(windows))]
    #[test]
    fn effective_path_env_uses_profile_path_entries() {
        let _env_lock = shell_env_test_lock();
        let home = tempfile::TempDir::new().expect("temp home should be created");
        let profile_bin = home.path().join("profile-bin");
        std::fs::create_dir_all(&profile_bin).expect("profile bin dir should be created");
        std::fs::write(
            home.path().join(".zshrc"),
            format!("export PATH={}:$PATH\n", profile_bin.display()),
        )
        .expect("profile should be written");

        let _home_guard = EnvGuard::set_os("HOME", home.path().as_os_str());
        let _path_guard = EnvGuard::set_os("PATH", std::ffi::OsStr::new("/usr/bin:/bin"));

        let path = effective_path_env().expect("effective PATH should be available");
        assert!(path.contains(&profile_bin.to_string_lossy().to_string()));
    }

    #[cfg(not(windows))]
    #[test]
    fn shell_profile_env_var_reads_from_profile_when_process_env_missing() {
        let _env_lock = shell_env_test_lock();
        let home = tempfile::TempDir::new().expect("temp home should be created");
        let expected = home.path().join("tools/claude");
        std::fs::create_dir_all(expected.parent().expect("parent dir should exist"))
            .expect("parent dir should be created");
        std::fs::write(
            home.path().join(".zshrc"),
            format!("export CLAUDE_BIN={}\n", expected.display()),
        )
        .expect("profile should be written");

        let _home_guard = EnvGuard::set_os("HOME", home.path().as_os_str());
        let _claude_bin_guard = {
            let original = std::env::var_os("CLAUDE_BIN");
            unsafe { std::env::remove_var("CLAUDE_BIN") };
            EnvGuard { key: "CLAUDE_BIN", original }
        };

        let value = shell_profile_env_var("CLAUDE_BIN").expect("CLAUDE_BIN should be inferred");
        assert_eq!(value, expected.to_string_lossy().to_string());
    }

    #[test]
    fn resolve_executable_accepts_existing_absolute_path() {
        let file = tempfile::NamedTempFile::new().expect("temp file should be created");
        let resolved = resolve_executable(file.path().to_string_lossy().as_ref())
            .expect("absolute file path should resolve");
        assert_eq!(resolved, file.path());
    }

    #[test]
    fn which_in_path_value_finds_program_in_custom_path() {
        let dir = tempfile::TempDir::new().expect("temp dir should be created");
        let program_name = if cfg!(windows) { "demo-cli.exe" } else { "demo-cli" };
        let program_path = dir.path().join(program_name);
        std::fs::write(&program_path, b"#!/bin/sh\nexit 0\n").expect("program should be written");

        let found = which_in_path_value(program_name, &std::ffi::OsString::from(dir.path()))
            .expect("program should be found in custom path");
        assert_eq!(found, program_path);
    }

    #[cfg(not(windows))]
    #[test]
    fn std_command_uses_profile_augmented_path() {
        let _env_lock = shell_env_test_lock();
        let home = tempfile::TempDir::new().expect("temp home should be created");
        let bin_dir = home.path().join("profile-bin");
        std::fs::create_dir_all(&bin_dir).expect("profile bin dir should be created");
        std::fs::write(
            home.path().join(".zshrc"),
            format!("export PATH={}:$PATH\n", bin_dir.display()),
        )
        .expect("profile should be written");

        let _home_guard = EnvGuard::set_os("HOME", home.path().as_os_str());
        let _path_guard = EnvGuard::set_os("PATH", std::ffi::OsStr::new("/usr/bin:/bin"));

        let command = std_command("git");
        let path_value = command
            .get_envs()
            .find(|(key, _)| *key == std::ffi::OsStr::new("PATH"))
            .and_then(|(_, value)| value)
            .and_then(|value| value.to_str())
            .expect("PATH should be set on std_command");
        assert!(path_value.contains(&bin_dir.to_string_lossy().to_string()));
    }

    #[test]
    fn git_std_command_applies_git_default_environment() {
        let command = git_std_command();
        let debug = format!("{command:?}");
        assert!(debug.contains("GIT_PAGER"));
        assert!(debug.contains("GIT_MERGE_AUTOEDIT"));
        assert!(debug.contains("GIT_EDITOR"));
    }
}
