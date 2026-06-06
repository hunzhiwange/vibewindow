//! Gateway-side cleanup execution for the desktop cleaner.

use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;

use super::fs::{covers_target, expand_env_path, measure_cleanup_target};
use super::scan::{ScanDetailKind, unsupported_platform_message};
use vw_api_types::cleaner::CleanerCleanupRequest;

pub(super) fn execute_cleanup(
    request: CleanerCleanupRequest,
    cancel_flag: Arc<AtomicBool>,
    progress_output: Arc<Mutex<String>>,
) -> Result<String, String> {
    if cfg!(target_os = "macos") {
        run_macos_cleanup(&request, &cancel_flag, &progress_output)
    } else if cfg!(target_os = "windows") {
        run_windows_cleanup(&request, &cancel_flag, &progress_output)
    } else {
        Err(unsupported_platform_message())
    }
}

#[derive(Clone, Debug, Default)]
struct CleanupStats {
    targets: Vec<CleanupTarget>,
}

impl CleanupStats {
    fn track_directory(&mut self, raw_path: &'static str) {
        self.track(raw_path, ScanDetailKind::Directory);
    }

    fn track_matching_files(
        &mut self,
        raw_path: &'static str,
        extensions: &'static [&'static str],
    ) {
        self.track(raw_path, ScanDetailKind::FileExtensions(extensions));
    }

    fn summary_line(&self) -> String {
        format!("本次预计清理垃圾数据：{}", format_bytes(self.expected_removed()))
    }

    fn actual_removed_line(&self) -> String {
        format!("本次实际删除垃圾数据：{}", format_bytes(self.actual_removed()))
    }

    fn expected_removed(&self) -> u64 {
        self.targets.iter().map(|target| target.before_bytes).sum()
    }

    fn actual_removed(&self) -> u64 {
        self.targets
            .iter()
            .map(|target| {
                let after_bytes = measure_cleanup_target(&target.path, target.kind);
                target.before_bytes.saturating_sub(after_bytes)
            })
            .sum()
    }

    fn track(&mut self, raw_path: &'static str, kind: ScanDetailKind) {
        let path = std::path::PathBuf::from(expand_env_path(raw_path));
        if self.targets.iter().any(|target| target.covers(&path, kind)) {
            return;
        }
        self.targets.retain(|target| !covers_target(&path, kind, &target.path, target.kind));
        let before_bytes = measure_cleanup_target(&path, kind);
        self.targets.push(CleanupTarget { path, kind, before_bytes });
    }
}

#[derive(Clone, Debug)]
struct CleanupTarget {
    path: std::path::PathBuf,
    kind: ScanDetailKind,
    before_bytes: u64,
}

impl CleanupTarget {
    fn covers(&self, other_path: &std::path::Path, other_kind: ScanDetailKind) -> bool {
        covers_target(&self.path, self.kind, other_path, other_kind)
    }
}

