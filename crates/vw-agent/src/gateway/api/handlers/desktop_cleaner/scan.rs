//! Gateway-side scan logic for the desktop cleaner.

use super::fs::{directory_size, matching_file_size};
use vw_api_types::cleaner::{
    CleanerScanDetail, CleanerScanGroup, CleanerScanItem, CleanerScanReport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScanDetailKind {
    Directory,
    FileExtensions(&'static [&'static str]),
}

pub(super) struct ScanGroupBlueprint {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) items: Vec<ScanItemBlueprint>,
}

pub(super) struct ScanItemBlueprint {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) subtitle: &'static str,
    pub(super) sensitive: bool,
    pub(super) details: Vec<ScanDetailBlueprint>,
}

pub(super) struct ScanDetailBlueprint {
    pub(super) label: &'static str,
    pub(super) path: &'static str,
    pub(super) kind: ScanDetailKind,
}

pub(super) fn scan_cleanup_targets() -> Result<CleanerScanReport, String> {
    match current_platform() {
        Some(CleanerPlatform::MacOs) => Ok(scan_platform_groups(macos_scan_blueprints())),
        Some(CleanerPlatform::Windows) => Ok(scan_platform_groups(windows_scan_blueprints())),
        None => Err(unsupported_platform_message()),
    }
}

pub(super) fn scan_dir(label: &'static str, path: &'static str) -> ScanDetailBlueprint {
    ScanDetailBlueprint { label, path, kind: ScanDetailKind::Directory }
}

pub(super) fn scan_files(
    label: &'static str,
    path: &'static str,
    extensions: &'static [&'static str],
) -> ScanDetailBlueprint {
    ScanDetailBlueprint { label, path, kind: ScanDetailKind::FileExtensions(extensions) }
}

fn scan_platform_groups(blueprints: Vec<ScanGroupBlueprint>) -> CleanerScanReport {
    let mut total_bytes = 0u64;
    let mut matched_items = 0usize;
    let mut groups = Vec::new();

    for group in blueprints {
        let mut group_total = 0u64;
        let mut items = Vec::new();

        for item in group.items {
            let mut item_total = 0u64;
            let mut details = Vec::new();

            for detail in item.details {
                let detail_bytes = scan_detail_bytes(&detail);
                item_total = item_total.saturating_add(detail_bytes);
                details.push(CleanerScanDetail {
                    label: detail.label.to_string(),
                    path: detail.path.to_string(),
                    total_bytes: detail_bytes,
                });
            }

            if item_total > 0 {
                matched_items += 1;
            }

            group_total = group_total.saturating_add(item_total);
            items.push(CleanerScanItem {
                id: item.id.to_string(),
                title: item.title.to_string(),
                subtitle: item.subtitle.to_string(),
                sensitive: item.sensitive,
                total_bytes: item_total,
                details,
            });
        }

        total_bytes = total_bytes.saturating_add(group_total);
        groups.push(CleanerScanGroup {
            id: group.id.to_string(),
            title: group.title.to_string(),
            subtitle: group.subtitle.to_string(),
            total_bytes: group_total,
            items,
        });
    }

    CleanerScanReport { total_bytes, matched_items, groups }
}

