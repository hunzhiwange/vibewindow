#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("programs_tests"));
}

#[test]
fn binary_names_match_current_platform() {
    if cfg!(windows) {
        assert_eq!(super::opencode_binary_name(), "opencode.exe");
        assert_eq!(super::claude_binary_name(), "claude.exe");
    } else {
        assert_eq!(super::opencode_binary_name(), "opencode");
        assert_eq!(super::claude_binary_name(), "claude");
    }
}

#[test]
fn program_matchers_compare_file_name_case_insensitively() {
    let opencode = if cfg!(windows) { "C:\\tools\\OPENCODE.EXE" } else { "/tools/opencode" };
    let claude = if cfg!(windows) { "C:\\tools\\CLAUDE.EXE" } else { "/tools/claude" };

    assert!(super::is_opencode_program(opencode));
    assert!(super::is_claude_program(claude));
    assert!(!super::is_opencode_program("/tools/claude"));
    assert!(!super::is_claude_program("/tools/opencode"));
}

#[test]
fn select_opencode_prefers_explicit_existing_binary() {
    let temp = tempfile::TempDir::new().expect("temp dir should be created");
    let explicit = temp.path().join(super::opencode_binary_name());
    std::fs::write(&explicit, "").expect("explicit binary should be written");

    let (program, args) = super::select_opencode_program_and_prefix_args(
        Some(explicit.to_string_lossy().to_string()),
        None,
        Some(std::path::PathBuf::from("/resolved/opencode")),
        Some("bunx".to_string()),
        Some("npx".to_string()),
    );

    assert_eq!(program, explicit.to_string_lossy().as_ref());
    assert!(args.is_empty());
}

#[test]
fn select_opencode_uses_home_install_before_path_lookup() {
    let temp = tempfile::TempDir::new().expect("temp dir should be created");
    let home_bin = temp.path().join(".opencode").join("bin");
    std::fs::create_dir_all(&home_bin).expect("home opencode dir should be created");
    let home_opencode = home_bin.join(super::opencode_binary_name());
    std::fs::write(&home_opencode, "").expect("home opencode should be written");

    let (program, args) = super::select_opencode_program_and_prefix_args(
        Some(temp.path().join("missing").to_string_lossy().to_string()),
        Some(temp.path()),
        Some(std::path::PathBuf::from("/resolved/opencode")),
        Some("bunx".to_string()),
        Some("npx".to_string()),
    );

    assert_eq!(program, home_opencode.to_string_lossy().as_ref());
    assert!(args.is_empty());
}

#[test]
fn select_opencode_falls_back_through_resolved_bunx_npx_and_name() {
    let (program, args) = super::select_opencode_program_and_prefix_args(
        None,
        None,
        Some(std::path::PathBuf::from("/resolved/opencode")),
        Some("bunx".to_string()),
        Some("npx".to_string()),
    );
    assert_eq!(program, "/resolved/opencode");
    assert!(args.is_empty());

    let (program, args) = super::select_opencode_program_and_prefix_args(
        None,
        None,
        None,
        Some("bunx".to_string()),
        Some("npx".to_string()),
    );
    assert_eq!(program, "bunx");
    assert_eq!(args, vec!["opencode-ai@latest".to_string()]);

    let (program, args) =
        super::select_opencode_program_and_prefix_args(None, None, None, None, Some("npx".into()));
    assert_eq!(program, "npx");
    assert_eq!(args, vec!["-y".to_string(), "opencode-ai@latest".to_string()]);

    let (program, args) =
        super::select_opencode_program_and_prefix_args(None, None, None, None, None);
    assert_eq!(program, "opencode");
    assert!(args.is_empty());
}

#[test]
fn select_claude_prefers_explicit_then_home_then_resolved_then_name() {
    let temp = tempfile::TempDir::new().expect("temp dir should be created");
    let explicit = temp.path().join("explicit").join(super::claude_binary_name());
    std::fs::create_dir_all(explicit.parent().unwrap()).expect("explicit dir should be created");
    std::fs::write(&explicit, "").expect("explicit binary should be written");

    assert_eq!(
        super::select_claude_program(Some(explicit.to_string_lossy().to_string()), None, None),
        explicit.to_string_lossy().as_ref()
    );

    let home_bin = temp.path().join(".claude").join("local");
    std::fs::create_dir_all(&home_bin).expect("home claude dir should be created");
    let home_claude = home_bin.join(super::claude_binary_name());
    std::fs::write(&home_claude, "").expect("home claude should be written");
    assert_eq!(
        super::select_claude_program(
            Some(temp.path().join("missing").to_string_lossy().to_string()),
            Some(temp.path()),
            Some(std::path::PathBuf::from("/resolved/claude")),
        ),
        home_claude.to_string_lossy().as_ref()
    );

    assert_eq!(
        super::select_claude_program(
            None,
            None,
            Some(std::path::PathBuf::from("/resolved/claude"))
        ),
        "/resolved/claude"
    );
    assert_eq!(super::select_claude_program(None, None, None), super::claude_binary_name());
}

