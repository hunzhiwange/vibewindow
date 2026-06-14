use super::*;
use tempfile::TempDir;

fn entry(key: &str, content: &str) -> MemoryEntry {
    MemoryEntry {
        id: key.to_string(),
        key: key.to_string(),
        content: content.to_string(),
        category: MemoryCategory::Core,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        session_id: None,
        score: None,
    }
}

fn memory_with_threshold(workspace: &std::path::Path, threshold: usize) -> LucidMemory {
    let sqlite = SqliteMemory::new(workspace).unwrap();
    LucidMemory::with_options(
        workspace,
        sqlite,
        "missing-lucid-command".to_string(),
        17,
        threshold,
        Duration::from_millis(250),
        Duration::from_millis(250),
        Duration::from_millis(5),
    )
}

#[test]
fn with_options_clamps_threshold_and_builds_command_args() {
    let tmp = TempDir::new().unwrap();
    let memory = memory_with_threshold(tmp.path(), 0);

    assert_eq!(memory.local_hit_threshold, 1);

    let store_args = memory.build_store_args("pref", "Use Rust", &MemoryCategory::Daily);
    assert_eq!(store_args[0], "store");
    assert_eq!(store_args[1], "pref: Use Rust");
    assert_eq!(store_args[2], "--type=context");
    assert_eq!(store_args[3], format!("--project={}", tmp.path().display()));

    let recall_args = memory.build_recall_args("auth");
    assert_eq!(recall_args[0], "context");
    assert_eq!(recall_args[1], "auth");
    assert_eq!(recall_args[2], "--budget=17");
    assert_eq!(recall_args[3], format!("--project={}", tmp.path().display()));
}

#[test]
fn category_mapping_covers_known_custom_and_visual_labels() {
    assert_eq!(LucidMemory::to_lucid_type(&MemoryCategory::Core), "decision");
    assert_eq!(LucidMemory::to_lucid_type(&MemoryCategory::Daily), "context");
    assert_eq!(LucidMemory::to_lucid_type(&MemoryCategory::Conversation), "conversation");
    assert_eq!(
        LucidMemory::to_lucid_type(&MemoryCategory::Custom("notes".to_string())),
        "learning"
    );

    assert_eq!(LucidMemory::to_memory_category("decision"), MemoryCategory::Core);
    assert_eq!(LucidMemory::to_memory_category("solution"), MemoryCategory::Core);
    assert_eq!(LucidMemory::to_memory_category("context"), MemoryCategory::Conversation);
    assert_eq!(LucidMemory::to_memory_category("conversation"), MemoryCategory::Conversation);
    assert_eq!(LucidMemory::to_memory_category("bug"), MemoryCategory::Daily);
    assert_eq!(
        LucidMemory::to_memory_category("visual-note"),
        MemoryCategory::Custom("visual".to_string())
    );
    assert_eq!(
        LucidMemory::to_memory_category("research"),
        MemoryCategory::Custom("research".to_string())
    );
}

#[test]
fn merge_results_deduplicates_case_insensitively_and_respects_limit() {
    let primary = vec![entry("A", "Same"), entry("B", "Second")];
    let secondary = vec![entry("a", "same"), entry("C", "Third"), entry("D", "Fourth")];

    assert!(LucidMemory::merge_results(primary.clone(), secondary.clone(), 0).is_empty());

    let merged = LucidMemory::merge_results(primary, secondary, 3);
    let keys = merged.iter().map(|entry| entry.key.as_str()).collect::<Vec<_>>();

    assert_eq!(keys, vec!["A", "B", "C"]);
}

#[test]
fn parse_lucid_context_ignores_noise_maps_categories_and_scores_results() {
    let raw = r#"
outside
<lucid-context>

not a bullet
- [decision] Use refresh middleware
- [visual-note] Compare screenshot diff
- [bug] Fix panic
- [context] Workspace is src/auth.rs
- [unknown] Keep raw label
- [decision]
</lucid-context>
- [decision] ignored after close
"#;

    let entries = LucidMemory::parse_lucid_context(raw);

    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].id, "lucid:0");
    assert_eq!(entries[0].key, "lucid_0");
    assert_eq!(entries[0].category, MemoryCategory::Core);
    assert_eq!(entries[1].category, MemoryCategory::Custom("visual".to_string()));
    assert_eq!(entries[2].category, MemoryCategory::Daily);
    assert_eq!(entries[3].category, MemoryCategory::Conversation);
    assert_eq!(entries[4].category, MemoryCategory::Custom("unknown".to_string()));
    assert_eq!(entries[0].score, Some(1.0));
    assert_eq!(entries[1].score, Some(0.95));
}

#[test]
fn parse_lucid_context_score_has_floor() {
    let mut raw = String::from("<lucid-context>\n");
    for idx in 0..25 {
        raw.push_str(&format!("- [decision] item {idx}\n"));
    }
    raw.push_str("</lucid-context>\n");

    let entries = LucidMemory::parse_lucid_context(&raw);

    assert_eq!(entries.len(), 25);
    assert_eq!(entries.last().unwrap().score, Some(0.1));
}

#[test]
fn failure_cooldown_can_be_marked_cleared_and_expire() {
    let tmp = TempDir::new().unwrap();
    let memory = memory_with_threshold(tmp.path(), 3);

    assert!(!memory.in_failure_cooldown());
    memory.mark_failure_now();
    assert!(memory.in_failure_cooldown());

    std::thread::sleep(Duration::from_millis(15));
    assert!(!memory.in_failure_cooldown());

    memory.mark_failure_now();
    assert!(memory.in_failure_cooldown());
    memory.clear_failure();
    assert!(!memory.in_failure_cooldown());
}

#[tokio::test]
async fn trait_methods_delegate_to_local_sqlite() {
    let tmp = TempDir::new().unwrap();
    let memory = memory_with_threshold(tmp.path(), 99);

    assert!(memory.health_check().await);
    memory.store("pref", "Rust local fallback", MemoryCategory::Core, Some("s1")).await.unwrap();

    assert_eq!(memory.count().await.unwrap(), 1);
    assert_eq!(memory.get("pref").await.unwrap().unwrap().content, "Rust local fallback");
    assert_eq!(memory.list(Some(&MemoryCategory::Core), Some("s1")).await.unwrap().len(), 1);
    assert!(memory.recall("Rust", 0, Some("s1")).await.unwrap().is_empty());
    assert!(memory.forget("pref").await.unwrap());
    assert!(!memory.forget("pref").await.unwrap());
    assert_eq!(memory.count().await.unwrap(), 0);
}

#[tokio::test]
#[cfg(unix)]
async fn run_lucid_command_raw_returns_stdout_and_reports_stderr_failures() {
    let ok_args = vec!["-c".to_string(), "printf ok".to_string()];
    let output =
        LucidMemory::run_lucid_command_raw("sh", &ok_args, Duration::from_secs(1)).await.unwrap();
    assert_eq!(output, "ok");

    let fail_args = vec!["-c".to_string(), "printf nope >&2; exit 7".to_string()];
    let error = LucidMemory::run_lucid_command_raw("sh", &fail_args, Duration::from_secs(1))
        .await
        .unwrap_err();
    assert!(error.to_string().contains("lucid command failed: nope"));
}
