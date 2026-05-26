//! 提供 vw-cli 主入口。
//! 入口负责进程级初始化并委托 CLI 模块处理具体命令。

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vw_cli::run().await
}
#[cfg(test)]
#[path = "main_tests.rs"]
mod main_tests;
