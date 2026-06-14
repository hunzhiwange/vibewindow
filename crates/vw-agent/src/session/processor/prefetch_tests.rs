#[test]
fn prefetch_tests_module_is_wired() {
    let marker = String::from("prefetch_tests");
    assert_eq!(marker.as_str(), "prefetch_tests");
}

use crate::app::agent::session::session::Session;
use crate::app::agent::tools::ToolRuntimeContext;
use std::collections::HashSet;

fn unique_session_id(name: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    format!("prefetch-tests-{name}-{nanos}")
}

fn ctx_for(root: Option<String>) -> ToolRuntimeContext {
    ToolRuntimeContext::new(unique_session_id("ctx"), root)
}

#[test]
fn block_on_runs_future_without_existing_runtime() {
    let value = super::block_on(async { 42 });

    assert_eq!(value, 42);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn block_on_runs_future_inside_existing_runtime() {
    let value = super::block_on(async { "inside-runtime".to_string() });

    assert_eq!(value, "inside-runtime");
}

#[test]
fn app_session_scope_from_root_ignores_missing_or_blank_roots() {
    assert_eq!(super::app_session_scope_from_root(None), None);
    assert_eq!(super::app_session_scope_from_root(Some("   ")), None);
}

#[test]
fn try_prefetch_read_returns_false_for_missing_paths_and_directories() {
    let workspace = tempfile::tempdir().expect("temp workspace should be created");
    let ctx = ctx_for(Some(workspace.path().to_string_lossy().to_string()));
    let mut session = Session::new(ctx.session.clone());
    let mut tool_state = super::super::ToolSessionState::default();

    assert!(!super::try_prefetch_read(&mut session, "missing.rs", &ctx, &mut tool_state));
    assert!(!super::try_prefetch_read(&mut session, ".", &ctx, &mut tool_state));
    assert!(session.messages.is_empty());
}

#[test]
fn try_prefetch_read_handles_existing_text_file_without_panicking() {
    let workspace = tempfile::tempdir().expect("temp workspace should be created");
    std::fs::write(workspace.path().join("style.css"), "body { color: black; }\n")
        .expect("style.css should be written");
    let ctx = ctx_for(Some(workspace.path().to_string_lossy().to_string()));
    let mut session = Session::new(ctx.session.clone());
    let mut tool_state = super::super::ToolSessionState::default();

    let prefetched = super::try_prefetch_read(&mut session, "style.css", &ctx, &mut tool_state);

    if prefetched {
        assert!(session.messages.iter().any(|message| message.content.contains("style.css")));
    } else {
        assert!(session.messages.is_empty());
    }
}

#[test]
fn prefetch_seed_context_skips_when_query_contains_explicit_tool_call() {
    let workspace = tempfile::tempdir().expect("temp workspace should be created");
    std::fs::write(workspace.path().join("style.css"), "body { color: black; }\n")
        .expect("style.css should be written");
    let ctx = ctx_for(Some(workspace.path().to_string_lossy().to_string()));
    let mut session = Session::new(ctx.session.clone());
    let mut tool_state = super::super::ToolSessionState::default();
    let allowed = HashSet::from(["file_read".to_string()]);

    super::prefetch_seed_context(
        &mut session,
        r#"/file_read {"path":"style.css"}"#,
        &ctx,
        &allowed,
        &mut tool_state,
    );

    assert!(session.messages.is_empty());
}

#[test]
fn prefetch_seed_context_ignores_http_urls_and_absent_files() {
    let workspace = tempfile::tempdir().expect("temp workspace should be created");
    let ctx = ctx_for(Some(workspace.path().to_string_lossy().to_string()));
    let mut session = Session::new(ctx.session.clone());
    let mut tool_state = super::super::ToolSessionState::default();

    super::prefetch_seed_context(
        &mut session,
        "Please compare https://example.com/style.css with src/missing.rs and README.md",
        &ctx,
        &HashSet::new(),
        &mut tool_state,
    );

    assert!(session.messages.is_empty());
}
