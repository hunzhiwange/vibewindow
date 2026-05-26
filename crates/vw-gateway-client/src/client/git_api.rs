use vw_api_types::git::{GitCommitRequest, GitCommitResponse};

use super::GatewayClient;

impl GatewayClient {
    /// 通过网关触发一次 Git 提交。
    pub async fn git_commit(
        &self,
        request: &GitCommitRequest,
    ) -> Result<GitCommitResponse, String> {
        self.post_json("/v1/git/commit", &[], request).await
    }
}

#[cfg(test)]
#[path = "git_api_tests.rs"]
mod git_api_tests;