fn run_macos_cleanup(
    request: &CleanerCleanupRequest,
    cancel_flag: &Arc<AtomicBool>,
    progress_output: &Arc<Mutex<String>>,
) -> Result<String, String> {
    let mut log = vec!["开始执行 macOS 垃圾清理...".to_string()];
    let mut user_lines = Vec::new();
    let mut admin_lines = Vec::new();
    let mut stats = CleanupStats::default();
    publish_log(&log, progress_output);

    if cancelled(cancel_flag, &mut log, progress_output) {
        return Ok(log.join("\n\n"));
    }

    if request.clear_system_temp {
        stats.track_directory("$TMPDIR");
        stats.track_directory("/private/var/folders");
        user_lines.push("rm -rf \"$TMPDIR\"* 2>/dev/null || true".to_string());
        admin_lines.push("rm -rf /private/var/folders/*/*/*/T/* 2>/dev/null || true".to_string());
    }
    if request.clear_app_cache {
        stats.track_directory("$HOME/Library/Caches");
        stats.track_directory("$HOME/.cache");
        user_lines.push("rm -rf \"$HOME/Library/Caches\"/* 2>/dev/null || true".to_string());
        user_lines.push("rm -rf \"$HOME/.cache\"/* 2>/dev/null || true".to_string());
    }
    if request.clear_downloads {
        stats.track_directory("$HOME/Downloads");
        user_lines.push("if [ -d \"$HOME/Downloads\" ]; then find \"$HOME/Downloads\" -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null; fi".to_string());
    }
    if request.clear_wechat_work {
        stats.track_directory("$HOME/Library/Containers/com.tencent.WeWorkMac/Data/Library/Caches");
        stats.track_directory(
            "$HOME/Library/Group Containers/2N38VWS5BX.com.tencent.WeWorkMac/Library/Caches",
        );
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.tencent.WeWorkMac\"/Data/Library/Caches/* 2>/dev/null || true".to_string());
        user_lines.push("rm -rf \"$HOME/Library/Group Containers/2N38VWS5BX.com.tencent.WeWorkMac\"/Library/Caches/* 2>/dev/null || true".to_string());
    }
    if request.clear_wechat {
        stats.track_directory("$HOME/Library/Containers/com.tencent.xinWeChat/Data/Library/Caches");
        stats.track_directory("$HOME/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/xwechat_files/xwechat/cache");
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.tencent.xinWeChat\"/Data/Library/Caches/* 2>/dev/null || true".to_string());
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.tencent.xinWeChat\"/Data/Library/Application Support/com.tencent.xinWeChat/xwechat_files/xwechat/cache/* 2>/dev/null || true".to_string());
    }
    if request.clear_qq {
        stats.track_directory("$HOME/Library/Containers/com.tencent.qq/Data/Library/Caches");
        stats.track_directory("$HOME/Library/Containers/com.tencent.qq/Data/Library/Application Support/QQ/global/nt_qq/Cache");
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.tencent.qq\"/Data/Library/Caches/* 2>/dev/null || true".to_string());
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.tencent.qq\"/Data/Library/Application Support/QQ/global/nt_qq/Cache/* 2>/dev/null || true".to_string());
    }
    if request.clear_dingtalk {
        stats.track_directory(
            "$HOME/Library/Containers/com.alibaba.DingTalkMac/Data/Library/Caches",
        );
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.alibaba.DingTalkMac\"/Data/Library/Caches/* 2>/dev/null || true".to_string());
    }
    if request.clear_feishu {
        stats.track_directory("$HOME/Library/Containers/com.bytedance.feishu/Data/Library/Caches");
        stats.track_directory("$HOME/Library/Containers/com.bytedance.feishu/Data/Library/Application Support/LarkShell");
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.bytedance.feishu\"/Data/Library/Caches/* 2>/dev/null || true".to_string());
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.bytedance.feishu\"/Data/Library/Application Support/LarkShell/* 2>/dev/null || true".to_string());
    }
    if request.clear_installers {
        stats.track_matching_files("$HOME/Downloads", &["dmg", "pkg", "xip", "zip", "iso"]);
        stats.track_matching_files("$HOME/Desktop", &["dmg", "pkg", "xip", "zip", "iso"]);
        user_lines.push("find \"$HOME/Downloads\" \"$HOME/Desktop\" -type f \\( -iname '*.dmg' -o -iname '*.pkg' -o -iname '*.xip' -o -iname '*.zip' -o -iname '*.iso' \\) -delete 2>/dev/null || true".to_string());
    }
    if request.clear_other_apps {
        for path in [
            "$HOME/Library/Application Support/Cursor/Cache",
            "$HOME/Library/Application Support/Code/Cache",
            "$HOME/Library/Application Support/Slack/Cache",
            "$HOME/Library/Application Support/discord/Cache",
            "$HOME/Library/Application Support/Notion/Cache",
            "$HOME/Library/Application Support/Telegram Desktop/tdata/user_data/cache",
        ] {
            stats.track_directory(path);
            user_lines.push(format!("rm -rf \"{path}\"/* 2>/dev/null || true"));
        }
    }
    if request.clear_safari {
        stats.track_directory("$HOME/Library/Caches/com.apple.Safari");
        stats.track_directory("$HOME/Library/Safari/LocalStorage");
        user_lines.push(
            "rm -rf \"$HOME/Library/Caches/com.apple.Safari\"/* 2>/dev/null || true".to_string(),
        );
        user_lines
            .push("rm -rf \"$HOME/Library/Safari/LocalStorage\"/* 2>/dev/null || true".to_string());
    }
    if request.clear_chrome {
        stats.track_directory("$HOME/Library/Caches/Google/Chrome");
        stats.track_directory("$HOME/Library/Application Support/Google/Chrome/Default/Cache");
        user_lines.push(
            "rm -rf \"$HOME/Library/Caches/Google/Chrome\"/* 2>/dev/null || true".to_string(),
        );
        user_lines.push("rm -rf \"$HOME/Library/Application Support/Google/Chrome/Default/Cache\"/* 2>/dev/null || true".to_string());
    }
    if request.clear_edge {
        stats.track_directory("$HOME/Library/Caches/Microsoft Edge");
        stats.track_directory("$HOME/Library/Application Support/Microsoft Edge/Default/Cache");
        user_lines.push(
            "rm -rf \"$HOME/Library/Caches/Microsoft Edge\"/* 2>/dev/null || true".to_string(),
        );
        user_lines.push("rm -rf \"$HOME/Library/Application Support/Microsoft Edge/Default/Cache\"/* 2>/dev/null || true".to_string());
    }
    if request.clear_firefox {
        stats.track_directory("$HOME/Library/Caches/Firefox");
        user_lines
            .push("rm -rf \"$HOME/Library/Caches/Firefox\"/* 2>/dev/null || true".to_string());
    }
    if request.clear_mail {
        stats.track_directory("$HOME/Library/Containers/com.apple.mail/Data/Library/Caches");
        stats
            .track_directory("$HOME/Library/Containers/com.apple.mail/Data/Library/Mail Downloads");
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.apple.mail/Data/Library/Caches\"/* 2>/dev/null || true".to_string());
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.apple.mail/Data/Library/Mail Downloads\"/* 2>/dev/null || true".to_string());
    }
    if request.clear_logs {
        stats.track_directory("$HOME/Library/Logs");
        stats.track_directory("$HOME/Library/Application Support/CrashReporter");
        user_lines.push("rm -rf \"$HOME/Library/Logs\"/* 2>/dev/null || true".to_string());
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/CrashReporter\"/* 2>/dev/null || true"
                .to_string(),
        );
    }
    if request.clear_package_cache {
        stats.track_directory("$HOME/Library/Caches/Homebrew");
        stats.track_directory("$HOME/.npm");
        stats.track_directory("$HOME/Library/Caches/Yarn");
        stats.track_directory("$HOME/Library/pnpm");
        user_lines.push(
            "if command -v npm >/dev/null 2>&1; then npm cache clean --force; fi".to_string(),
        );
        user_lines
            .push("if command -v yarn >/dev/null 2>&1; then yarn cache clean; fi".to_string());
        user_lines
            .push("if command -v pnpm >/dev/null 2>&1; then pnpm store prune; fi".to_string());
        user_lines.push("if command -v brew >/dev/null 2>&1; then brew cleanup -s && rm -rf \"$(brew --cache)\"; fi".to_string());
    }
    if request.empty_trash {
        stats.track_directory("$HOME/.Trash");
        user_lines.push("rm -rf \"$HOME/.Trash\"/* 2>/dev/null || true".to_string());
    }

    run_shell_batches("macOS", log, stats, user_lines, admin_lines, cancel_flag, progress_output)
}

