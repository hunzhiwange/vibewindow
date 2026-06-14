use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use crate::app::agent::config::{SandboxConfig, SecurityConfig};

    #[test]
    fn detect_best_sandbox_returns_something() {
        let sandbox = detect_best_sandbox();
        // Should always return at least NoopSandbox
        assert!(sandbox.is_available());
    }

    #[test]
    fn explicit_none_returns_noop() {
        let config = SecurityConfig {
            sandbox: SandboxConfig {
                enabled: Some(false),
                backend: SandboxBackend::None,
                firejail_args: Vec::new(),
            },
            ..Default::default()
        };
        let sandbox = create_sandbox(&config);
        assert_eq!(sandbox.name(), "none");
    }

    #[test]
    fn disabled_flag_overrides_specific_backend() {
        let config = SecurityConfig {
            sandbox: SandboxConfig {
                enabled: Some(false),
                backend: SandboxBackend::Docker,
                firejail_args: Vec::new(),
            },
            ..Default::default()
        };

        let sandbox = create_sandbox(&config);
        assert_eq!(sandbox.name(), "none");
        assert!(sandbox.is_available());
    }

    #[test]
    fn auto_mode_detects_something() {
        let config = SecurityConfig {
            sandbox: SandboxConfig {
                enabled: None, // Auto-detect
                backend: SandboxBackend::Auto,
                firejail_args: Vec::new(),
            },
            ..Default::default()
        };
        let sandbox = create_sandbox(&config);
        // Should return some sandbox (at least NoopSandbox)
        assert!(sandbox.is_available());
    }

    #[test]
    fn unavailable_explicit_backends_fall_back_to_noop_or_report_available_backend() {
        for backend in [
            SandboxBackend::Landlock,
            SandboxBackend::Firejail,
            SandboxBackend::Bubblewrap,
            SandboxBackend::Docker,
        ] {
            let config = SecurityConfig {
                sandbox: SandboxConfig { enabled: Some(true), backend, firejail_args: Vec::new() },
                ..Default::default()
            };
            let sandbox = create_sandbox(&config);
            assert!(sandbox.is_available(), "create_sandbox must return a usable sandbox object");
            assert!(
                ["none", "landlock", "firejail", "bubblewrap", "docker"].contains(&sandbox.name())
            );
        }
    }
}
