//! Shell 命令 allowlist 辅助逻辑。
//!
//! 本模块只处理旧 allowlist 语义中的命令名/路径匹配和参数路径筛查。它不直接
//! 做最终授权决策，而是为 `SecurityPolicy` 和权限规则引擎提供局部判断能力。

use super::path_utils::{expand_user_path, looks_like_path};
use super::shell_lexer::{skip_env_assignments, split_unquoted_segments, strip_wrapping_quotes};
use super::shell_redirect::{attached_short_option_value, redirection_target};

/// 判断 allowlist 条目是否匹配当前可执行文件。
///
/// `allowed` 是配置中的允许项，可以是 `*`、命令 basename 或路径；`executable`
/// 是命令行中解析出的可执行文件；`executable_base` 是其 basename。返回 `true`
/// 表示该条目允许执行该命令。空条目永远不匹配，路径条目会先展开 `~`，避免
/// 用户目录写法差异绕过路径比较。
pub fn is_allowlist_entry_match(allowed: &str, executable: &str, executable_base: &str) -> bool {
    let allowed = strip_wrapping_quotes(allowed).trim();
    if allowed.is_empty() {
        return false;
    }

    if allowed == "*" {
        return true;
    }

    if looks_like_path(allowed) {
        let allowed_path = expand_user_path(allowed);
        let executable_path = expand_user_path(executable);
        return executable_path == allowed_path;
    }

    allowed == executable_base
}

/// 从命令参数中找出第一个不允许访问的路径。
///
/// `command` 是待检查的 shell 命令，`is_path_allowed` 是调用方提供的路径授权
/// 谓词。返回第一个被拒绝的路径字符串；如果未发现路径参数或所有路径都允许，
/// 返回 `None`。
pub fn find_forbidden_path_argument<F>(command: &str, is_path_allowed: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    let forbidden_candidate = |raw: &str| {
        let candidate = strip_wrapping_quotes(raw).trim();
        if candidate.is_empty() || candidate.contains("://") {
            return None;
        }
        // URL 由上面的协议检查排除；这里仅把看起来像文件系统路径的参数交给
        // 策略层，避免把普通标志值误判为路径。
        if looks_like_path(candidate) && !is_path_allowed(candidate) {
            Some(candidate.to_string())
        } else {
            None
        }
    };

    for segment in split_unquoted_segments(command) {
        let cmd_part = skip_env_assignments(&segment);
        let mut words = cmd_part.split_whitespace();
        let Some(executable) = words.next() else {
            continue;
        };

        if let Some(target) = redirection_target(strip_wrapping_quotes(executable)) {
            if let Some(blocked) = forbidden_candidate(target) {
                return Some(blocked);
            }
        }

        for token in words {
            let candidate = strip_wrapping_quotes(token).trim();
            if candidate.is_empty() || candidate.contains("://") {
                continue;
            }

            if let Some(target) = redirection_target(candidate) {
                if let Some(blocked) = forbidden_candidate(target) {
                    return Some(blocked);
                }
            }

            if candidate.starts_with('-') {
                // 许多工具把路径塞进 `--flag=value` 或 `-ofile`，需要检查值部分；
                // 但不把普通开关本身当作路径，避免扩大误报面。
                if let Some((_, value)) = candidate.split_once('=') {
                    if let Some(blocked) = forbidden_candidate(value) {
                        return Some(blocked);
                    }
                }
                if let Some(value) = attached_short_option_value(candidate) {
                    if let Some(blocked) = forbidden_candidate(value) {
                        return Some(blocked);
                    }
                }
                continue;
            }

            if let Some(blocked) = forbidden_candidate(candidate) {
                return Some(blocked);
            }
        }
    }

    None
}

#[cfg(test)]
#[path = "allowlist_tests.rs"]
mod allowlist_tests;
