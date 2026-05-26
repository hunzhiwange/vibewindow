use super::*;

#[cfg(target_arch = "wasm32")]
#[test]
fn wasm_cli_stub_is_explicitly_unsupported() {
    let config = crate::app::agent::config::Config::default();
    let err = handle_command(crate::app::agent::skill::SkillCommands::List, &config).unwrap_err();
    assert!(err.to_string().contains("not supported"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn skills_dir_helper_points_under_workspace() {
    let workspace = std::path::Path::new("/tmp/workspace");
    assert_eq!(skills_dir(workspace), workspace.join("skills"));
}