#[test]
fn executor_command_for_opencode_resolved_adds_model_and_prompt_conditionally() {
    let auto = super::ExecutorCommand::for_opencode_resolved(
        "/repo",
        "auto",
        "   ",
        "opencode".to_string(),
        vec!["pkg".to_string()],
    );
    assert_eq!(auto.program, "opencode");
    assert_eq!(auto.cwd, "/repo");
    assert!(auto.stdin_content.is_none());
    assert!(auto.args.contains(&"run".to_string()));
    assert!(auto.args.contains(&"--dir=/repo".to_string()));
    assert!(!auto.args.iter().any(|arg| arg.starts_with("--model=")));
    assert_eq!(auto.args.last().map(String::as_str), Some("--thinking"));

    let custom = super::ExecutorCommand::for_opencode_resolved(
        "/repo",
        "gpt-x",
        "do it",
        "opencode".to_string(),
        Vec::new(),
    );
    assert!(custom.args.contains(&"--model=gpt-x".to_string()));
    assert_eq!(custom.args.last().map(String::as_str), Some("do it"));
}

#[test]
fn executor_command_for_claude_uses_stdin_and_supported_aliases() {
    let cmd = super::ExecutorCommand::for_claude("/repo", "sonnet", "hello");

    assert_eq!(cmd.cwd, "/repo");
    assert_eq!(cmd.stdin_content.as_deref(), Some("hello"));
    assert!(cmd.args.contains(&"--output-format".to_string()));
    assert!(cmd.args.windows(2).any(|pair| pair[0] == "--model" && pair[1] == "sonnet"));
    assert!(cmd.args.windows(2).any(|pair| pair[0] == "--add-dir" && pair[1] == "/repo"));
}

#[test]
fn executor_command_for_claude_defaults_unknown_model_alias() {
    let cmd = super::ExecutorCommand::for_claude("/repo", "unknown-model", "hello");

    assert!(cmd.args.contains(&"--model=default".to_string()));
}

#[test]
fn executor_command_for_codex_skips_auto_model_and_passes_prompt_as_arg() {
    let auto = super::ExecutorCommand::for_codex("/repo", "auto", "prompt");
    assert_eq!(auto.cwd, "/repo");
    assert!(auto.stdin_content.is_none());
    assert!(auto.args.contains(&"exec".to_string()));
    assert!(auto.args.contains(&"--skip-git-repo-check".to_string()));
    assert!(!auto.args.contains(&"--model".to_string()));
    assert_eq!(auto.args.last().map(String::as_str), Some("prompt"));

    let custom = super::ExecutorCommand::for_codex("/repo", "gpt-5", "prompt");
    assert!(custom.args.windows(2).any(|pair| pair[0] == "--model" && pair[1] == "gpt-5"));
}

#[test]
fn build_executor_command_routes_backends() {
    let opencode = super::build_executor_command(
        super::TaskExecutorBackend::OpenCode,
        "/repo",
        "auto",
        "prompt",
    );
    assert!(
        super::is_opencode_program(&opencode.program)
            || opencode.program == "opencode"
            || opencode.args.iter().any(|arg| arg.contains("opencode-ai"))
    );

    let internal = super::build_executor_command(
        super::TaskExecutorBackend::Internal,
        "/repo",
        "auto",
        "prompt",
    );
    assert!(
        super::is_opencode_program(&internal.program)
            || internal.program == "opencode"
            || internal.args.iter().any(|arg| arg.contains("opencode-ai"))
    );

    let claude = super::build_executor_command(
        super::TaskExecutorBackend::Claude,
        "/repo",
        "auto",
        "prompt",
    );
    assert!(super::is_claude_program(&claude.program) || claude.program == "claude");

    let codex =
        super::build_executor_command(super::TaskExecutorBackend::Codex, "/repo", "auto", "prompt");
    assert!(codex.args.contains(&"exec".to_string()));
}

#[test]
fn spawn_executor_child_reports_missing_program() {
    let temp = tempfile::TempDir::new().expect("temp dir should be created");
    let cmd = super::ExecutorCommand {
        program: temp.path().join("missing-program").to_string_lossy().to_string(),
        args: Vec::new(),
        cwd: temp.path().to_string_lossy().to_string(),
        stdin_content: None,
    };

    let error = match super::spawn_executor_child(&cmd) {
        Ok(mut child) => {
            let _ = child.kill();
            panic!("missing program should fail");
        }
        Err(error) => error,
    };

    assert!(error.contains("Failed to spawn"));
}
