use super::*;

#[test]
fn vcs_info_default_has_no_branch() {
    let info = Info { branch: None };
    assert!(info.branch.is_none());
}
