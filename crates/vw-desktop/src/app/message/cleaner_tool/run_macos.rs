//! 承接系统清理工具的执行阶段，按平台分派清理命令并汇总清理结果。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{CleanupRequest, CleanupStats, cancelled, command_output, escape_applescript};
use std::process::Command;

/// run_macos_cleanup 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn run_macos_cleanup(
    request: &CleanupRequest,
    cancel_flag: &std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<String, String> {
    let mut log = vec!["开始执行 macOS 垃圾清理...".to_string()];
    let mut user_lines = Vec::new();
    let mut admin_lines = Vec::new();
    let mut stats = CleanupStats::default();

    if cancelled(cancel_flag, &mut log) {
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
        user_lines.push(
            "if [ -d \"$HOME/Downloads\" ]; then find \"$HOME/Downloads\" -mindepth 1 -maxdepth 1 -exec rm -rf {} + 2>/dev/null; fi"
                .to_string(),
        );
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
        stats.track_directory(
            "$HOME/Library/Containers/com.bytedance.feishu/Data/Library/Application Support/LarkShell",
        );
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.bytedance.feishu\"/Data/Library/Caches/* 2>/dev/null || true".to_string());
        user_lines.push("rm -rf \"$HOME/Library/Containers/com.bytedance.feishu\"/Data/Library/Application Support/LarkShell/* 2>/dev/null || true".to_string());
    }
    if request.clear_installers {
        stats.track_matching_files("$HOME/Downloads", &["dmg", "pkg", "xip", "zip", "iso"]);
        stats.track_matching_files("$HOME/Desktop", &["dmg", "pkg", "xip", "zip", "iso"]);
        user_lines.push(
            "find \"$HOME/Downloads\" \"$HOME/Desktop\" -type f \\( -iname '*.dmg' -o -iname '*.pkg' -o -iname '*.xip' -o -iname '*.zip' -o -iname '*.iso' \\) -delete 2>/dev/null || true"
                .to_string(),
        );
    }
    if request.clear_other_apps {
        stats.track_directory("$HOME/Library/Application Support/Cursor/Cache");
        stats.track_directory("$HOME/Library/Application Support/Code/Cache");
        stats.track_directory("$HOME/Library/Application Support/Slack/Cache");
        stats.track_directory("$HOME/Library/Application Support/discord/Cache");
        stats.track_directory("$HOME/Library/Application Support/Notion/Cache");
        stats.track_directory(
            "$HOME/Library/Application Support/Telegram Desktop/tdata/user_data/cache",
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/Cursor/Cache\"/* 2>/dev/null || true"
                .to_string(),
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/Code/Cache\"/* 2>/dev/null || true"
                .to_string(),
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/Slack/Cache\"/* 2>/dev/null || true"
                .to_string(),
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/discord/Cache\"/* 2>/dev/null || true"
                .to_string(),
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/Notion/Cache\"/* 2>/dev/null || true"
                .to_string(),
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/Telegram Desktop/tdata/user_data/cache\"/* 2>/dev/null || true"
                .to_string(),
        );
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
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/Google/Chrome/Default/Cache\"/* 2>/dev/null || true"
                .to_string(),
        );
    }
    if request.clear_edge {
        stats.track_directory("$HOME/Library/Caches/Microsoft Edge");
        stats.track_directory("$HOME/Library/Application Support/Microsoft Edge/Default/Cache");
        user_lines.push(
            "rm -rf \"$HOME/Library/Caches/Microsoft Edge\"/* 2>/dev/null || true".to_string(),
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Application Support/Microsoft Edge/Default/Cache\"/* 2>/dev/null || true"
                .to_string(),
        );
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
        user_lines.push(
            "rm -rf \"$HOME/Library/Containers/com.apple.mail/Data/Library/Caches\"/* 2>/dev/null || true"
                .to_string(),
        );
        user_lines.push(
            "rm -rf \"$HOME/Library/Containers/com.apple.mail/Data/Library/Mail Downloads\"/* 2>/dev/null || true"
                .to_string(),
        );
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
        user_lines.push(
            "if command -v brew >/dev/null 2>&1; then brew cleanup -s && rm -rf \"$(brew --cache)\"; fi"
                .to_string(),
        );
    }
    if request.empty_trash {
        stats.track_directory("$HOME/.Trash");
        user_lines.push("rm -rf \"$HOME/.Trash\"/* 2>/dev/null || true".to_string());
    }

    log.push(stats.summary_line());

    if !user_lines.is_empty() {
        if cancelled(cancel_flag, &mut log) {
            return Ok(log.join("\n\n"));
        }
        let output = Command::new("/bin/bash")
            .args(["-lc", &user_lines.join("\n")])
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
        let script = escape_applescript(&admin_lines.join("\n"));
        let output = Command::new("osascript")
            .args(["-e", &format!("do shell script \"{script}\" with administrator privileges")])
            .output()
            .map_err(|e| format!("发起管理员授权失败: {e}"))?;
        log.push(command_output("系统目录清理", &output));
        if !output.status.success() {
            return Err(log.join("\n\n"));
        }
    }

    log.push(stats.actual_removed_line());
    log.push("macOS 清理流程已结束。建议你稍后打开系统存储查看释放空间。".to_string());
    Ok(log.join("\n\n"))
}
#[cfg(test)]
#[path = "run_macos_tests.rs"]
mod run_macos_tests;
