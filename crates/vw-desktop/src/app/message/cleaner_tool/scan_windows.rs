//! 承接系统清理工具的扫描阶段，发现可清理目标并生成面向界面的扫描明细。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::{ScanGroupBlueprint, ScanItemBlueprint, scan_dir, scan_files};

/// windows_scan_blueprints 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn windows_scan_blueprints() -> Vec<ScanGroupBlueprint> {
    vec![
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
                        scan_dir(
                            "Windows Error Reporting",
                            "C:\\ProgramData\\Microsoft\\Windows\\WER",
                        ),
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
                ScanItemBlueprint {
                    id: "other_apps",
                    title: "其他应用",
                    subtitle: "常用软件缓存，如 Cursor、VS Code、Slack、Discord、Notion、Telegram",
                    sensitive: false,
                    details: vec![
                        scan_dir("Cursor 缓存", "%APPDATA%\\Cursor\\Cache"),
                        scan_dir("VS Code 缓存", "%APPDATA%\\Code\\Cache"),
                        scan_dir("Slack 缓存", "%APPDATA%\\Slack\\Cache"),
                        scan_dir("Discord 缓存", "%APPDATA%\\discord\\Cache"),
                        scan_dir("Notion 缓存", "%APPDATA%\\Notion\\Cache"),
                        scan_dir(
                            "Telegram 缓存",
                            "%APPDATA%\\Telegram Desktop\\tdata\\user_data\\cache",
                        ),
                    ],
                },
                ScanItemBlueprint {
                    id: "wechat_work",
                    title: "企业微信",
                    subtitle: "插件缓存与会话缓存，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir("XPlugin 缓存", "%APPDATA%\\Tencent\\WeCom\\XPlugin\\Cache"),
                        scan_dir("WeCom 缓存", "%APPDATA%\\Tencent\\WeCom\\Cache"),
                    ],
                },
                ScanItemBlueprint {
                    id: "wechat",
                    title: "微信",
                    subtitle: "插件缓存与本地缓存，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir(
                            "XPlugin 缓存",
                            "%APPDATA%\\Tencent\\WeChat\\XPlugin\\Plugins\\RadiumWMPF",
                        ),
                        scan_dir("WeChat Cache", "%LOCALAPPDATA%\\Tencent\\WeChat\\Cache"),
                    ],
                },
                ScanItemBlueprint {
                    id: "qq",
                    title: "QQ",
                    subtitle: "NT QQ 缓存与临时文件，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir("NT_QQ Cache", "%APPDATA%\\Tencent\\QQ\\NT_QQ\\Cache"),
                        scan_dir("QQ Temp", "%LOCALAPPDATA%\\Tencent\\QQ\\Temp"),
                    ],
                },
                ScanItemBlueprint {
                    id: "dingtalk",
                    title: "钉钉",
                    subtitle: "应用缓存与会话索引，默认不勾选",
                    sensitive: true,
                    details: vec![scan_dir("DingTalk Cache", "%APPDATA%\\DingTalk\\Cache")],
                },
                ScanItemBlueprint {
                    id: "feishu",
                    title: "飞书",
                    subtitle: "应用缓存与本地文档缓存，默认不勾选",
                    sensitive: true,
                    details: vec![
                        scan_dir("Feishu Cache", "%APPDATA%\\LarkShell\\Cache"),
                        scan_dir("Feishu Local Storage", "%APPDATA%\\LarkShell\\Local Storage"),
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
                    id: "chrome",
                    title: "Chrome",
                    subtitle: "Chrome 浏览器缓存，默认不勾选",
                    sensitive: false,
                    details: vec![scan_dir(
                        "Chrome 页面缓存",
                        "%LOCALAPPDATA%\\Google\\Chrome\\User Data\\Default\\Cache\\Cache_Data",
                    )],
                },
                ScanItemBlueprint {
                    id: "edge",
                    title: "Edge",
                    subtitle: "Edge 浏览器缓存，默认不勾选",
                    sensitive: false,
                    details: vec![scan_dir(
                        "Edge 页面缓存",
                        "%LOCALAPPDATA%\\Microsoft\\Edge\\User Data\\Default\\Cache\\Cache_Data",
                    )],
                },
                ScanItemBlueprint {
                    id: "mail",
                    title: "Mail 缓存",
                    subtitle: "邮件附件缓存，默认不勾选",
                    sensitive: false,
                    details: vec![scan_dir(
                        "Outlook 临时附件缓存",
                        "%LOCALAPPDATA%\\Microsoft\\Windows\\INetCache\\Content.Outlook",
                    )],
                },
            ],
        },
    ]
}
#[cfg(test)]
#[path = "scan_windows_tests.rs"]
mod scan_windows_tests;
