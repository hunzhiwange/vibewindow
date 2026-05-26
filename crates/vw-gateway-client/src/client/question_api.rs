use vw_shared::question;

use super::GatewayClient;

impl GatewayClient {
    /// 列出当前待处理的问题请求。
    pub async fn question_list(&self) -> Result<Vec<question::Request>, String> {
        self.get_json("/v1/question", &[]).await
    }

    /// 回答指定问题请求，并提交多组选项答案。
    pub async fn question_reply(
        &self,
        request_id: &str,
        answers: Vec<Vec<String>>,
    ) -> Result<bool, String> {
        self.post_json(
            &format!("/v1/question/{request_id}/reply"),
            &[],
            &serde_json::json!({ "answers": answers }),
        )
        .await
    }

    /// 拒绝指定问题请求。
    pub async fn question_reject(&self, request_id: &str) -> Result<bool, String> {
        self.post_json(&format!("/v1/question/{request_id}/reject"), &[], &serde_json::json!({}))
            .await
    }
}

#[cfg(test)]
#[path = "question_api_tests.rs"]
mod question_api_tests;
