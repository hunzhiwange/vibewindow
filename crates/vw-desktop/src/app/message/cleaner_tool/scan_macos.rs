//! 承接系统清理工具的扫描阶段，发现可清理目标并生成面向界面的扫描明细。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{ScanGroupBlueprint, ScanItemBlueprint, scan_dir, scan_files};

/// macos_scan_blueprints 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn macos_scan_blueprints() -> Vec<ScanGroupBlueprint> {
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
                ScanItemBlueprint {
                    id: "wechat_work",
                    title: "企业微信",
                    subtitle: "缓存与群文件索引，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir(
                            "企业微信缓存",
                            "$HOME/Library/Containers/com.tencent.WeWorkMac/Data/Library/Caches",
                        ),
                        scan_dir(
                            "群组容器缓存",
                            "$HOME/Library/Group Containers/2N38VWS5BX.com.tencent.WeWorkMac/Library/Caches",
                        ),
                    ],
                },
                ScanItemBlueprint {
                    id: "wechat",
                    title: "微信",
                    subtitle: "会话缓存与下载索引，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir(
                            "微信容器缓存",
                            "$HOME/Library/Containers/com.tencent.xinWeChat/Data/Library/Caches",
                        ),
                        scan_dir(
                            "xwechat 缓存",
                            "$HOME/Library/Containers/com.tencent.xinWeChat/Data/Library/Application Support/com.tencent.xinWeChat/xwechat_files/xwechat/cache",
                        ),
                    ],
                },
                ScanItemBlueprint {
                    id: "qq",
                    title: "QQ",
                    subtitle: "NT QQ 缓存与缩略图，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir(
                            "QQ 容器缓存",
                            "$HOME/Library/Containers/com.tencent.qq/Data/Library/Caches",
                        ),
                        scan_dir(
                            "NT QQ 缓存",
                            "$HOME/Library/Containers/com.tencent.qq/Data/Library/Application Support/QQ/global/nt_qq/Cache",
                        ),
                    ],
                },
                ScanItemBlueprint {
                    id: "dingtalk",
                    title: "钉钉",
                    subtitle: "应用缓存与协作数据索引，默认不勾选",
                    sensitive: true,
                    details: vec![scan_dir(
                        "钉钉缓存",
                        "$HOME/Library/Containers/com.alibaba.DingTalkMac/Data/Library/Caches",
                    )],
                },
                ScanItemBlueprint {
                    id: "feishu",
                    title: "飞书",
                    subtitle: "文档与会话缓存，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir(
                            "飞书主缓存",
                            "$HOME/Library/Containers/com.bytedance.feishu/Data/Library/Caches",
                        ),
                        scan_dir(
                            "飞书应用支持缓存",
                            "$HOME/Library/Containers/com.bytedance.feishu/Data/Library/Application Support/LarkShell",
                        ),
                    ],
                },
            ],
        },
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
        },
    ]
}
#[cfg(test)]
#[path = "scan_macos_tests.rs"]
mod scan_macos_tests;
