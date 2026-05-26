#[test]
#[cfg(not(target_arch = "wasm32"))]
fn scan_dir_builds_directory_blueprint() {
    let detail = super::scan_dir("Caches", "$HOME/Library/Caches");
    assert_eq!(detail.label, "Caches");
    assert_eq!(detail.path, "$HOME/Library/Caches");
    assert_eq!(detail.kind, super::ScanDetailKind::Directory);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn scan_files_builds_extension_blueprint() {
    let extensions = &["log", "tmp"];
    let detail = super::scan_files("Logs", "$TMPDIR", extensions);
    assert_eq!(detail.label, "Logs");
    assert_eq!(detail.path, "$TMPDIR");
    assert_eq!(detail.kind, super::ScanDetailKind::FileExtensions(extensions));
}
