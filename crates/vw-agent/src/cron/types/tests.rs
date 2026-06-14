//! Cron 类型转换的单元测试。
//!
//! 当前覆盖 `JobType` 的字符串解析边界，确保配置和外部输入只接受明确支持的任务
//! 类型。

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use super::JobType;

    /// 验证已支持的任务类型可以大小写不敏感地解析。
    #[test]
    fn job_type_try_from_accepts_known_values_case_insensitive() {
        assert_eq!(JobType::try_from("shell").unwrap(), JobType::Shell);
        assert_eq!(JobType::try_from("SHELL").unwrap(), JobType::Shell);
        assert_eq!(JobType::try_from("agent").unwrap(), JobType::Agent);
        assert_eq!(JobType::try_from("AgEnT").unwrap(), JobType::Agent);
    }

    /// 验证空字符串和未知任务类型会被显式拒绝。
    #[test]
    fn job_type_try_from_rejects_invalid_values() {
        assert!(JobType::try_from("").is_err());
        assert!(JobType::try_from("unknown").is_err());
    }

    #[test]
    fn job_type_into_static_str_matches_serialized_names() {
        let shell: &'static str = JobType::Shell.into();
        let agent: &'static str = JobType::Agent.into();

        assert_eq!(shell, "shell");
        assert_eq!(agent, "agent");
    }

    #[test]
    fn session_target_parse_and_as_str_are_case_insensitive() {
        assert_eq!(SessionTarget::parse("main"), SessionTarget::Main);
        assert_eq!(SessionTarget::parse("MAIN").as_str(), "main");
        assert_eq!(SessionTarget::parse("isolated"), SessionTarget::Isolated);
        assert_eq!(SessionTarget::parse("unknown").as_str(), "isolated");
    }

    #[test]
    fn delivery_config_defaults_and_deserialize_defaults_best_effort() {
        let default = DeliveryConfig::default();
        assert_eq!(default.mode, "none");
        assert!(default.channel.is_none());
        assert!(default.to.is_none());
        assert!(default.best_effort);

        let parsed: DeliveryConfig = serde_json::from_str(r#"{"mode":"announce"}"#).unwrap();
        assert_eq!(parsed.mode, "announce");
        assert!(parsed.best_effort);
    }

    #[test]
    fn normalize_fallbacks_trims_empty_values_and_duplicates() {
        let normalized = normalize_fallbacks(vec![
            " gpt-4.1 ".into(),
            "".into(),
            "gpt-4.1".into(),
            "claude".into(),
            "  ".into(),
        ]);

        assert_eq!(normalized, vec!["gpt-4.1".to_string(), "claude".to_string()]);
    }

    #[test]
    fn cron_job_patch_default_leaves_all_fields_unset() {
        let patch = CronJobPatch::default();

        assert!(patch.job_type.is_none());
        assert!(patch.schedule.is_none());
        assert!(patch.command.is_none());
        assert!(patch.prompt.is_none());
        assert!(patch.enabled.is_none());
        assert!(patch.fallbacks.is_none());
        assert!(patch.full_access.is_none());
        assert!(patch.task_pool.is_none());
    }
}
