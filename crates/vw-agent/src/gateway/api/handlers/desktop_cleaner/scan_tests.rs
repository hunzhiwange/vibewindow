#[test]
fn scan_dir_builds_directory_blueprint() {
    let detail = super::scan_dir("Caches", "$HOME/Library/Caches");

    assert_eq!(detail.label, "Caches");
    assert_eq!(detail.path, "$HOME/Library/Caches");
    assert_eq!(detail.kind, super::ScanDetailKind::Directory);
}

#[test]
fn scan_files_builds_extension_blueprint() {
    let extensions = &["log", "tmp"];
    let detail = super::scan_files("Logs", "$TMPDIR", extensions);

    assert_eq!(detail.label, "Logs");
    assert_eq!(detail.path, "$TMPDIR");
    assert_eq!(detail.kind, super::ScanDetailKind::FileExtensions(extensions));
}

#[test]
fn scan_platform_groups_accumulates_totals_and_matched_items() {
    let temp = tempfile::tempdir().expect("temp dir");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir(&cache_dir).expect("cache dir");
    std::fs::write(cache_dir.join("a.tmp"), vec![0_u8; 7]).expect("cache file");
    std::fs::write(temp.path().join("installer.zip"), vec![0_u8; 11]).expect("zip file");
    std::fs::write(temp.path().join("note.txt"), vec![0_u8; 13]).expect("txt file");

    let cache_path = Box::leak(cache_dir.to_string_lossy().into_owned().into_boxed_str());
    let root_path = Box::leak(temp.path().to_string_lossy().into_owned().into_boxed_str());
    let report = super::scan_platform_groups(vec![super::ScanGroupBlueprint {
        id: "group_test",
        title: "测试组",
        subtitle: "本地测试",
        items: vec![
            super::ScanItemBlueprint {
                id: "cache",
                title: "缓存",
                subtitle: "缓存目录",
                sensitive: false,
                details: vec![super::scan_dir("缓存目录", cache_path)],
            },
            super::ScanItemBlueprint {
                id: "installers",
                title: "安装包",
                subtitle: "安装包文件",
                sensitive: true,
                details: vec![super::scan_files("安装包", root_path, &["zip"])],
            },
            super::ScanItemBlueprint {
                id: "empty",
                title: "空项",
                subtitle: "不存在",
                sensitive: false,
                details: vec![super::scan_dir("不存在", "/definitely/missing/vw-cleaner")],
            },
        ],
    }]);

    assert_eq!(report.total_bytes, 18);
    assert_eq!(report.matched_items, 2);
    assert_eq!(report.groups[0].total_bytes, 18);
    assert_eq!(report.groups[0].items[0].total_bytes, 7);
    assert_eq!(report.groups[0].items[1].total_bytes, 11);
    assert_eq!(report.groups[0].items[2].total_bytes, 0);
    assert!(report.groups[0].items[1].sensitive);
}

#[test]
fn unsupported_platform_message_lists_supported_platforms() {
    let message = super::unsupported_platform_message();

    assert!(message.contains("当前系统暂不支持"));
    assert!(message.contains("macOS"));
    assert!(message.contains("Windows"));
}

#[test]
fn platform_blueprints_have_stable_groups_and_items() {
    let macos = super::macos_scan_blueprints();
    let windows = super::windows_scan_blueprints();

    assert_eq!(
        macos.iter().map(|group| group.id).collect::<Vec<_>>(),
        ["group_system", "group_apps", "group_web"]
    );
    assert_eq!(
        windows.iter().map(|group| group.id).collect::<Vec<_>>(),
        ["group_system", "group_apps", "group_web"]
    );
    assert!(macos[0].items.iter().any(|item| item.id == "downloads" && item.sensitive));
    assert!(windows[1].items.iter().any(|item| item.id == "wechat_work" && item.sensitive));
}

#[test]
fn app_item_helpers_build_directory_details() {
    let mac = super::macos_app_item("demo", "Demo", "缓存", &[("Cache", "$HOME/Demo")]);
    let win =
        super::windows_app_item("demo", "Demo", "缓存", false, &[("Cache", "%APPDATA%\\Demo")]);

    assert_eq!(mac.id, "demo");
    assert!(mac.sensitive);
    assert_eq!(mac.details[0].kind, super::ScanDetailKind::Directory);
    assert_eq!(win.id, "demo");
    assert!(!win.sensitive);
    assert_eq!(win.details[0].path, "%APPDATA%\\Demo");
}

#[test]
fn scan_cleanup_targets_reports_unsupported_platform_on_linux() {
    if cfg!(any(target_os = "macos", target_os = "windows")) {
        return;
    }

    assert_eq!(super::scan_cleanup_targets(), Err(super::unsupported_platform_message()));
}
