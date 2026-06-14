use super::*;
use crate::session::message;

fn user_message_with_summary(
    id: &str,
    diffs: Vec<snapshot::FileDiff>,
    parts: Vec<message::Part>,
) -> message::WithParts {
    message::WithParts {
        info: message::Info::User(Box::new(message::UserInfo {
            id: id.to_string(),
            session_id: "ses_1".to_string(),
            time: message::UserTime { created: 1 },
            summary: Some(message::FileDiffSummary { title: None, body: None, diffs }),
            agent: "gateway".to_string(),
            model: message::ModelRef { provider_id: String::new(), model_id: String::new() },
            system: None,
            tools: None,
            variant: None,
        })),
        parts,
    }
}

fn modified_file_diff(file: &str, additions: i64, deletions: i64) -> snapshot::FileDiff {
    snapshot::FileDiff {
        file: file.to_string(),
        before: String::new(),
        after: String::new(),
        additions,
        deletions,
        status: Some(snapshot::DiffStatus::Modified),
    }
}

#[test]
fn inputs_use_gateway_field_names() {
    let summarize = serde_json::to_value(SummarizeInput {
        session_id: "s1".to_string(),
        message_id: "m1".to_string(),
    })
    .unwrap();
    assert_eq!(summarize["sessionID"], "s1");
    assert_eq!(summarize["messageID"], "m1");

    let diff =
        serde_json::to_value(DiffInput { session_id: "s1".to_string(), message_id: None }).unwrap();
    assert_eq!(diff["sessionID"], "s1");
    assert!(diff.get("messageID").is_none());
}

#[test]
fn file_diffs_from_patch_parts_counts_unique_files() {
    let messages = vec![message::WithParts {
        info: message::Info::User(Box::new(message::UserInfo {
            id: "msg_1".to_string(),
            session_id: "ses_1".to_string(),
            time: message::UserTime { created: 1 },
            summary: None,
            agent: "gateway".to_string(),
            model: message::ModelRef { provider_id: String::new(), model_id: String::new() },
            system: None,
            tools: None,
            variant: None,
        })),
        parts: vec![
            message::Part::Patch(message::PatchPart {
                base: message::PartBase {
                    id: "prt_1".to_string(),
                    session_id: "ses_1".to_string(),
                    message_id: "msg_1".to_string(),
                },
                hash: "tree_1".to_string(),
                files: vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
            }),
            message::Part::Patch(message::PatchPart {
                base: message::PartBase {
                    id: "prt_2".to_string(),
                    session_id: "ses_1".to_string(),
                    message_id: "msg_1".to_string(),
                },
                hash: "tree_2".to_string(),
                files: vec!["src/main.rs".to_string()],
            }),
        ],
    }];

    let diffs = file_diffs_from_patch_parts(&messages);

    assert_eq!(diffs.len(), 2);
    assert!(diffs.iter().any(|diff| diff.file == "src/main.rs"));
    assert!(diffs.iter().any(|diff| diff.file == "src/lib.rs"));
}

#[test]
fn file_diffs_from_message_summaries_sums_changed_files() {
    let messages = vec![
        user_message_with_summary(
            "msg_1",
            vec![modified_file_diff("src/main.rs", 2, 1), modified_file_diff("src/lib.rs", 1, 0)],
            Vec::new(),
        ),
        user_message_with_summary(
            "msg_2",
            vec![modified_file_diff("src/main.rs", 3, 2)],
            vec![message::Part::Patch(message::PatchPart {
                base: message::PartBase {
                    id: "prt_1".to_string(),
                    session_id: "ses_1".to_string(),
                    message_id: "msg_2".to_string(),
                },
                hash: "tree_1".to_string(),
                files: vec!["src/main.rs".to_string(), "src/new.rs".to_string()],
            })],
        ),
    ];

    let diffs = file_diffs_from_message_summaries(&messages);

    let main = diffs.iter().find(|diff| diff.file == "src/main.rs").unwrap();
    let lib = diffs.iter().find(|diff| diff.file == "src/lib.rs").unwrap();
    let new = diffs.iter().find(|diff| diff.file == "src/new.rs").unwrap();
    assert_eq!(main.additions, 5);
    assert_eq!(main.deletions, 3);
    assert_eq!(lib.additions, 1);
    assert_eq!(lib.deletions, 0);
    assert_eq!(new.additions, 0);
    assert_eq!(new.deletions, 0);
}

#[test]
fn summary_from_diffs_totals_counts_and_file_count() {
    let summary = summary_from_diffs(&[
        modified_file_diff("src/main.rs", 4, 2),
        modified_file_diff("src/lib.rs", 1, 3),
    ]);

    assert_eq!(summary.additions, 5);
    assert_eq!(summary.deletions, 5);
    assert_eq!(summary.files, 2);
    assert!(summary.diffs.is_none());
}

