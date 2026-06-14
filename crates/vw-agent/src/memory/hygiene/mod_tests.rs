mod additional_hygiene_tests {
    use super::super::*;
    use crate::app::agent::config::MemoryConfig;
    use chrono::{NaiveDate, Utc};
    use std::time::{Duration as StdDuration, SystemTime};
    use tempfile::TempDir;

    fn cfg_disabled() -> MemoryConfig {
        MemoryConfig { hygiene_enabled: false, ..MemoryConfig::default() }
    }

    #[test]
    fn hygiene_report_total_actions_sums_every_counter() {
        let report = HygieneReport {
            archived_memory_files: 1,
            archived_session_files: 2,
            purged_memory_archives: 3,
            purged_session_archives: 4,
            pruned_conversation_rows: 5,
        };

        assert_eq!(report.total_actions(), 15);
    }

    #[test]
    fn run_if_due_disabled_skips_state_and_storage_writes() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path();
        let state = state_path(workspace).unwrap();

        run_if_due(&cfg_disabled(), workspace).unwrap();

        assert!(!state.exists());
    }

    #[test]
    fn should_run_now_handles_missing_recent_old_and_invalid_state() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path();
        let state = state_path(workspace).unwrap();

        assert!(should_run_now(workspace).unwrap());

        write_state(workspace, &HygieneReport::default()).unwrap();
        assert!(!should_run_now(workspace).unwrap());

        let old_state = HygieneState {
            last_run_at: Some((Utc::now() - chrono::Duration::hours(13)).to_rfc3339()),
            last_report: HygieneReport::default(),
        };
        std::fs::write(&state, serde_json::to_vec(&old_state).unwrap()).unwrap();
        assert!(should_run_now(workspace).unwrap());

        let missing_timestamp = HygieneState {
            last_run_at: None,
            last_report: HygieneReport::default(),
        };
        std::fs::write(&state, serde_json::to_vec(&missing_timestamp).unwrap()).unwrap();
        assert!(should_run_now(workspace).unwrap());

        let invalid_timestamp = HygieneState {
            last_run_at: Some("not-a-date".to_string()),
            last_report: HygieneReport::default(),
        };
        std::fs::write(&state, serde_json::to_vec(&invalid_timestamp).unwrap()).unwrap();
        assert!(should_run_now(workspace).unwrap());

        std::fs::write(&state, b"not json").unwrap();
        assert!(should_run_now(workspace).unwrap());
    }

    #[test]
    fn archive_daily_memory_files_ignores_non_candidates_and_uses_unique_archive_name() {
        let tmp = TempDir::new().unwrap();
        let storage = tmp.path();
        let memory_dir = storage.join("memory");
        let archive_dir = memory_dir.join("archive");
        std::fs::create_dir_all(&archive_dir).unwrap();

        let old = (chrono::Local::now().date_naive() - chrono::Duration::days(10))
            .format("%Y-%m-%d")
            .to_string();
        let source = memory_dir.join(format!("{old}.md"));
        let existing_archive = archive_dir.join(format!("{old}.md"));
        let renamed_archive = archive_dir.join(format!("{old}_1.md"));
        let invalid = memory_dir.join("not-a-date.md");
        let non_markdown = memory_dir.join(format!("{old}.txt"));
        let subdir = memory_dir.join(format!("{old}_subdir.md"));

        std::fs::write(&source, "old").unwrap();
        std::fs::write(&existing_archive, "already archived").unwrap();
        std::fs::write(&invalid, "invalid").unwrap();
        std::fs::write(&non_markdown, "text").unwrap();
        std::fs::create_dir_all(&subdir).unwrap();

        assert_eq!(archive_daily_memory_files(storage, 7).unwrap(), 1);
        assert!(!source.exists());
        assert!(existing_archive.exists());
        assert!(renamed_archive.exists());
        assert!(invalid.exists());
        assert!(non_markdown.exists());
        assert!(subdir.is_dir());
    }

    #[test]
    fn purge_session_archives_removes_old_date_prefixed_files_only() {
        let tmp = TempDir::new().unwrap();
        let storage = tmp.path();
        let archive_dir = storage.join("sessions").join("archive");
        std::fs::create_dir_all(&archive_dir).unwrap();

        let old = (chrono::Local::now().date_naive() - chrono::Duration::days(40))
            .format("%Y-%m-%d")
            .to_string();
        let recent = (chrono::Local::now().date_naive() - chrono::Duration::days(2))
            .format("%Y-%m-%d")
            .to_string();
        let old_file = archive_dir.join(format!("{old}-agent.log"));
        let recent_file = archive_dir.join(format!("{recent}-agent.log"));
        let invalid_file = archive_dir.join("manual-session.log");

        std::fs::write(&old_file, "old").unwrap();
        std::fs::write(&recent_file, "recent").unwrap();
        std::fs::write(&invalid_file, "manual").unwrap();

        assert_eq!(purge_session_archives(storage, 30).unwrap(), 1);
        assert!(!old_file.exists());
        assert!(recent_file.exists());
        assert!(invalid_file.exists());
    }

    #[test]
    fn zero_day_and_missing_directories_are_noops() {
        let tmp = TempDir::new().unwrap();
        let storage = tmp.path();

        assert_eq!(archive_daily_memory_files(storage, 0).unwrap(), 0);
        assert_eq!(archive_daily_memory_files(storage, 7).unwrap(), 0);
        assert_eq!(archive_session_files(storage, 0).unwrap(), 0);
        assert_eq!(archive_session_files(storage, 7).unwrap(), 0);
        assert_eq!(purge_memory_archives(storage, 0).unwrap(), 0);
        assert_eq!(purge_memory_archives(storage, 30).unwrap(), 0);
        assert_eq!(purge_session_archives(storage, 0).unwrap(), 0);
        assert_eq!(purge_session_archives(storage, 30).unwrap(), 0);
        assert_eq!(prune_conversation_rows(storage, 0).unwrap(), 0);
        assert_eq!(prune_conversation_rows(storage, 30).unwrap(), 0);
    }

    #[test]
    fn date_and_filename_helpers_cover_edges() {
        let leap_day = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();

        assert_eq!(memory_date_from_filename("2024-02-29.md"), Some(leap_day));
        assert_eq!(memory_date_from_filename("2024-02-29_notes.md"), Some(leap_day));
        assert_eq!(memory_date_from_filename("2024-02-29.txt"), None);
        assert_eq!(memory_date_from_filename("2024-02-30.md"), None);
        assert_eq!(date_prefix("2024-02-29-session.json"), Some(leap_day));
        assert_eq!(date_prefix("short"), None);
        assert_eq!(date_prefix("not-a-date-session"), None);
        assert_eq!(split_name("file.txt"), ("file", "txt"));
        assert_eq!(split_name("multi.part.name"), ("multi.part", "name"));
        assert_eq!(split_name("noext"), ("noext", ""));
    }

    #[test]
    fn archive_path_and_age_helpers_are_conservative() {
        let tmp = TempDir::new().unwrap();
        let archive_dir = tmp.path().join("archive");
        std::fs::create_dir_all(&archive_dir).unwrap();
        let source = tmp.path().join("event.log");
        let target = archive_dir.join("event.log");
        std::fs::write(&source, "event").unwrap();
        std::fs::write(&target, "existing").unwrap();

        assert_eq!(unique_archive_target(&archive_dir, "fresh.log"), archive_dir.join("fresh.log"));
        assert_eq!(unique_archive_target(&archive_dir, "event.log"), archive_dir.join("event_1.log"));
        assert!(is_older_than(&source, SystemTime::now() + StdDuration::from_secs(1)));
        assert!(!is_older_than(&tmp.path().join("missing.log"), SystemTime::now()));

        move_to_archive(&source, &archive_dir).unwrap();
        assert!(!source.exists());
        assert!(archive_dir.join("event_1.log").exists());
    }
}
