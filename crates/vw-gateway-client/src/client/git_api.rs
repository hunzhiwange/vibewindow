use vw_api_types::git::{
    GitCommandRequest, GitCommandResponse, GitCommitRequest, GitCommitResponse, GitMergeRequest,
    GitMergeResponse,
};

use super::GatewayClient;

impl GatewayClient {
    /// 通过网关触发一次 Git 提交。
    pub async fn git_commit(
        &self,
        request: &GitCommitRequest,
    ) -> Result<GitCommitResponse, String> {
        self.post_json("/v1/git/commit", &[], request).await
    }

    /// 通过网关执行一次受限 Git 命令。
    pub async fn git_command(
        &self,
        request: &GitCommandRequest,
    ) -> Result<GitCommandResponse, String> {
        self.post_json("/v1/git/command", &[], request).await
    }

    /// 通过网关触发一次 Git 分支合并。
    pub async fn git_merge(&self, request: &GitMergeRequest) -> Result<GitMergeResponse, String> {
        self.post_json("/v1/git/merge", &[], request).await
    }
}

#[cfg(test)]
#[path = "git_api_tests.rs"]
mod git_api_tests;
