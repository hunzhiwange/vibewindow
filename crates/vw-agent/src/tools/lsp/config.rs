//! LSP server 配置和命令解析。
//!
//! 本模块根据文件扩展名选择语言服务器，并按“环境变量优先、PATH 查找兜底”的
//! 顺序解析可执行命令。它不启动进程，只返回后端可消费的静态配置。

use std::path::Path;

use which::which;

/// 某个语言服务器的可执行命令候选。
///
/// `env_vars` 允许用户用绝对路径或自定义命令覆盖默认 `program`。解析失败时会继续
/// 尝试同一 server 的下一个候选。
#[derive(Clone, Copy)]
pub(crate) struct ServerCommandCandidate {
    pub program: &'static str,
    pub args: &'static [&'static str],
    pub env_vars: &'static [&'static str],
}

/// 文件扩展名到 LSP server 的静态映射。
///
/// `server_key` 用于缓存和展示，`language_id` 用于 `didOpen` 文档同步。
#[derive(Clone, Copy)]
pub(crate) struct ServerConfig {
    pub server_key: &'static str,
    pub language_id: &'static str,
    pub extensions: &'static [&'static str],
    pub commands: &'static [ServerCommandCandidate],
}

/// 已解析的语言服务器启动命令。
///
/// `program` 是最终可执行路径或环境变量提供的命令，`args` 是固定启动参数。
#[derive(Debug, Clone)]
pub(crate) struct ResolvedServerCommand {
    pub program: String,
    pub args: Vec<String>,
}

const RUST_COMMANDS: &[ServerCommandCandidate] = &[ServerCommandCandidate {
    program: "rust-analyzer",
    args: &[],
    env_vars: &["RUST_ANALYZER", "RUST_ANALYZER_PATH"],
}];

const TS_COMMANDS: &[ServerCommandCandidate] = &[ServerCommandCandidate {
    program: "typescript-language-server",
    args: &["--stdio"],
    env_vars: &["TYPESCRIPT_LANGUAGE_SERVER", "TYPESCRIPT_LANGUAGE_SERVER_PATH"],
}];

const PYTHON_COMMANDS: &[ServerCommandCandidate] = &[
    ServerCommandCandidate {
        program: "pyright-langserver",
        args: &["--stdio"],
        env_vars: &["PYRIGHT_LANGSERVER", "PYRIGHT_LANGSERVER_PATH"],
    },
    ServerCommandCandidate { program: "pylsp", args: &[], env_vars: &["PYLSP", "PYLSP_PATH"] },
];

const GO_COMMANDS: &[ServerCommandCandidate] =
    &[ServerCommandCandidate { program: "gopls", args: &[], env_vars: &["GOPLS", "GOPLS_PATH"] }];

const CLANGD_COMMANDS: &[ServerCommandCandidate] = &[ServerCommandCandidate {
    program: "clangd",
    args: &[],
    env_vars: &["CLANGD", "CLANGD_PATH"],
}];

const JSON_COMMANDS: &[ServerCommandCandidate] = &[ServerCommandCandidate {
    program: "vscode-json-language-server",
    args: &["--stdio"],
    env_vars: &["VSCODE_JSON_LANGUAGE_SERVER", "VSCODE_JSON_LANGUAGE_SERVER_PATH"],
}];

const YAML_COMMANDS: &[ServerCommandCandidate] = &[ServerCommandCandidate {
    program: "yaml-language-server",
    args: &["--stdio"],
    env_vars: &["YAML_LANGUAGE_SERVER", "YAML_LANGUAGE_SERVER_PATH"],
}];

const TOML_COMMANDS: &[ServerCommandCandidate] = &[ServerCommandCandidate {
    program: "taplo",
    args: &["lsp", "stdio"],
    env_vars: &["TAPLO", "TAPLO_PATH"],
}];

const LUA_COMMANDS: &[ServerCommandCandidate] = &[ServerCommandCandidate {
    program: "lua-language-server",
    args: &[],
    env_vars: &["LUA_LANGUAGE_SERVER", "LUA_LANGUAGE_SERVER_PATH"],
}];

const SERVER_CONFIGS: &[ServerConfig] = &[
    ServerConfig {
        server_key: "rust-analyzer",
        language_id: "rust",
        extensions: &["rs"],
        commands: RUST_COMMANDS,
    },
    ServerConfig {
        server_key: "typescript-language-server",
        language_id: "typescript",
        extensions: &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"],
        commands: TS_COMMANDS,
    },
    ServerConfig {
        server_key: "pyright-langserver",
        language_id: "python",
        extensions: &["py"],
        commands: PYTHON_COMMANDS,
    },
    ServerConfig {
        server_key: "gopls",
        language_id: "go",
        extensions: &["go"],
        commands: GO_COMMANDS,
    },
    ServerConfig {
        server_key: "clangd",
        language_id: "cpp",
        extensions: &["c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx"],
        commands: CLANGD_COMMANDS,
    },
    ServerConfig {
        server_key: "vscode-json-language-server",
        language_id: "json",
        extensions: &["json", "jsonc"],
        commands: JSON_COMMANDS,
    },
    ServerConfig {
        server_key: "yaml-language-server",
        language_id: "yaml",
        extensions: &["yaml", "yml"],
        commands: YAML_COMMANDS,
    },
    ServerConfig {
        server_key: "taplo",
        language_id: "toml",
        extensions: &["toml"],
        commands: TOML_COMMANDS,
    },
    ServerConfig {
        server_key: "lua-language-server",
        language_id: "lua",
        extensions: &["lua"],
        commands: LUA_COMMANDS,
    },
];

/// 根据文件路径查找匹配的 LSP server 配置。
///
/// 返回 `None` 表示扩展名未知或不可转换为 UTF-8；该函数不检查 server 是否安装。
pub(crate) fn server_config_for_path(path: &Path) -> Option<&'static ServerConfig> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    SERVER_CONFIGS
        .iter()
        .find(|config| config.extensions.iter().any(|candidate| *candidate == extension))
}

/// 解析给定 server 配置的可执行命令。
///
/// 先读取候选环境变量，取第一个非空值；若未配置，再从 `PATH` 查找默认程序。
/// 返回 `None` 表示当前机器没有可用命令。
pub(crate) fn resolve_command(config: &ServerConfig) -> Option<ResolvedServerCommand> {
    for candidate in config.commands {
        for env_key in candidate.env_vars {
            let value = std::env::var(env_key).ok().map(|value| value.trim().to_string());
            let Some(program) = value.filter(|value| !value.is_empty()) else {
                continue;
            };
            return Some(ResolvedServerCommand {
                program,
                args: candidate.args.iter().map(|arg| (*arg).to_string()).collect(),
            });
        }

        let Ok(path) = which(candidate.program) else {
            continue;
        };
        return Some(ResolvedServerCommand {
            program: path.to_string_lossy().to_string(),
            args: candidate.args.iter().map(|arg| (*arg).to_string()).collect(),
        });
    }

    None
}
#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
