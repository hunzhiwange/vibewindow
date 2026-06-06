use iced::widget::text_editor;
use vw_gateway_client::{
    KnowledgeChunkDto, KnowledgeDatasetDto, KnowledgeDocumentDto, KnowledgeIndexingMode,
    KnowledgeRetrievalMode, KnowledgeRuntimeStatus,
};

/// 知识库详情页签。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KnowledgeDetailTab {
    #[default]
    Documents,
    Retrieval,
    Settings,
}

impl KnowledgeDetailTab {
    pub(crate) const ALL: [Self; 3] = [Self::Documents, Self::Retrieval, Self::Settings];

    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Documents => "文档",
            Self::Retrieval => "召回测试",
            Self::Settings => "设置",
        }
    }
}

/// 桌面端知识库工作台状态。
#[derive(Debug)]
pub struct KnowledgeUiState {
    pub(crate) datasets: Vec<KnowledgeDatasetDto>,
    pub(crate) documents: Vec<KnowledgeDocumentDto>,
    pub(crate) retrieve_results: Vec<KnowledgeChunkDto>,
    pub(crate) runtime_status: Option<KnowledgeRuntimeStatus>,
    pub(crate) selected_dataset_id: Option<String>,
    pub(crate) active_tab: KnowledgeDetailTab,
    pub(crate) dataset_search_query: String,
    pub(crate) document_search_query: String,
    pub(crate) dataset_name_input: String,
    pub(crate) dataset_description_input: String,
    pub(crate) dataset_indexing_mode: KnowledgeIndexingMode,
    pub(crate) dataset_retrieval_mode: KnowledgeRetrievalMode,
    pub(crate) document_name_input: String,
    pub(crate) document_content_editor: text_editor::Content,
    pub(crate) retrieve_query_input: String,
    pub(crate) retrieve_top_k_input: String,
    pub(crate) loading_status: bool,
    pub(crate) loading_datasets: bool,
    pub(crate) loading_documents: bool,
    pub(crate) creating_dataset: bool,
    pub(crate) creating_document: bool,
    pub(crate) deleting: bool,
    pub(crate) retrieving: bool,
    pub(crate) notification: Option<String>,
    pub(crate) notification_is_error: bool,
}

impl Default for KnowledgeUiState {
    fn default() -> Self {
        Self {
            datasets: Vec::new(),
            documents: Vec::new(),
            retrieve_results: Vec::new(),
            runtime_status: None,
            selected_dataset_id: None,
            active_tab: KnowledgeDetailTab::Documents,
            dataset_search_query: String::new(),
            document_search_query: String::new(),
            dataset_name_input: String::new(),
            dataset_description_input: String::new(),
            dataset_indexing_mode: KnowledgeIndexingMode::Economy,
            dataset_retrieval_mode: KnowledgeRetrievalMode::FullText,
            document_name_input: String::new(),
            document_content_editor: text_editor::Content::new(),
            retrieve_query_input: String::new(),
            retrieve_top_k_input: "10".to_string(),
            loading_status: false,
            loading_datasets: false,
            loading_documents: false,
            creating_dataset: false,
            creating_document: false,
            deleting: false,
            retrieving: false,
            notification: None,
            notification_is_error: false,
        }
    }
}

impl KnowledgeUiState {
    pub(crate) fn selected_dataset(&self) -> Option<&KnowledgeDatasetDto> {
        let selected_id = self.selected_dataset_id.as_deref()?;
        self.datasets.iter().find(|dataset| dataset.id == selected_id)
    }

    pub(crate) fn is_busy(&self) -> bool {
        self.loading_status
            || self.loading_datasets
            || self.loading_documents
            || self.creating_dataset
            || self.creating_document
            || self.deleting
            || self.retrieving
    }

    pub(crate) fn select_first_dataset_if_needed(&mut self) {
        if self.selected_dataset_id.as_ref().is_some_and(|selected_id| {
            self.datasets.iter().any(|dataset| dataset.id == *selected_id)
        }) {
            return;
        }
        self.selected_dataset_id = self.datasets.first().map(|dataset| dataset.id.clone());
    }
}
