use super::{indexing_label, retrieval_label, support_label};
use vw_gateway_client::{KnowledgeIndexingMode, KnowledgeRetrievalMode};

#[test]
fn knowledge_labels_match_ui_copy() {
    assert_eq!(indexing_label(&KnowledgeIndexingMode::Economy), "经济");
    assert_eq!(indexing_label(&KnowledgeIndexingMode::HighQuality), "高质量");
    assert_eq!(retrieval_label(&KnowledgeRetrievalMode::Hybrid), "混合检索");
    assert_eq!(support_label(true), "可用");
    assert_eq!(support_label(false), "不可用");
}