fn scan_detail_bytes(detail: &ScanDetailBlueprint) -> u64 {
    match detail.kind {
        ScanDetailKind::Directory => directory_size(detail.path),
        ScanDetailKind::FileExtensions(extensions) => matching_file_size(detail.path, extensions),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CleanerPlatform {
    MacOs,
    Windows,
}

fn current_platform() -> Option<CleanerPlatform> {
    if cfg!(target_os = "macos") {
        Some(CleanerPlatform::MacOs)
    } else if cfg!(target_os = "windows") {
        Some(CleanerPlatform::Windows)
    } else {
        None
    }
}

pub(super) fn unsupported_platform_message() -> String {
    [
        "当前系统暂不支持该清理工具。",
        "目前仅为 macOS 和 Windows 内置了直接清理逻辑。",
        "如需支持 Linux，可继续扩展一套对应执行策略。",
    ]
    .join("\n")
}

fn macos_scan_blueprints() -> Vec<ScanGroupBlueprint> {
    vec![
        ScanGroupBlueprint {
            id: "group_system",
            title: "系统垃圾",
            subtitle: "系统临时文件、缓存、日志、下载与开发工具缓存",
            items: vec![
                ScanItemBlueprint {
                    id: "system_temp",
                    title: "系统临时文件",
                    subtitle: "临时目录与系统运行过程产生的缓存碎片",
                    sensitive: false,
                    details: vec![
                        scan_dir("系统临时目录", "$TMPDIR"),
                        scan_dir("系统缓存目录", "/private/var/folders"),
                    ],
                },
                ScanItemBlueprint {
                    id: "app_cache",
                    title: "应用缓存与缩略图缓存",
                    subtitle: "浏览器、应用通用缓存与系统缩略图",
                    sensitive: false,
                    details: vec![
                        scan_dir("用户缓存", "$HOME/Library/Caches"),
                        scan_dir("开发缓存", "$HOME/.cache"),
                    ],
                },
                ScanItemBlueprint {
                    id: "logs",
                    title: "日志与崩溃转储",
                    subtitle: "系统运行日志与 CrashReporter 文件",
                    sensitive: false,
                    details: vec![
                        scan_dir("系统日志", "$HOME/Library/Logs"),
                        scan_dir("崩溃报告", "$HOME/Library/Application Support/CrashReporter"),
                    ],
                },
                ScanItemBlueprint {
                    id: "package_cache",
                    title: "开发工具缓存",
                    subtitle: "npm、yarn、pnpm、Homebrew 等常见缓存",
                    sensitive: false,
                    details: vec![
                        scan_dir("Homebrew 缓存", "$HOME/Library/Caches/Homebrew"),
                        scan_dir("npm 缓存", "$HOME/.npm"),
                        scan_dir("Yarn 缓存", "$HOME/Library/Caches/Yarn"),
                        scan_dir("pnpm 缓存", "$HOME/Library/pnpm"),
                    ],
                },
                ScanItemBlueprint {
                    id: "downloads",
                    title: "下载",
                    subtitle: "下载目录中的文件，默认不勾选",
                    sensitive: true,
                    details: vec![scan_dir("下载目录", "$HOME/Downloads")],
                },
                ScanItemBlueprint {
                    id: "trash",
                    title: "废纸篓",
                    subtitle: "永久删除前请确认其中没有待恢复文件",
                    sensitive: false,
                    details: vec![scan_dir("废纸篓", "$HOME/.Trash")],
                },
            ],
        },
        ScanGroupBlueprint {
            id: "group_apps",
            title: "应用垃圾",
            subtitle: "聊天软件、安装包与常用软件缓存",
            items: vec![
                ScanItemBlueprint {
                    id: "installers",
                    title: "安装包",
                    subtitle: "下载目录和桌面的安装包文件，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_files(
                            "下载目录安装包",
                            "$HOME/Downloads",
                            &["dmg", "pkg", "xip", "zip", "iso"],
                        ),
                        scan_files(
                            "桌面安装包",
                            "$HOME/Desktop",
                            &["dmg", "pkg", "xip", "zip", "iso"],
                        ),
                    ],
                },
                ScanItemBlueprint {
                    id: "other_apps",
                    title: "其他应用",
                    subtitle: "常用软件缓存，如 Cursor、VS Code、Slack、Discord、Notion、Telegram",
                    sensitive: false,
                    details: vec![
                        scan_dir("Cursor 缓存", "$HOME/Library/Application Support/Cursor/Cache"),
                        scan_dir("VS Code 缓存", "$HOME/Library/Application Support/Code/Cache"),
                        scan_dir("Slack 缓存", "$HOME/Library/Application Support/Slack/Cache"),
                        scan_dir("Discord 缓存", "$HOME/Library/Application Support/discord/Cache"),
                        scan_dir("Notion 缓存", "$HOME/Library/Application Support/Notion/Cache"),
                        scan_dir(
                            "Telegram 缓存",
                            "$HOME/Library/Application Support/Telegram Desktop/tdata/user_data/cache",
                        ),
                    ],
                },
                macos_app_item(
                    "wechat_work",
                    "企业微信",
                    "缓存与群文件索引，默认不勾选",
                    &[
                        (
                            "企业微信缓存",
                            "$HOME/Library/Containers/com.tencent.WeWorkMac/Data/Library/Caches",
                        ),
                        (
                            "群组容器缓存",
                            "$HOME/Library/Group Containers/2N38VWS5BX.com.tencent.WeWorkMac/Library/Caches",
                        ),
                    ],
                ),
                macos_app_item(
                    "wechat",
                    "微信",
                    "会话缓存与下载索引，默认不勾选",
                    &[
                        (
                            "微信容器缓存",
                            "$HOME/Library/Containers/com.tencent.xinWeChat/Data/Library/Caches",
                        ),
                        (
                            "xwechat 缓存",
                            "$HOME/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/xwechat_files/xwechat/cache",
                        ),
                    ],
                ),
                macos_app_item(
                    "qq",
                    "QQ",
                    "NT QQ 缓存与缩略图，默认不勾选",
                    &[
                        (
                            "QQ 容器缓存",
                            "$HOME/Library/Containers/com.tencent.qq/Data/Library/Caches",
                        ),
                        (
                            "NT QQ 缓存",
                            "$HOME/Library/Containers/com.tencent.qq/Data/Library/Application Support/QQ/global/nt_qq/Cache",
                        ),
                    ],
                ),
                macos_app_item(
                    "dingtalk",
                    "钉钉",
                    "应用缓存与协作数据索引，默认不勾选",
                    &[(
                        "钉钉缓存",
                        "$HOME/Library/Containers/com.alibaba.DingTalkMac/Data/Library/Caches",
                    )],
                ),
                macos_app_item(
                    "feishu",
                    "飞书",
                    "文档与会话缓存，默认不勾选",
                    &[
                        (
                            "飞书主缓存",
                            "$HOME/Library/Containers/com.bytedance.feishu/Data/Library/Caches",
                        ),
                        (
                            "飞书应用支持缓存",
                            "$HOME/Library/Containers/com.bytedance.feishu/Data/Library/Application Support/LarkShell",
                        ),
                    ],
                ),
            ],
        },
        macos_web_group(),
    ]
}

