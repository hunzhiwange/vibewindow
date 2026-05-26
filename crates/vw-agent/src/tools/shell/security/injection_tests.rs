//! 注入风险筛选辅助函数的行为测试。

use super::SecurityPipeline;
use super::injection::{has_blocking_injection, injection_findings};

#[test]
fn injection_helpers_filter_injection_findings() {
    let report = SecurityPipeline::new(true).validate_command("echo $(uname -a)");
    assert!(has_blocking_injection(&report));
    assert!(!injection_findings(&report).is_empty());
}