fn run_windows_cleanup(
    request: &CleanerCleanupRequest,
    cancel_flag: &Arc<AtomicBool>,
    progress_output: &Arc<Mutex<String>>,
) -> Result<String, String> {
    let mut log = vec!["开始执行 Windows 垃圾清理...".to_string()];
    let mut user_lines = Vec::new();
    let mut admin_lines = Vec::new();
    let mut stats = CleanupStats::default();
    publish_log(&log, progress_output);

    if cancelled(cancel_flag, &mut log, progress_output) {
        return Ok(log.join("\n\n"));
    }

    if request.clear_system_temp {
        stats.track_directory("%TEMP%");
        stats.track_directory("%LOCALAPPDATA%\\Temp");
        stats.track_directory("C:\\Windows\\Temp");
        user_lines.push("Remove-Item \"$env:TEMP\\*\" -Recurse -Force".to_string());
        user_lines.push("Remove-Item \"$env:LOCALAPPDATA\\Temp\\*\" -Recurse -Force".to_string());
        admin_lines.push("Remove-Item \"C:\\Windows\\Temp\\*\" -Recurse -Force".to_string());
    }
    if request.clear_app_cache {
        stats.track_directory("%LOCALAPPDATA%\\Microsoft\\Windows\\INetCache");
        stats.track_directory("%LOCALAPPDATA%\\Microsoft\\Windows\\Explorer");
        user_lines.push(
            "Remove-Item \"$env:LOCALAPPDATA\\Microsoft\\Windows\\INetCache\\*\" -Recurse -Force"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:LOCALAPPDATA\\Microsoft\\Windows\\Explorer\\thumbcache_*\" -Force"
                .to_string(),
        );
    }
    if request.clear_downloads {
        stats.track_directory("%USERPROFILE%\\Downloads");
        user_lines.push("if (Test-Path \"$env:USERPROFILE\\Downloads\") { Get-ChildItem \"$env:USERPROFILE\\Downloads\" -Force | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue }".to_string());
    }
    add_windows_chat_cleaners(request, &mut stats, &mut user_lines);
    if request.clear_installers {
        stats.track_matching_files(
            "%USERPROFILE%\\Downloads",
            &["exe", "msi", "msix", "msixbundle", "appx", "zip", "7z"],
        );
        stats.track_matching_files(
            "%USERPROFILE%\\Desktop",
            &["exe", "msi", "msix", "msixbundle", "appx", "zip", "7z"],
        );
        user_lines.push("Get-ChildItem \"$env:USERPROFILE\\Downloads\",\"$env:USERPROFILE\\Desktop\" -File -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.Extension -match '^(?i)\\.(exe|msi|msix|msixbundle|appx|zip|7z)$' } | Remove-Item -Force -ErrorAction SilentlyContinue".to_string());
    }
    if request.clear_other_apps {
        for (track, command_path) in [
            ("%APPDATA%\\Cursor\\Cache", "$env:APPDATA\\Cursor\\Cache\\*"),
            ("%APPDATA%\\Code\\Cache", "$env:APPDATA\\Code\\Cache\\*"),
            ("%APPDATA%\\Slack\\Cache", "$env:APPDATA\\Slack\\Cache\\*"),
            ("%APPDATA%\\discord\\Cache", "$env:APPDATA\\discord\\Cache\\*"),
            ("%APPDATA%\\Notion\\Cache", "$env:APPDATA\\Notion\\Cache\\*"),
            (
                "%APPDATA%\\Telegram Desktop\\tdata\\user_data\\cache",
                "$env:APPDATA\\Telegram Desktop\\tdata\\user_data\\cache\\*",
            ),
        ] {
            stats.track_directory(track);
            user_lines.push(format!(
                "Remove-Item \"{command_path}\" -Recurse -Force -ErrorAction SilentlyContinue"
            ));
        }
    }
    if request.clear_chrome {
        stats.track_directory(
            "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache\\Cache_Data",
        );
        user_lines.push("Remove-Item \"$env:LOCALAPPDATA\\Google\\Chrome\\User Data\\Default\\Cache\\Cache_Data\\*\" -Recurse -Force -ErrorAction SilentlyContinue".to_string());
    }
    if request.clear_edge {
        stats.track_directory(
            "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\Cache\\Cache_Data",
        );
        user_lines.push("Remove-Item \"$env:LOCALAPPDATA\\Microsoft\\Edge\\User Data\\Default\\Cache\\Cache_Data\\*\" -Recurse -Force -ErrorAction SilentlyContinue".to_string());
    }
    if request.clear_mail {
        stats.track_directory("%LOCALAPPDATA%\\Microsoft\\Windows\\INetCache\\Content.Outlook");
        user_lines.push("Remove-Item \"$env:LOCALAPPDATA\\Microsoft\\Windows\\INetCache\\Content.Outlook\\*\" -Recurse -Force -ErrorAction SilentlyContinue".to_string());
    }
    if request.clear_logs {
        stats.track_directory("%LOCALAPPDATA%\\CrashDumps");
        stats.track_directory("C:\\ProgramData\\Microsoft\\Windows\\WER");
        user_lines
            .push("Remove-Item \"$env:LOCALAPPDATA\\CrashDumps\\*\" -Recurse -Force".to_string());
        admin_lines.push(
            "Remove-Item \"C:\\ProgramData\\Microsoft\\Windows\\WER\\*\" -Recurse -Force"
                .to_string(),
        );
    }
    if request.clear_package_cache {
        stats.track_directory("%LOCALAPPDATA%\\npm-cache");
        stats.track_directory("%USERPROFILE%\\.nuget\\packages");
        user_lines.push(
            "if (Get-Command npm -ErrorAction SilentlyContinue) { npm cache clean --force }"
                .to_string(),
        );
        user_lines
            .push("Remove-Item \"$env:LOCALAPPDATA\\npm-cache\\*\" -Recurse -Force".to_string());
        user_lines.push(
            "Remove-Item \"$env:USERPROFILE\\.nuget\\packages\\*\" -Recurse -Force".to_string(),
        );
    }
    if request.empty_trash {
        user_lines.push("Clear-RecycleBin -Force".to_string());
    }

    run_shell_batches("Windows", log, stats, user_lines, admin_lines, cancel_flag, progress_output)
}

