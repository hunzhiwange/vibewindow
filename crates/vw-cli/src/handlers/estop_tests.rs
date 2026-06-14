use super::{build_engage_level, build_resume_selector};
use crate::cli::EstopLevelArg;
use vw_agent::security::{EstopLevel, ResumeSelector};

#[test]
fn build_engage_level_validates_argument_combinations() {
    assert!(matches!(build_engage_level(None, vec![], vec![]).unwrap(), EstopLevel::KillAll));
    assert!(matches!(
        build_engage_level(Some(EstopLevelArg::NetworkKill), vec![], vec![]).unwrap(),
        EstopLevel::NetworkKill
    ));
    assert!(matches!(
        build_engage_level(
            Some(EstopLevelArg::DomainBlock),
            vec!["example.com".to_string()],
            vec![],
        )
        .unwrap(),
        EstopLevel::DomainBlock(domains) if domains == vec!["example.com".to_string()]
    ));
    assert!(build_engage_level(Some(EstopLevelArg::ToolFreeze), vec![], vec![]).is_err());
    assert!(build_engage_level(Some(EstopLevelArg::KillAll), vec!["x".into()], vec![]).is_err());
}

#[test]
fn build_resume_selector_allows_one_scope_only() {
    assert!(matches!(
        build_resume_selector(true, vec![], vec![]).unwrap(),
        ResumeSelector::Network
    ));
    assert!(matches!(
        build_resume_selector(false, vec!["example.com".into()], vec![]).unwrap(),
        ResumeSelector::Domains(domains) if domains == vec!["example.com".to_string()]
    ));
    assert!(matches!(
        build_resume_selector(false, vec![], vec![]).unwrap(),
        ResumeSelector::KillAll
    ));
    assert!(build_resume_selector(true, vec!["example.com".into()], vec![]).is_err());
}
