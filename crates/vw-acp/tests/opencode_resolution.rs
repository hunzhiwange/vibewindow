//! 验证内置 opencode agent 的可执行文件解析优先级。
//!
//! 当前覆盖 `OPENCODE_BIN` 环境变量路径，确保用户显式配置的二进制优先于默认
//! 命令发现逻辑，并且生成的 ACP 参数保持稳定。

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use vw_acp::built_in_agent_specs;

/// 临时修改环境变量并在 drop 时恢复，避免测试污染同进程中的后续用例。
struct EnvGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

impl EnvGuard {
    /// 设置指定环境变量并记录原值。
    fn set_os(key: &'static str, value: &std::ffi::OsStr) -> Self {
        let original = std::env::var_os(key);
        unsafe { std::env::set_var(key, value) };
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    /// 恢复环境变量到测试进入前的状态。
    fn drop(&mut self) {
        match &self.original {
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

/// 生成唯一临时目录，用于放置测试专用的 opencode 可执行文件。
fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("vw-acp-opencode-{nanos}-{}", std::process::id()))
}

/// 写入一个可执行文件；Unix 上额外设置执行位以匹配真实二进制发现条件。
fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("temp file parent should exist"))
        .expect("temp file parent should be created");
    fs::write(path, contents).expect("temp file should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // 环境变量可能指向真实可执行文件，测试文件也需要执行位才能覆盖该路径。
        let mut perms = fs::metadata(path).expect("temp file metadata should exist").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("temp file permissions should be updated");
    }
}

/// 验证 `OPENCODE_BIN` 指向的路径会成为内置 opencode agent 的命令。
#[test]
fn built_in_opencode_prefers_configured_binary_env() {
    let temp_dir = unique_temp_dir();
    let opencode_path = temp_dir.join(if cfg!(windows) { "opencode.exe" } else { "opencode" });
    write_file(&opencode_path, "#!/bin/sh\nexit 0\n");

    let _opencode_bin_guard = EnvGuard::set_os("OPENCODE_BIN", opencode_path.as_os_str());

    let specs = built_in_agent_specs();
    let spec = specs.get("opencode").expect("opencode spec should exist");

    assert_eq!(spec.command, opencode_path.to_string_lossy().to_string());
    assert_eq!(spec.args, vec!["acp".to_string()]);

    let _ = fs::remove_dir_all(temp_dir);
}