fn add_windows_chat_cleaners(
    request: &CleanerCleanupRequest,
    stats: &mut CleanupStats,
    user_lines: &mut Vec<String>,
) {
    let mut add = |enabled: bool, track: &'static str, command_path: &'static str| {
        if enabled {
            stats.track_directory(track);
            user_lines.push(format!(
                "Remove-Item \"{command_path}\" -Recurse -Force -ErrorAction SilentlyContinue"
            ));
        }
    };
    add(
        request.clear_wechat_work,
        "%APPDATA%\\Tencent\\WeCom\\XPlugin\\Cache",
        "$env:APPDATA\\Tencent\\WeCom\\XPlugin\\Cache\\*",
    );
    add(
        request.clear_wechat_work,
        "%APPDATA%\\Tencent\\WeCom\\Cache",
        "$env:APPDATA\\Tencent\\WeCom\\Cache\\*",
    );
    add(
        request.clear_wechat,
        "%APPDATA%\\Tencent\\WeChat\\XPlugin\\Plugins\\RadiumWMPF",
        "$env:APPDATA\\Tencent\\WeChat\\XPlugin\\Plugins\\RadiumWMPF\\*",
    );
    add(
        request.clear_wechat,
        "%LOCALAPPDATA%\\Tencent\\WeChat\\Cache",
        "$env:LOCALAPPDATA\\Tencent\\WeChat\\Cache\\*",
    );
    add(
        request.clear_qq,
        "%APPDATA%\\Tencent\\QQ\\NT_QQ\\Cache",
        "$env:APPDATA\\Tencent\\QQ\\NT_QQ\\Cache\\*",
    );
    add(
        request.clear_qq,
        "%LOCALAPPDATA%\\Tencent\\QQ\\Temp",
        "$env:LOCALAPPDATA\\Tencent\\QQ\\Temp\\*",
    );
    add(request.clear_dingtalk, "%APPDATA%\\DingTalk\\Cache", "$env:APPDATA\\DingTalk\\Cache\\*");
    add(request.clear_feishu, "%APPDATA%\\LarkShell\\Cache", "$env:APPDATA\\LarkShell\\Cache\\*");
    add(
        request.clear_feishu,
        "%APPDATA%\\LarkShell\\Local Storage",
        "$env:APPDATA\\LarkShell\\Local Storage\\*",
    );
}

