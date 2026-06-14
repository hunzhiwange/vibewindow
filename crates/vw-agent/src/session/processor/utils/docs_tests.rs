#[test]
fn is_docs_request_requires_docs_list_and_read_intent() {
    assert!(super::is_docs_request("请读取 docs 文件列表"));
    assert!(super::is_docs_request("DOCS 请查看文件 列表"));
    assert!(super::is_docs_request("帮我列出 docs 列表"));

    assert!(!super::is_docs_request("请读取文件列表"));
    assert!(!super::is_docs_request("docs"));
    assert!(!super::is_docs_request("docs 文件列表"));
    assert!(!super::is_docs_request("请读取 docs"));
}

#[test]
fn list_docs_returns_error_when_docs_directory_is_missing() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let root = workspace.path().to_string_lossy().to_string();

    assert_eq!(super::list_docs(Some(&root)), Err("docs 目录不存在".to_string()));
}

#[test]
fn list_docs_walks_visible_files_skips_noise_and_sorts() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let docs = workspace.path().join("docs");
    std::fs::create_dir_all(docs.join("guide")).expect("guide dir");
    std::fs::create_dir_all(docs.join(".hidden")).expect("hidden dir");
    std::fs::create_dir_all(docs.join("node_modules/pkg")).expect("node_modules dir");
    std::fs::create_dir_all(docs.join("target/debug")).expect("target dir");
    std::fs::write(docs.join("zeta.md"), "z").expect("zeta");
    std::fs::write(docs.join("guide/alpha.md"), "a").expect("alpha");
    std::fs::write(docs.join(".hidden/secret.md"), "secret").expect("secret");
    std::fs::write(docs.join("node_modules/pkg/readme.md"), "noise").expect("noise");
    std::fs::write(docs.join("target/debug/out.md"), "noise").expect("target noise");

    let root = workspace.path().to_string_lossy().to_string();
    let docs = super::list_docs(Some(&root)).expect("docs should list");

    assert_eq!(docs, vec!["docs/guide/alpha.md", "docs/zeta.md"]);
}

#[test]
fn list_docs_uses_current_dir_when_root_is_absent() {
    let cwd = std::env::current_dir().expect("current dir");
    let result = super::list_docs(None);

    if cwd.join("docs").exists() {
        let docs = result.expect("docs via current dir");
        assert!(docs.iter().all(|path| path.starts_with("docs/")));
    } else {
        assert_eq!(result, Err("docs 目录不存在".to_string()));
    }
}

#[test]
fn walk_docs_stops_after_depth_limit() {
    let workspace = tempfile::tempdir().expect("temp workspace");
    let docs = workspace.path().join("docs");
    let mut dir = docs.clone();
    std::fs::create_dir_all(&dir).expect("docs dir");
    for idx in 0..12 {
        dir = dir.join(format!("level-{idx}"));
        std::fs::create_dir_all(&dir).expect("nested dir");
    }
    std::fs::write(dir.join("visible-at-depth-12.md"), "visible").expect("visible doc");
    let too_deep = dir.join("level-12");
    std::fs::create_dir_all(&too_deep).expect("too deep dir");
    std::fs::write(too_deep.join("hidden-at-depth-13.md"), "hidden").expect("hidden doc");

    let mut out = Vec::new();
    super::walk_docs(workspace.path(), &docs, &mut out, 0);

    assert!(out.iter().any(|path| path.ends_with("visible-at-depth-12.md")));
    assert!(!out.iter().any(|path| path.ends_with("hidden-at-depth-13.md")));
}