fn macos_app_item(
    id: &'static str,
    title: &'static str,
    subtitle: &'static str,
    paths: &[(&'static str, &'static str)],
) -> ScanItemBlueprint {
    ScanItemBlueprint {
        id,
        title,
        subtitle,
        sensitive: true,
        details: paths.iter().map(|(label, path)| scan_dir(label, path)).collect(),
    }
}

fn macos_web_group() -> ScanGroupBlueprint {
    ScanGroupBlueprint {
        id: "group_web",
        title: "上网垃圾",
        subtitle: "浏览器与邮件缓存，默认保持未勾选",
        items: vec![
            ScanItemBlueprint {
                id: "safari",
                title: "Safari",
                subtitle: "Safari 页面资源与本地网站缓存，默认不勾选",
                sensitive: false,
                details: vec![
                    scan_dir("Safari 缓存", "$HOME/Library/Caches/com.apple.Safari"),
                    scan_dir("Safari 网站数据", "$HOME/Library/Safari/LocalStorage"),
                ],
            },
            ScanItemBlueprint {
                id: "chrome",
                title: "Chrome",
                subtitle: "Chrome 缓存数据，默认不勾选",
                sensitive: false,
                details: vec![
                    scan_dir("Chrome 缓存", "$HOME/Library/Caches/Google/Chrome"),
                    scan_dir(
                        "Chrome 页面缓存",
                        "$HOME/Library/Application Support/Google/Chrome/Default/Cache",
                    ),
                ],
            },
            ScanItemBlueprint {
                id: "edge",
                title: "Edge",
                subtitle: "Edge 浏览器缓存，默认不勾选",
                sensitive: false,
                details: vec![
                    scan_dir("Edge 缓存", "$HOME/Library/Caches/Microsoft Edge"),
                    scan_dir(
                        "Edge 页面缓存",
                        "$HOME/Library/Application Support/Microsoft Edge/Default/Cache",
                    ),
                ],
            },
            ScanItemBlueprint {
                id: "firefox",
                title: "Firefox",
                subtitle: "Firefox 浏览器缓存，默认不勾选",
                sensitive: false,
                details: vec![scan_dir("Firefox 缓存", "$HOME/Library/Caches/Firefox")],
            },
            ScanItemBlueprint {
                id: "mail",
                title: "Mail 缓存",
                subtitle: "邮件附件与索引缓存，默认不勾选",
                sensitive: false,
                details: vec![
                    scan_dir(
                        "Mail 缓存",
                        "$HOME/Library/Containers/com.apple.mail/Data/Library/Caches",
                    ),
                    scan_dir(
                        "Mail 下载缓存",
                        "$HOME/Library/Containers/com.apple.mail/Data/Library/Mail Downloads",
                    ),
                ],
            },
        ],
    }
}

