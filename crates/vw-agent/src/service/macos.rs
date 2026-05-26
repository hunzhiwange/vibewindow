use crate::app::agent::config::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::common::{build_launchd_env_vars, run_capture, run_checked, xml_escape};

/// macOS launchd 服务的标识符标签。
pub(super) const SERVICE_LABEL: &str = "com.vibewindow.daemon";

pub(super) fn install_macos(config: &Config) -> Result<()> {
    let file = macos_service_file()?;
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }

    let exe = std::env::current_exe().context("Failed to resolve current executable")?;
    let logs_dir =
        config.config_path.parent().map_or_else(|| PathBuf::from("."), PathBuf::from).join("logs");
    fs::create_dir_all(&logs_dir)?;

    let stdout = logs_dir.join("daemon.stdout.log");
    let stderr = logs_dir.join("daemon.stderr.log");
    let env_block = build_launchd_env_vars();

    let plist = format!(
        r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>daemon</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>{env_block}
  <key>StandardOutPath</key>
  <string>{stdout}</string>
  <key>StandardErrorPath</key>
  <string>{stderr}</string>
</dict>
</plist>
"#,
        label = SERVICE_LABEL,
        exe = xml_escape(&exe.display().to_string()),
        env_block = env_block,
        stdout = xml_escape(&stdout.display().to_string()),
        stderr = xml_escape(&stderr.display().to_string())
    );

    fs::write(&file, plist)?;
    println!("✅ Installed launchd service: {}", file.display());
    println!("   Start with: vibewindow service start");
    Ok(())
}

pub(super) fn start_macos() -> Result<()> {
    let plist = macos_service_file()?;
    run_checked(Command::new("launchctl").arg("load").arg("-w").arg(&plist))?;
    run_checked(Command::new("launchctl").arg("start").arg(SERVICE_LABEL))?;
    println!("✅ Service started");
    Ok(())
}

pub(super) fn stop_macos() -> Result<()> {
    let plist = macos_service_file()?;
    let _ = run_checked(Command::new("launchctl").arg("stop").arg(SERVICE_LABEL));
    let _ = run_checked(Command::new("launchctl").arg("unload").arg("-w").arg(&plist));
    println!("✅ Service stopped");
    Ok(())
}

pub(super) fn status_macos() -> Result<()> {
    let out = run_capture(Command::new("launchctl").arg("list"))?;
    let running = out.lines().any(|line| line.contains(SERVICE_LABEL));
    println!("Service: {}", if running { "✅ running/loaded" } else { "❌ not loaded" });
    println!("Unit: {}", macos_service_file()?.display());
    Ok(())
}

pub(super) fn uninstall_macos() -> Result<()> {
    let file = macos_service_file()?;
    if file.exists() {
        fs::remove_file(&file).with_context(|| format!("Failed to remove {}", file.display()))?;
    }
    println!("✅ Service uninstalled ({})", file.display());
    Ok(())
}

pub(super) fn macos_service_file() -> Result<PathBuf> {
    let home = directories::UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .context("Could not find home directory")?;
    Ok(home.join("Library").join("LaunchAgents").join(format!("{SERVICE_LABEL}.plist")))
}
