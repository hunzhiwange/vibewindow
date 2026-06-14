//! 桌面启动阶段的 CLI 与本机服务引导逻辑。
//!
//! 这里不依赖 gateway，因为它的职责正是确保 gateway 所在的本机服务可用。

#[cfg(not(target_arch = "wasm32"))]
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_GATEWAY_PORT: u16 = 42617;
#[cfg(not(target_arch = "wasm32"))]
const DESKTOP_GATEWAY_PORT_BASE: u16 = 14600;
#[cfg(not(target_arch = "wasm32"))]
const DESKTOP_GATEWAY_PORT_RANGE: u16 = 100;
#[cfg(not(target_arch = "wasm32"))]
const MACOS_SERVICE_LABEL: &str = "com.vibewindow.daemon";
#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
const WINDOWS_TASK_NAME: &str = "VibeWindow Daemon";

#[cfg(not(target_arch = "wasm32"))]
pub async fn bootstrap_cli_service_async() -> Result<(), String> {
    tokio::task::spawn_blocking(bootstrap_cli_service)
        .await
        .map_err(|err| format!("CLI 服务引导任务失败: {err}"))?
}

#[cfg(not(target_arch = "wasm32"))]
fn bootstrap_cli_service() -> Result<(), String> {
    let home = resolve_home_dir()?;
    let config_dir = vw_config_types::paths::home_config_dir(&home);
    let cli = ensure_cli_installed(&home)?;

    if !service_installed(&home) {
        if let Some(port) = ensure_first_install_gateway_port(&home)? {
            sync_gateway_client_bootstrap_port(port);
        }
        run_cli_command(&cli, &config_dir, &["service", "install"])?;
    }

    if !service_running(&home) {
        if let Err(server_start_error) = run_cli_command(&cli, &config_dir, &["server", "start"]) {
            tracing::debug!(
                target: "vw_desktop",
                error = %server_start_error,
                "vibewindow server start failed; falling back to service start"
            );
            run_cli_command(&cli, &config_dir, &["service", "start"])?;
        }
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_cli_installed(home: &Path) -> Result<PathBuf, String> {
    let install_dir = vw_config_types::paths::home_config_dir(home).join("bin");
    let target_bin = install_dir.join(cli_binary_name());
    if verify_cli_binary(&target_bin).is_ok() {
        return Ok(target_bin);
    }

    fs::create_dir_all(&install_dir).map_err(|err| format!("创建 CLI 安装目录失败: {err}"))?;

    let install_result = match resolve_agent_binary_path() {
        Ok(source_bin) => {
            fs::copy(&source_bin, &target_bin).map_err(|err| {
                format!(
                    "复制 CLI 可执行文件失败: {} -> {}: {err}",
                    source_bin.display(),
                    target_bin.display()
                )
            })?;
            Ok(())
        }
        Err(source_error) => match std::env::var("VIBEWINDOW_CLI_URL") {
            Ok(url) => {
                let tmp_path = install_dir.join(if cfg!(windows) {
                    "vibewindow.tmp.exe"
                } else {
                    "vibewindow.tmp"
                });
                download_file(&url, &tmp_path)?;
                fs::rename(&tmp_path, &target_bin)
                    .map_err(|err| format!("移动 CLI 临时文件失败: {err}"))
            }
            Err(_) => Err(format!(
                "未找到可打包的 CLI 可执行文件。{source_error}。\
                     可设置 VIBEWINDOW_CLI_URL 后重启桌面端。"
            )),
        },
    };

    if install_result.is_ok() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target_bin, fs::Permissions::from_mode(0o755))
                .map_err(|err| format!("设置 CLI 执行权限失败: {err}"))?;
        }

        let _ = ensure_path_profiles(home, &install_dir).map_err(|err| {
            tracing::warn!(target: "vw_desktop", error = %err, "failed to persist CLI PATH profile");
            err
        });
        verify_cli_binary(&target_bin)?;
        return Ok(target_bin);
    }

    let path_cli = PathBuf::from(cli_binary_name());
    if verify_cli_binary(&path_cli).is_ok() {
        return Ok(path_cli);
    }

    Err(install_result.expect_err("install_result is an error"))
}

