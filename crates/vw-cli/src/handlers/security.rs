//! 处理 CLI 安全相关命令。
//! 模块把安全策略展示与用户确认流程限制在 CLI 边界内。

use anyhow::Result;

use crate::cli::SecurityCommands;
use crate::config::Config;
use vw_agent::security;

/// 执行 handle_security_command 对应的模块功能。
///
/// 参数含义保持与函数签名一致；返回值用于调用方继续组合处理，错误由返回类型显式表达。
pub(crate) async fn handle_security_command(
    config: &Config,
    security_command: SecurityCommands,
) -> Result<()> {
    match security_command {
        SecurityCommands::UpdateGuardCorpus { source, checksum } => {
            let report = security::semantic_guard::update_guard_corpus(
                config,
                source.as_deref(),
                checksum.as_deref(),
            )
            .await?;

            println!("Semantic guard corpus update completed.");
            println!("  Source:           {}", report.source);
            println!("  SHA-256:          {}", report.sha256);
            println!("  Parsed records:   {}", report.parsed_records);
            println!("  Upserted records: {}", report.upserted_records);
            println!("  Collection:       {}", report.collection);
            Ok(())
        }
    }
}
#[cfg(test)]
#[path = "security_tests.rs"]
mod security_tests;