fn windows_scan_blueprints() -> Vec<ScanGroupBlueprint> {
    vec![windows_system_group(), windows_apps_group(), windows_web_group()]
}

fn windows_system_group() -> ScanGroupBlueprint {
    ScanGroupBlueprint {
        id: "group_system",
        title: "系统垃圾",
        subtitle: "系统临时文件、缓存、日志、下载与开发工具缓存",
        items: vec![
            ScanItemBlueprint {
                id: "system_temp",
                title: "系统临时文件",
                subtitle: "临时目录与系统更新过程产生的残留",
                sensitive: false,
                details: vec![
                    scan_dir("用户临时目录", "%TEMP%"),
                    scan_dir("本地临时目录", "%LOCALAPPDATA%\\Temp"),
                    scan_dir("Windows 临时目录", "C:\\Windows\\Temp"),
                ],
            },
            ScanItemBlueprint {
                id: "app_cache",
                title: "应用缓存与缩略图缓存",
                subtitle: "浏览器缓存与 Explorer 缩略图数据库",
                sensitive: false,
                details: vec![
                    scan_dir("INetCache", "%LOCALAPPDATA%\\Microsoft\\Windows\\INetCache"),
                    scan_dir("缩略图缓存", "%LOCALAPPDATA%\\Microsoft\\Windows\\Explorer"),
                ],
            },
            ScanItemBlueprint {
                id: "logs",
                title: "日志与崩溃转储",
                subtitle: "CrashDumps 与 WER 崩溃报告",
                sensitive: false,
                details: vec![
                    scan_dir("CrashDumps", "%LOCALAPPDATA%\\CrashDumps"),
                    scan_dir("Windows Error Reporting", "C:\\ProgramData\\Microsoft\\Windows\\WER"),
                ],
            },
            ScanItemBlueprint {
                id: "package_cache",
                title: "开发工具缓存",
                subtitle: "npm 与 NuGet 等开发依赖缓存",
                sensitive: false,
                details: vec![
                    scan_dir("npm 缓存", "%LOCALAPPDATA%\\npm-cache"),
                    scan_dir("NuGet 包缓存", "%USERPROFILE%\\.nuget\\packages"),
                ],
            },
            ScanItemBlueprint {
                id: "downloads",
                title: "下载",
                subtitle: "下载目录中的文件，默认不勾选",
                sensitive: true,
                details: vec![scan_dir("下载目录", "%USERPROFILE%\\Downloads")],
            },
            ScanItemBlueprint {
                id: "trash",
                title: "回收站",
                subtitle: "永久清空前请确认没有待恢复文件",
                sensitive: false,
                details: vec![scan_dir("回收站", "$Recycle.Bin")],
            },
        ],
    }
}