#[cfg(not(target_arch = "wasm32"))]
fn verify_cli_binary(path: &Path) -> Result<(), String> {
    let output = Command::new(path)
        .arg("--version")
        .output()
        .map_err(|err| format!("执行 {} --version 失败: {err}", path.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_failure_message(path, &["--version"], output))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_cli_command(cli: &Path, config_dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cli)
        .arg("--config-dir")
        .arg(config_dir)
        .args(args)
        .output()
        .map_err(|err| format!("执行 {} 失败: {err}", format_cli_command(cli, args)))?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(stdout);
    }
    Err(command_failure_message(cli, args, output))
}

#[cfg(not(target_arch = "wasm32"))]
fn command_failure_message(cli: &Path, args: &[&str], output: std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    if detail.is_empty() {
        format!("{} 退出状态异常: {}", format_cli_command(cli, args), output.status)
    } else {
        format!("{} 失败: {detail}", format_cli_command(cli, args))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn format_cli_command(cli: &Path, args: &[&str]) -> String {
    let mut command = cli.display().to_string();
    for arg in args {
        command.push(' ');
        command.push_str(arg);
    }
    command
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_first_install_gateway_port(home: &Path) -> Result<Option<u16>, String> {
    let config_dir = vw_config_types::paths::home_config_dir(home);
    let config_path = config_dir.join("vibewindow.json");
    let existing_root = read_config_root(&config_path)?;
    let mut root = if existing_root.as_object().is_some_and(|obj| obj.is_empty()) {
        default_config_root(home)?
    } else {
        merge_json_defaults(default_config_root(home)?, existing_root)
    };

    let current_port = root
        .get("gateway")
        .and_then(|value| value.get("port"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u16::try_from(value).ok());

    if current_port.is_some_and(|port| port != DEFAULT_GATEWAY_PORT) {
        return Ok(None);
    }

    let port = random_desktop_gateway_port();
    set_gateway_port(&mut root, port);
    write_json_atomically(&config_path, &root)?;
    Ok(Some(port))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_config_root(path: &Path) -> Result<serde_json::Value, String> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(path)
        .map_err(|err| format!("读取配置文件失败 {}: {err}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&content)
        .map_err(|err| format!("解析配置文件失败 {}: {err}", path.display()))
}

#[cfg(not(target_arch = "wasm32"))]
fn default_config_root(home: &Path) -> Result<serde_json::Value, String> {
    let mut config = vw_config_types::config::Config::default();
    let config_dir = vw_config_types::paths::home_config_dir(home);
    config.config_path = config_dir.join("vibewindow.json");
    config.workspace_dir = config_dir.join("workspace");
    serde_json::to_value(config).map_err(|err| format!("序列化默认配置失败: {err}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn merge_json_defaults(
    default: serde_json::Value,
    existing: serde_json::Value,
) -> serde_json::Value {
    match (default, existing) {
        (serde_json::Value::Object(mut defaults), serde_json::Value::Object(existing)) => {
            for (key, value) in existing {
                let merged = defaults
                    .remove(&key)
                    .map(|default_value| merge_json_defaults(default_value, value.clone()))
                    .unwrap_or(value);
                defaults.insert(key, merged);
            }
            serde_json::Value::Object(defaults)
        }
        (_, existing) => existing,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_gateway_port(root: &mut serde_json::Value, port: u16) {
    if !root.is_object() {
        *root = serde_json::json!({});
    }
    let obj = root.as_object_mut().expect("root object exists");
    let gateway = obj.entry("gateway").or_insert_with(|| serde_json::json!({}));
    if !gateway.is_object() {
        *gateway = serde_json::json!({});
    }
    gateway
        .as_object_mut()
        .expect("gateway object exists")
        .insert("port".to_string(), serde_json::Value::Number(port.into()));
}

#[cfg(not(target_arch = "wasm32"))]
fn write_json_atomically(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("配置路径缺少父目录: {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|err| format!("创建配置目录失败 {}: {err}", parent.display()))?;

    let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or("vibewindow.json");
    let tmp_path = parent.join(format!(
        ".{file_name}.tmp-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    ));
    let content =
        serde_json::to_string_pretty(value).map_err(|err| format!("序列化配置文件失败: {err}"))?;
    fs::write(&tmp_path, content)
        .map_err(|err| format!("写入临时配置文件失败 {}: {err}", tmp_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600))
            .map_err(|err| format!("设置配置文件权限失败 {}: {err}", tmp_path.display()))?;
    }

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)
            .map_err(|err| format!("移除旧配置文件失败 {}: {err}", path.display()))?;
    }

    fs::rename(&tmp_path, path).map_err(|err| {
        let _ = fs::remove_file(&tmp_path);
        format!("替换配置文件失败 {}: {err}", path.display())
    })?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn random_desktop_gateway_port() -> u16 {
    let start = rand::thread_rng().gen_range(0..DESKTOP_GATEWAY_PORT_RANGE);
    for offset in 0..DESKTOP_GATEWAY_PORT_RANGE {
        let port = DESKTOP_GATEWAY_PORT_BASE + ((start + offset) % DESKTOP_GATEWAY_PORT_RANGE);
        if port_available(port) {
            return port;
        }
    }
    DESKTOP_GATEWAY_PORT_BASE + start
}

#[cfg(not(target_arch = "wasm32"))]
fn port_available(port: u16) -> bool {
    std::net::TcpListener::bind(("127.0.0.1", port)).is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_gateway_client_bootstrap_port(port: u16) {
    let mut cfg = super::system_settings::load_gateway_client_bootstrap_config();
    let mut active = cfg.active_server();
    if active.port != DEFAULT_GATEWAY_PORT && !cfg.servers.is_empty() {
        return;
    }

    active.host = if active.host.trim().is_empty() { "127.0.0.1".to_string() } else { active.host };
    active.port = port;

    let mut servers = cfg.normalized_servers();
    let mut replaced = false;
    for server in &mut servers {
        if server.id == active.id {
            *server = active.clone();
            replaced = true;
        } else if server.id == "local" && server.port == DEFAULT_GATEWAY_PORT {
            server.port = port;
        }
    }
    if !replaced {
        servers.insert(0, active.clone());
    }
    cfg.set_servers(servers, active.id);
    super::system_settings::save_gateway_client_bootstrap_config(&cfg);
}

#[cfg(not(target_arch = "wasm32"))]
fn service_installed(home: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        return home
            .join("Library")
            .join("LaunchAgents")
            .join(format!("{MACOS_SERVICE_LABEL}.plist"))
            .exists();
    }

    #[cfg(target_os = "linux")]
    {
        return home
            .join(".config")
            .join("systemd")
            .join("user")
            .join("vibewindow.service")
            .exists()
            || Path::new("/etc/init.d/vibewindow").exists();
    }

    #[cfg(target_os = "windows")]
    {
        return Command::new("schtasks")
            .args(["/Query", "/TN", WINDOWS_TASK_NAME])
            .output()
            .is_ok_and(|output| output.status.success());
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = home;
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn service_running(home: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        let _ = home;
        return Command::new("launchctl")
            .arg("list")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .is_some_and(|text| text.lines().any(|line| line.contains(MACOS_SERVICE_LABEL)));
    }

    #[cfg(target_os = "linux")]
    {
        let systemd_active = Command::new("systemctl")
            .args(["--user", "is-active", "--quiet", "vibewindow.service"])
            .status()
            .is_ok_and(|status| status.success());
        if systemd_active {
            return true;
        }
        if Path::new("/etc/init.d/vibewindow").exists() {
            return Command::new("rc-service")
                .args(["vibewindow", "status"])
                .status()
                .is_ok_and(|status| status.success());
        }
        let _ = home;
        false
    }

    #[cfg(target_os = "windows")]
    {
        let _ = home;
        return Command::new("schtasks")
            .args(["/Query", "/TN", WINDOWS_TASK_NAME, "/FO", "LIST"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .is_some_and(|text| text.contains("Running"));
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = home;
        false
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_home_dir() -> Result<PathBuf, String> {
    if let Some(user_dirs) = directories::UserDirs::new() {
        return Ok(user_dirs.home_dir().to_path_buf());
    }
    if let Ok(home) = std::env::var("HOME")
        && !home.trim().is_empty()
    {
        return Ok(PathBuf::from(home));
    }
    if let Ok(home) = std::env::var("USERPROFILE")
        && !home.trim().is_empty()
    {
        return Ok(PathBuf::from(home));
    }
    Err("无法定位用户主目录".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn cli_binary_name() -> &'static str {
    if cfg!(windows) { "vibewindow.exe" } else { "vibewindow" }
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_agent_binary_path() -> Result<PathBuf, String> {
    let current_exe =
        std::env::current_exe().map_err(|err| format!("获取当前可执行文件失败: {err}"))?;
    let exe_dir = current_exe.parent().ok_or_else(|| "无法定位当前可执行文件目录".to_string())?;
    let agent_names = if cfg!(windows) {
        ["vibewindow.exe", "vibe-agent.exe"]
    } else {
        ["vibewindow", "vibe-agent"]
    };
    let mut candidates = agent_names.iter().map(|name| exe_dir.join(name)).collect::<Vec<_>>();

    if let Some(parent) = exe_dir.parent() {
        for agent_name in agent_names {
            candidates.push(parent.join(agent_name));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        for agent_name in agent_names {
            candidates.push(cwd.join("target").join("release").join(agent_name));
            candidates.push(cwd.join("target").join("debug").join(agent_name));
        }
    }

    for candidate in candidates {
        if candidate.exists() && candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err("未找到内置 CLI 可执行文件 vibewindow（兼容名 vibe-agent）".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn download_file(url: &str, dest: &Path) -> Result<(), String> {
    if cfg!(windows) {
        let status = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", url, dest.display()))
            .status()
            .map_err(|err| format!("启动下载命令失败: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err("下载 CLI 失败，请检查 VIBEWINDOW_CLI_URL 或 PowerShell。".to_string());
    }

    let curl = Command::new("curl").arg("-fsSL").arg(url).arg("-o").arg(dest).status();
    if curl.is_ok_and(|status| status.success()) {
        return Ok(());
    }

    let wget = Command::new("wget").arg("-qO").arg(dest).arg(url).status();
    if wget.is_ok_and(|status| status.success()) {
        return Ok(());
    }
    Err("下载 CLI 失败，请检查 VIBEWINDOW_CLI_URL，或安装 curl/wget。".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_path_profiles(home: &Path, install_dir: &Path) -> Result<Vec<String>, String> {
    #[cfg(windows)]
    let profiles = vec![
        home.join("Documents").join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1"),
        home.join("Documents").join("PowerShell").join("Microsoft.PowerShell_profile.ps1"),
    ];
    #[cfg(not(windows))]
    let profiles = vec![home.join(".zshrc"), home.join(".bashrc"), home.join(".profile")];

    let mut updated = Vec::new();
    for profile in profiles {
        if ensure_profile_contains_path(&profile, install_dir)? {
            updated.push(profile.display().to_string());
        }
    }
    Ok(updated)
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_profile_contains_path(profile: &Path, install_dir: &Path) -> Result<bool, String> {
    #[cfg(windows)]
    let export_line = format!(r#"$env:Path = "{};$env:Path""#, install_dir.display());
    #[cfg(not(windows))]
    let export_line = format!("export PATH={}:$PATH", install_dir.display());

    if let Some(parent) = profile.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("创建配置文件目录失败 {}: {err}", parent.display()))?;
    }

    let install_path_text = install_dir.to_string_lossy();
    let existing = fs::read_to_string(profile).unwrap_or_default();
    if existing.contains(export_line.as_str()) || existing.contains(install_path_text.as_ref()) {
        return Ok(false);
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(profile)
        .map_err(|err| format!("打开配置文件失败 {}: {err}", profile.display()))?;

    if !existing.ends_with('\n') && !existing.is_empty() {
        file.write_all(b"\n")
            .map_err(|err| format!("写入配置文件失败 {}: {err}", profile.display()))?;
    }
    file.write_all(export_line.as_bytes())
        .map_err(|err| format!("写入配置文件失败 {}: {err}", profile.display()))?;
    file.write_all(b"\n")
        .map_err(|err| format!("写入配置文件失败 {}: {err}", profile.display()))?;
    Ok(true)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn merge_json_defaults_preserves_existing_values_and_fills_missing() {
        let default = serde_json::json!({
            "gateway": { "host": "127.0.0.1", "port": 42617 },
            "default_temperature": 0.7
        });
        let existing = serde_json::json!({
            "gateway": { "host": "0.0.0.0" },
            "custom": true
        });

        let merged = merge_json_defaults(default, existing);

        assert_eq!(merged["gateway"]["host"], "0.0.0.0");
        assert_eq!(merged["gateway"]["port"], 42617);
        assert_eq!(merged["default_temperature"], 0.7);
        assert_eq!(merged["custom"], true);
    }

    #[test]
    fn set_gateway_port_creates_gateway_object() {
        let mut root = serde_json::json!({});
        set_gateway_port(&mut root, 14642);

        assert_eq!(root["gateway"]["port"], 14642);
    }
}