#[test]
fn merge_file_diff_accumulates_and_preserves_useful_fields() {
    let mut diffs = vec![snapshot::FileDiff {
        file: "src/main.rs".to_string(),
        before: String::new(),
        after: "old".to_string(),
        additions: 1,
        deletions: 2,
        status: None,
    }];

    merge_file_diff(
        &mut diffs,
        &snapshot::FileDiff {
            file: "src/main.rs".to_string(),
            before: "before".to_string(),
            after: "after".to_string(),
            additions: 3,
            deletions: 4,
            status: Some(snapshot::DiffStatus::Modified),
        },
    );

    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].additions, 4);
    assert_eq!(diffs[0].deletions, 6);
    assert_eq!(diffs[0].before, "before");
    assert_eq!(diffs[0].after, "after");
    assert_eq!(diffs[0].status, Some(snapshot::DiffStatus::Modified));
}

#[test]
fn unquote_git_path_decodes_octal_common_escapes_and_leaves_plain_text() {
    assert_eq!(unquote_git_path(r#""caf\303\251.txt""#), "café.txt");
    assert_eq!(unquote_git_path(r#""dir\nfile\t\"quoted\".md""#), "dir\nfile\t\"quoted\".md");
    assert_eq!(unquote_git_path("plain.txt"), "plain.txt");
}

#[test]
fn unquote_git_path_handles_dangling_unknown_and_invalid_utf8_escapes() {
    assert_eq!(unquote_git_path(r#""trail\\""#), "trail\\");
    assert_eq!(unquote_git_path(r#""\z\b\f\v""#), "z\u{0008}\u{000c}\u{000b}");
    assert_eq!(unquote_git_path(r#""bad\377utf8""#), "bad�utf8");
    assert_eq!(unquote_git_path(r#""""#), "");
}

#[test]
fn extra_builds_json_map_from_pairs() {
    let map = extra([("title", Value::String("Hello".to_string())), ("count", Value::from(2))]);

    assert_eq!(map.get("title").and_then(Value::as_str), Some("Hello"));
    assert_eq!(map.get("count").and_then(Value::as_i64), Some(2));
}

#[test]
fn compute_diff_uses_message_summary_and_patch_fallback_without_snapshots() {
    let messages = vec![user_message_with_summary(
        "msg_1",
        vec![modified_file_diff("src/main.rs", 2, 1)],
        vec![message::Part::Patch(message::PatchPart {
            base: message::PartBase {
                id: "prt_1".to_string(),
                session_id: "ses_1".to_string(),
                message_id: "msg_1".to_string(),
            },
            hash: "tree_1".to_string(),
            files: vec!["src/main.rs".to_string(), "src/extra.rs".to_string()],
        })],
    )];

    let diffs = compute_diff(&messages).expect("diffs");

    assert!(diffs.iter().any(|diff| diff.file == "src/main.rs" && diff.additions == 2));
    assert!(diffs.iter().any(|diff| diff.file == "src/extra.rs" && diff.additions == 0));
}

#[test]
fn compute_diff_empty_messages_returns_no_diffs() {
    let diffs = compute_diff(&[]).expect("diffs");

    assert!(diffs.is_empty());
}

#[tokio::test]
async fn summarize_message_returns_ok_for_missing_message_without_side_effects() {
    summarize_message("session-missing", "message-missing", &[]).await.unwrap();
}

#[tokio::test]
async fn diff_missing_storage_returns_empty_list() {
    let diffs = diff(DiffInput {
        session_id: format!("missing-summary-diff-{}", std::process::id()),
        message_id: Some("msg".to_string()),
    })
    .await;

    assert!(diffs.is_empty());
}

#[tokio::test]
async fn diff_reads_unquotes_and_rewrites_persisted_diff_paths() {
    let session_id = format!("summary-diff-{}", std::process::id());
    let stored = vec![snapshot::FileDiff {
        file: r#""caf\303\251.txt""#.to_string(),
        before: String::new(),
        after: String::new(),
        additions: 1,
        deletions: 0,
        status: Some(snapshot::DiffStatus::Added),
    }];
    crate::app::agent::storage::write(&["session_diff", &session_id], &stored).await.unwrap();

    let diffs = diff(DiffInput { session_id: session_id.clone(), message_id: None }).await;

    assert_eq!(diffs[0].file, "café.txt");
    let rewritten =
        crate::app::agent::storage::read::<Vec<snapshot::FileDiff>>(&["session_diff", &session_id])
            .await
            .unwrap();
    assert_eq!(rewritten[0].file, "café.txt");
}
