//! 提供 vibe-agent 二进制入口。
//! 入口只负责启动共享 CLI 逻辑，保持进程装配与业务处理分离。

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vw_cli::run().await
}

#[cfg(test)]
#[path = "vibe-agent_tests.rs"]
mod vibe_agent_tests;
