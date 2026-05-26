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
}