fn windows_apps_group() -> ScanGroupBlueprint {
    ScanGroupBlueprint {
        id: "group_apps",
        title: "应用垃圾",
        subtitle: "聊天软件、安装包与常用软件缓存",
        items: vec![
            ScanItemBlueprint {
                id: "installers",
                title: "安装包",
                subtitle: "下载目录和桌面的安装包文件，默认不勾选",
                sensitive: true,
                details: vec![
                    scan_files(
                        "下载目录安装包",
                        "%USERPROFILE%\\Downloads",
                        &["exe", "msi", "msix", "msixbundle", "appx", "zip", "7z"],
                    ),
                    scan_files(
                        "桌面安装包",
                        "%USERPROFILE%\\Desktop",
                        &["exe", "msi", "msix", "msixbundle", "appx", "zip", "7z"],
                    ),
                ],
            },
            windows_app_item(
                "other_apps",
                "其他应用",
                "常用软件缓存，如 Cursor、VS Code、Slack、Discord、Notion、Telegram",
                false,
                &[
                    ("Cursor 缓存", "%APPDATA%\\Cursor\\Cache"),
                    ("VS Code 缓存", "%APPDATA%\\Code\\Cache"),
                    ("Slack 缓存", "%APPDATA%\\Slack\\Cache"),
                    ("Discord 缓存", "%APPDATA%\\discord\\Cache"),
                    ("Notion 缓存", "%APPDATA%\\Notion\\Cache"),
                    ("Telegram 缓存", "%APPDATA%\\Telegram Desktop\\tdata\\user_data\\cache"),
                ],
            ),
            windows_app_item(
                "wechat_work",
                "企业微信",
                "插件缓存与会话缓存，默认不勾选",
                true,
                &[
                    ("XPlugin 缓存", "%APPDATA%\\Tencent\\WeCom\\XPlugin\\Cache"),
                    ("WeCom 缓存", "%APPDATA%\\Tencent\\WeCom\\Cache"),
                ],
            ),
            windows_app_item(
                "wechat",
                "微信",
                "插件缓存与本地缓存，默认不勾选",
                true,
                &[
                    ("XPlugin 缓存", "%APPDATA%\\Tencent\\WeChat\\XPlugin\\Plugins\\RadiumWMPF"),
                    ("WeChat Cache", "%LOCALAPPDATA%\\Tencent\\WeChat\\Cache"),
                ],
            ),
            windows_app_item(
                "qq",
                "QQ",
                "NT QQ 缓存与临时文件，默认不勾选",
                true,
                &[
                    ("NT_QQ Cache", "%APPDATA%\\Tencent\\QQ\\NT_QQ\\Cache"),
                    ("QQ Temp", "%LOCALAPPDATA%\\Tencent\\QQ\\Temp"),
                ],
            ),
            windows_app_item(
                "dingtalk",
                "钉钉",
                "应用缓存与会话索引，默认不勾选",
                true,
                &[("DingTalk Cache", "%APPDATA%\\DingTalk\\Cache")],
            ),
            windows_app_item(
                "feishu",
                "飞书",
                "应用缓存与本地文档缓存，默认不勾选",
                true,
                &[
                    ("Feishu Cache", "%APPDATA%\\LarkShell\\Cache"),
                    ("Feishu Local Storage", "%APPDATA%\\LarkShell\\Local Storage"),
                ],
            ),
        ],
    }
}

fn windows_app_item(
    id: &'static str,
    title: &'static str,
    subtitle: &'static str,
    sensitive: bool,
    paths: &[(&'static str, &'static str)],
) -> ScanItemBlueprint {
    ScanItemBlueprint {
        id,
        title,
        subtitle,
        sensitive,
        details: paths.iter().map(|(label, path)| scan_dir(label, path)).collect(),
    }
}

fn windows_web_group() -> ScanGroupBlueprint {
    ScanGroupBlueprint {
        id: "group_web",
        title: "上网垃圾",
        subtitle: "浏览器与邮件缓存，默认保持未勾选",
        items: vec![
            windows_app_item(
                "chrome",
                "Chrome",
                "Chrome 浏览器缓存，默认不勾选",
                false,
                &[(
                    "Chrome 页面缓存",
                    "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache\\Cache_Data",
                )],
            ),
            windows_app_item(
                "edge",
                "Edge",
                "Edge 浏览器缓存，默认不勾选",
                false,
                &[(
                    "Edge 页面缓存",
                    "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\Cache\\Cache_Data",
                )],
            ),
            windows_app_item(
                "mail",
                "Mail 缓存",
                "邮件附件缓存，默认不勾选",
                false,
                &[(
                    "Outlook 临时附件缓存",
                    "%LOCALAPPDATA%\\Microsoft\\Windows\\INetCache\\Content.Outlook",
                )],
            ),
        ],
    }
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod scan_tests;