fn run_shell_batches(
    platform: &str,
    mut log: Vec<String>,
    stats: CleanupStats,
    user_lines: Vec<String>,
    admin_lines: Vec<String>,
    cancel_flag: &Arc<AtomicBool>,
    progress_output: &Arc<Mutex<String>>,
) -> Result<String, String> {
    log.push(stats.summary_line());
    publish_log(&log, progress_output);

    if !user_lines.is_empty() {
        if cancelled(cancel_flag, &mut log, progress_output) {
            return Ok(log.join("\n\n"));
        }
        let output =
            user_cleanup_command(&user_lines).map_err(|e| format!("执行用户目录清理失败: {e}"))?;
        log.push(command_output("用户目录清理", &output));
        publish_log(&log, progress_output);
        if cancelled(cancel_flag, &mut log, progress_output) {
            return Ok(log.join("\n\n"));
        }
    }

    if !admin_lines.is_empty() {
        if cancelled(cancel_flag, &mut log, progress_output) {
            return Ok(log.join("\n\n"));
        }
        log.push("检测到系统目录清理，正在请求管理员授权...".to_string());
        publish_log(&log, progress_output);
        let output =
            admin_cleanup_command(&admin_lines).map_err(|e| format!("发起管理员授权失败: {e}"))?;
        log.push(command_output("系统目录清理", &output));
        publish_log(&log, progress_output);
        if !output.status.success() {
            return Err(log.join("\n\n"));
        }
    }

    log.push(stats.actual_removed_line());
    log.push(
        match platform {
            "macOS" => "macOS 清理流程已结束。建议你稍后打开系统存储查看释放空间。",
            "Windows" => "Windows 清理流程已结束。建议你打开系统存储设置确认释放空间。",
            _ => "清理流程已结束。",
        }
        .to_string(),
    );
    publish_log(&log, progress_output);
    Ok(log.join("\n\n"))
}

