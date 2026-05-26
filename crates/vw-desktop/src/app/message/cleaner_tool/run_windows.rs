//! 承接系统清理工具的执行阶段，按平台分派清理命令并汇总清理结果。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{CleanupRequest, CleanupStats, cancelled, command_output, quote_for_single_argument};
use std::process::Command;

/// run_windows_cleanup 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn run_windows_cleanup(
    request: &CleanupRequest,
    cancel_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<String, String> {
    let mut log = vec!["开始执行 Windows 垃圾清理...".to_string()];
    let mut user_lines = Vec::new();
    let mut admin_lines = Vec::new();
    let mut stats = CleanupStats::default();

    if cancelled(cancel_flag, &mut log) {
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
        user_lines.push(
            "if (Test-Path \"$env:USERPROFILE\\Downloads\") { Get-ChildItem \"$env:USERPROFILE\\Downloads\" -Force | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue }"
                .to_string(),
        );
    }
    if request.clear_wechat_work {
        stats.track_directory("%APPDATA%\\Tencent\\WeCom\\XPlugin\\Cache");
        stats.track_directory("%APPDATA%\\Tencent\\WeCom\\Cache");
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Tencent\\WeCom\\XPlugin\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Tencent\\WeCom\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_wechat {
        stats.track_directory("%APPDATA%\\Tencent\\WeChat\\XPlugin\\Plugins\\RadiumWMPF");
        stats.track_directory("%LOCALAPPDATA%\\Tencent\\WeChat\\Cache");
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Tencent\\WeChat\\XPlugin\\Plugins\\RadiumWMPF\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:LOCALAPPDATA\\Tencent\\WeChat\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_qq {
        stats.track_directory("%APPDATA%\\Tencent\\QQ\\NT_QQ\\Cache");
        stats.track_directory("%LOCALAPPDATA%\\Tencent\\QQ\\Temp");
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Tencent\\QQ\\NT_QQ\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:LOCALAPPDATA\\Tencent\\QQ\\Temp\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_dingtalk {
        stats.track_directory("%APPDATA%\\DingTalk\\Cache");
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\DingTalk\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_feishu {
        stats.track_directory("%APPDATA%\\LarkShell\\Cache");
        stats.track_directory("%APPDATA%\\LarkShell\\Local Storage");
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\LarkShell\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\LarkShell\\Local Storage\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_installers {
        stats.track_matching_files(
            "%USERPROFILE%\\Downloads",
            &["exe", "msi", "msix", "msixbundle", "appx", "zip", "7z"],
        );
        stats.track_matching_files(
            "%USERPROFILE%\\Desktop",
            &["exe", "msi", "msix", "msixbundle", "appx", "zip", "7z"],
        );
        user_lines.push(
            "Get-ChildItem \"$env:USERPROFILE\\Downloads\",\"$env:USERPROFILE\\Desktop\" -File -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.Extension -match '^(?i)\\.(exe|msi|msix|msixbundle|appx|zip|7z)$' } | Remove-Item -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_other_apps {
        stats.track_directory("%APPDATA%\\Cursor\\Cache");
        stats.track_directory("%APPDATA%\\Code\\Cache");
        stats.track_directory("%APPDATA%\\Slack\\Cache");
        stats.track_directory("%APPDATA%\\discord\\Cache");
        stats.track_directory("%APPDATA%\\Notion\\Cache");
        stats.track_directory("%APPDATA%\\Telegram Desktop\\tdata\\user_data\\cache");
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Cursor\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Code\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Slack\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\discord\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Notion\\Cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
        user_lines.push(
            "Remove-Item \"$env:APPDATA\\Telegram Desktop\\tdata\\user_data\\cache\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_chrome {
        stats.track_directory(
            "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache\\Cache_Data",
        );
        user_lines.push(
            "Remove-Item \"$env:LOCALAPPDATA\\Google\\Chrome\\User Data\\Default\\Cache\\Cache_Data\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_edge {
        stats.track_directory(
            "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\Cache\\Cache_Data",
        );
        user_lines.push(
            "Remove-Item \"$env:LOCALAPPDATA\\Microsoft\\Edge\\User Data\\Default\\Cache\\Cache_Data\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
    }
    if request.clear_mail {
        stats.track_directory("%LOCALAPPDATA%\\Microsoft\\Windows\\INetCache\\Content.Outlook");
        user_lines.push(
            "Remove-Item \"$env:LOCALAPPDATA\\Microsoft\\Windows\\INetCache\\Content.Outlook\\*\" -Recurse -Force -ErrorAction SilentlyContinue"
                .to_string(),
        );
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

    log.push(stats.summary_line());

    if !user_lines.is_empty() {
        if cancelled(cancel_flag, &mut log) {
            return Ok(log.join("\n\n"));
        }
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &user_lines.join("; ")])
            .output()
            .map_err(|e| format!("执行用户目录清理失败: {e}"))?;
        log.push(command_output("用户目录清理", &output));
        if cancelled(cancel_flag, &mut log) {
            return Ok(log.join("\n\n"));
        }
    }

    if !admin_lines.is_empty() {
        if cancelled(cancel_flag, &mut log) {
            return Ok(log.join("\n\n"));
        }
        log.push("检测到系统目录清理，正在请求管理员授权...".to_string());
        let elevated = format!(
            "$p = Start-Process PowerShell -Verb RunAs -Wait -PassThru -ArgumentList '-NoProfile -NonInteractive -Command {}'; exit $p.ExitCode",
            quote_for_single_argument(&admin_lines.join("; "))
        );
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &elevated])
            .output()
            .map_err(|e| format!("发起管理员授权失败: {e}"))?;
        log.push(command_output("系统目录清理", &output));
        if !output.status.success() {
            return Err(log.join("\n\n"));
        }
    }

    log.push(stats.actual_removed_line());
    log.push("Windows 清理流程已结束。建议你打开系统存储设置确认释放空间。".to_string());
    Ok(log.join("\n\n"))
}
#[cfg(test)]
#[path = "run_windows_tests.rs"]
mod run_windows_tests;
