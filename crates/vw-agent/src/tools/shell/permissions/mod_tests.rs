use std::path::PathBuf;

use crate::security::AutonomyLevel;
use crate::tools::shell::permissions::{
    Permission, PermissionContext, PermissionMode, PermissionResult,
};
use crate::tools::shell::security::{SecurityCategory, SecurityFinding, Severity};

#[test]
fn permission_result_helpers_build_expected_variants() {
    assert_eq!(PermissionResult::allow().permission, Some(Permission::Allow));
    assert_eq!(
        PermissionResult::deny("blocked").permission,
        Some(Permission::Deny { reason: "blocked".into() })
    );
    assert_eq!(
        PermissionResult::ask("review", Some("warning".into())).permission,
        Some(Permission::Ask { reason: "review".into(), warning: Some("warning".into()) })
    );
}

#[test]
fn permission_result_with_findings_replaces_findings() {
    let findings = vec![SecurityFinding {
        severity: Severity::Warn,
        category: SecurityCategory::UnsafePattern,
        message: "watch this".into(),
        suggestion: Some("review".into()),
    }];

    let result = PermissionResult::allow().with_findings(findings.clone());
    assert_eq!(result.security_findings, findings);
}

#[test]
fn permission_context_new_uses_safe_defaults() {
    let workspace_dir = PathBuf::from("/workspace");
    let context = PermissionContext::new(AutonomyLevel::Supervised, workspace_dir.clone());

    assert_eq!(context.autonomy, AutonomyLevel::Supervised);
    assert!(!context.in_sandbox);
    assert_eq!(context.mode, PermissionMode::Normal);
    assert!(!context.approved);
    assert_eq!(context.workspace_dir, workspace_dir);
    assert!(context.allowed_roots.is_empty());
}