fn user_cleanup_command(lines: &[String]) -> std::io::Result<std::process::Output> {
    if cfg!(target_os = "macos") {
        Command::new("/bin/bash").args(["-lc", &lines.join("\n")]).output()
    } else {
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &lines.join("; ")])
            .output()
    }
}

fn admin_cleanup_command(lines: &[String]) -> std::io::Result<std::process::Output> {
    if cfg!(target_os = "macos") {
        let script = escape_applescript(&lines.join("\n"));
        Command::new("osascript")
            .args(["-e", &format!("do shell script \"{script}\" with administrator privileges")])
            .output()
    } else {
        let elevated = format!(
            "$p = Start-Process PowerShell -Verb RunAs -Wait -PassThru -ArgumentList '-NoProfile -NonInteractive -Command {}'; exit $p.ExitCode",
            quote_for_single_argument(&lines.join("; "))
        );
        Command::new("powershell").args(["-NoProfile", "-Command", &elevated]).output()
    }
}

fn command_output(label: &str, output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let status = if output.status.success() { "成功" } else { "失败" };
    let mut lines = vec![format!("{label}: {status}")];
    if !stdout.is_empty() {
        lines.push(format!("stdout:\n{stdout}"));
    }
    if !stderr.is_empty() {
        lines.push(format!("stderr:\n{stderr}"));
    }
    lines.join("\n")
}

fn escape_applescript(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn quote_for_single_argument(input: &str) -> String {
    format!("'{}'", input.replace('\'', "''"))
}

fn cancelled(
    cancel_flag: &Arc<AtomicBool>,
    log: &mut Vec<String>,
    progress_output: &Arc<Mutex<String>>,
) -> bool {
    if cancel_flag.load(Ordering::Relaxed) {
        log.push("清理任务已取消，已停止后续步骤。".to_string());
        publish_log(log, progress_output);
        true
    } else {
        false
    }
}

fn publish_log(log: &[String], progress_output: &Arc<Mutex<String>>) {
    let mut output = progress_output.lock();
    *output = log.join("\n\n");
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod run_tests;
