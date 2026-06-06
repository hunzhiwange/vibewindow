//! 知识库工作台消息处理。

use crate::app::state::KnowledgeDetailTab;
use crate::app::{App, Message};
use iced::Task;
use iced::widget::text_editor;
use serde_json::Value;
use vw_gateway_client::{
    KnowledgeDatasetCreateRequest, KnowledgeDatasetDto, KnowledgeDocumentCreateRequest,
    KnowledgeDocumentDto, KnowledgeIndexingMode, KnowledgeRetrievalMode, KnowledgeRetrieveRequest,
    KnowledgeRetrieveResponse, KnowledgeRuntimeStatus,
};

#[derive(Debug, Clone)]
pub enum KnowledgeToolMessage {
    Refresh,
    StatusLoaded(Result<KnowledgeRuntimeStatus, String>),
    DatasetsLoaded(Result<Vec<KnowledgeDatasetDto>, String>),
    DocumentsLoaded(String, Result<Vec<KnowledgeDocumentDto>, String>),
    SelectDataset(String),
    SelectTab(KnowledgeDetailTab),
    DatasetSearchChanged(String),
    DocumentSearchChanged(String),
    DatasetNameChanged(String),
    DatasetDescriptionChanged(String),
    DatasetIndexingModeChanged(KnowledgeIndexingMode),
    DatasetRetrievalModeChanged(KnowledgeRetrievalMode),
    CreateDataset,
    DatasetCreated(Result<KnowledgeDatasetDto, String>),
    DeleteSelectedDataset,
    DatasetDeleted(Result<KnowledgeDatasetDto, String>),
    DocumentNameChanged(String),
    DocumentContentAction(text_editor::Action),
    CreateDocument,
    DocumentCreated(String, Result<KnowledgeDocumentDto, String>),
    DeleteDocument(String),
    DocumentDeleted(Result<KnowledgeDocumentDto, String>),
    RetrieveQueryChanged(String),
    RetrieveTopKChanged(String),
    Retrieve,
    RetrieveFinished(Result<KnowledgeRetrieveResponse, String>),
    ClearDocumentDraft,
    ClearNotification,
}

pub fn update(app: &mut App, message: KnowledgeToolMessage) -> Task<Message> {
    match message {
        KnowledgeToolMessage::Refresh => {
            app.knowledge.loading_status = true;
            app.knowledge.loading_datasets = true;
            Task::batch([load_status_task(), load_datasets_task()])
        }
        KnowledgeToolMessage::StatusLoaded(result) => {
            app.knowledge.loading_status = false;
            match result {
                Ok(status) => app.knowledge.runtime_status = Some(status),
                Err(error) => set_error(app, error),
            }
            Task::none()
        }
        KnowledgeToolMessage::DatasetsLoaded(result) => {
            app.knowledge.loading_datasets = false;
            match result {
                Ok(datasets) => {
                    app.knowledge.datasets = datasets;
                    app.knowledge.select_first_dataset_if_needed();
                    if let Some(dataset_id) = app.knowledge.selected_dataset_id.clone() {
                        app.knowledge.loading_documents = true;
                        load_documents_task(dataset_id)
                    } else {
                        app.knowledge.documents.clear();
                        Task::none()
                    }
                }
                Err(error) => {
                    set_error(app, error);
                    Task::none()
                }
            }
        }
        KnowledgeToolMessage::DocumentsLoaded(dataset_id, result) => {
            app.knowledge.loading_documents = false;
            if app.knowledge.selected_dataset_id.as_deref() != Some(dataset_id.as_str()) {
                return Task::none();
            }
            match result {
                Ok(documents) => app.knowledge.documents = documents,
                Err(error) => set_error(app, error),
            }
            Task::none()
        }
        KnowledgeToolMessage::SelectDataset(dataset_id) => {
            app.knowledge.selected_dataset_id = Some(dataset_id.clone());
            app.knowledge.documents.clear();
            app.knowledge.retrieve_results.clear();
            app.knowledge.loading_documents = true;
            load_documents_task(dataset_id)
        }
        KnowledgeToolMessage::SelectTab(tab) => {
            app.knowledge.active_tab = tab;
            Task::none()
        }
        KnowledgeToolMessage::DatasetSearchChanged(value) => {
            app.knowledge.dataset_search_query = value;
            Task::none()
        }
        KnowledgeToolMessage::DocumentSearchChanged(value) => {
            app.knowledge.document_search_query = value;
            Task::none()
        }
        KnowledgeToolMessage::DatasetNameChanged(value) => {
            app.knowledge.dataset_name_input = value;
            Task::none()
        }
        KnowledgeToolMessage::DatasetDescriptionChanged(value) => {
            app.knowledge.dataset_description_input = value;
            Task::none()
        }
        KnowledgeToolMessage::DatasetIndexingModeChanged(mode) => {
            app.knowledge.dataset_indexing_mode = mode;
            Task::none()
        }
        KnowledgeToolMessage::DatasetRetrievalModeChanged(mode) => {
            if mode == KnowledgeRetrievalMode::FullText {
                app.knowledge.dataset_indexing_mode = KnowledgeIndexingMode::Economy;
            }
            app.knowledge.dataset_retrieval_mode = mode;
            Task::none()
        }
        KnowledgeToolMessage::CreateDataset => create_dataset(app),
        KnowledgeToolMessage::DatasetCreated(result) => {
            app.knowledge.creating_dataset = false;
            match result {
                Ok(dataset) => {
                    app.knowledge.selected_dataset_id = Some(dataset.id.clone());
                    app.knowledge.dataset_name_input.clear();
                    app.knowledge.dataset_description_input.clear();
                    app.knowledge.datasets.insert(0, dataset);
                    set_success(app, "知识库已创建");
                    Task::batch([load_datasets_task(), load_status_task()])
                }
                Err(error) => {
                    set_error(app, error);
                    Task::none()
                }
            }
        }
        KnowledgeToolMessage::DeleteSelectedDataset => delete_selected_dataset(app),
        KnowledgeToolMessage::DatasetDeleted(result) => {
            app.knowledge.deleting = false;
            match result {
                Ok(dataset) => {
                    app.knowledge.datasets.retain(|item| item.id != dataset.id);
                    app.knowledge.select_first_dataset_if_needed();
                    app.knowledge.documents.clear();
                    app.knowledge.retrieve_results.clear();
                    set_success(app, "知识库已删除");
                    if let Some(dataset_id) = app.knowledge.selected_dataset_id.clone() {
                        app.knowledge.loading_documents = true;
                        load_documents_task(dataset_id)
                    } else {
                        Task::none()
                    }
                }
                Err(error) => {
                    set_error(app, error);
                    Task::none()
                }
            }
        }
        KnowledgeToolMessage::DocumentNameChanged(value) => {
            app.knowledge.document_name_input = value;
            Task::none()
        }
        KnowledgeToolMessage::DocumentContentAction(action) => {
            app.knowledge.document_content_editor.perform(action);
            Task::none()
        }
        KnowledgeToolMessage::CreateDocument => create_document(app),
        KnowledgeToolMessage::DocumentCreated(dataset_id, result) => {
            app.knowledge.creating_document = false;
            match result {
                Ok(_document) => {
                    app.knowledge.document_name_input.clear();
                    app.knowledge.document_content_editor = text_editor::Content::new();
                    set_success(app, "文档已添加并入库");
                    app.knowledge.loading_documents = true;
                    load_documents_task(dataset_id)
                }
                Err(error) => {
                    set_error(app, error);
                    Task::none()
                }
            }
        }
        KnowledgeToolMessage::DeleteDocument(document_id) => {
            if app.knowledge.deleting {
                return Task::none();
            }
            app.knowledge.deleting = true;
            Task::perform(delete_document_async(document_id), |result| {
                Message::Knowledge(KnowledgeToolMessage::DocumentDeleted(result))
            })
        }
        KnowledgeToolMessage::DocumentDeleted(result) => {
            app.knowledge.deleting = false;
            match result {
                Ok(document) => {
                    app.knowledge.documents.retain(|item| item.id != document.id);
                    set_success(app, "文档已删除");
                    Task::none()
                }
                Err(error) => {
                    set_error(app, error);
                    Task::none()
                }
            }
        }
        KnowledgeToolMessage::RetrieveQueryChanged(value) => {
            app.knowledge.retrieve_query_input = value;
            Task::none()
        }
        KnowledgeToolMessage::RetrieveTopKChanged(value) => {
            app.knowledge.retrieve_top_k_input = value;
            Task::none()
        }
        KnowledgeToolMessage::Retrieve => retrieve(app),
        KnowledgeToolMessage::RetrieveFinished(result) => {
            app.knowledge.retrieving = false;
            match result {
                Ok(response) => {
                    app.knowledge.retrieve_results = response.chunks;
                    set_success(app, "召回测试完成");
                }
                Err(error) => set_error(app, error),
            }
            Task::none()
        }
        KnowledgeToolMessage::ClearDocumentDraft => {
            app.knowledge.document_name_input.clear();
            app.knowledge.document_content_editor = text_editor::Content::new();
            Task::none()
        }
        KnowledgeToolMessage::ClearNotification => {
            app.knowledge.notification = None;
            app.knowledge.notification_is_error = false;
            Task::none()
        }
    }
}

pub(crate) fn ensure_loaded(app: &mut App) -> Task<Message> {
    if app.knowledge.datasets.is_empty() && !app.knowledge.loading_datasets {
        app.knowledge.loading_status = true;
        app.knowledge.loading_datasets = true;
        Task::batch([load_status_task(), load_datasets_task()])
    } else {
        Task::none()
    }
}

fn create_dataset(app: &mut App) -> Task<Message> {
    if app.knowledge.creating_dataset {
        return Task::none();
    }
    let name = app.knowledge.dataset_name_input.trim().to_string();
    if name.is_empty() {
        set_error(app, "知识库名称不能为空");
        return Task::none();
    }
    app.knowledge.creating_dataset = true;
    let body = KnowledgeDatasetCreateRequest {
        name,
        description: app.knowledge.dataset_description_input.trim().to_string(),
        indexing_mode: app.knowledge.dataset_indexing_mode.clone(),
        retrieval_mode: app.knowledge.dataset_retrieval_mode.clone(),
        embedding_model: None,
        rerank_model: None,
    };
    Task::perform(create_dataset_async(body), |result| {
        Message::Knowledge(KnowledgeToolMessage::DatasetCreated(result))
    })
}

fn delete_selected_dataset(app: &mut App) -> Task<Message> {
    if app.knowledge.deleting {
        return Task::none();
    }
    let Some(dataset_id) = app.knowledge.selected_dataset_id.clone() else {
        set_error(app, "请先选择知识库");
        return Task::none();
    };
    app.knowledge.deleting = true;
    Task::perform(delete_dataset_async(dataset_id), |result| {
        Message::Knowledge(KnowledgeToolMessage::DatasetDeleted(result))
    })
}

fn create_document(app: &mut App) -> Task<Message> {
    if app.knowledge.creating_document {
        return Task::none();
    }
    let Some(dataset_id) = app.knowledge.selected_dataset_id.clone() else {
        set_error(app, "请先选择知识库");
        return Task::none();
    };
    let name = app.knowledge.document_name_input.trim().to_string();
    if name.is_empty() {
        set_error(app, "文档名称不能为空");
        return Task::none();
    }
    let content = app.knowledge.document_content_editor.text();
    if content.trim().is_empty() {
        set_error(app, "文档内容不能为空");
        return Task::none();
    }
    app.knowledge.creating_document = true;
    let body =
        KnowledgeDocumentCreateRequest { name, content, metadata: Value::Null, enabled: true };
    let task_dataset_id = dataset_id.clone();
    Task::perform(create_document_async(dataset_id, body), move |result| {
        Message::Knowledge(KnowledgeToolMessage::DocumentCreated(task_dataset_id.clone(), result))
    })
}

fn retrieve(app: &mut App) -> Task<Message> {
    if app.knowledge.retrieving {
        return Task::none();
    }
    let Some(dataset_id) = app.knowledge.selected_dataset_id.clone() else {
        set_error(app, "请先选择知识库");
        return Task::none();
    };
    let query = app.knowledge.retrieve_query_input.trim().to_string();
    if query.is_empty() {
        set_error(app, "请输入召回测试文本");
        return Task::none();
    }
    let top_k =
        app.knowledge.retrieve_top_k_input.trim().parse::<usize>().unwrap_or(10).clamp(1, 50);
    app.knowledge.retrieving = true;
    let body = KnowledgeRetrieveRequest {
        query,
        dataset_ids: vec![dataset_id],
        top_k,
        score_threshold: None,
        metadata_filter: None,
    };
    Task::perform(retrieve_async(body), |result| {
        Message::Knowledge(KnowledgeToolMessage::RetrieveFinished(result))
    })
}

fn load_status_task() -> Task<Message> {
    Task::perform(status_async(), |result| {
        Message::Knowledge(KnowledgeToolMessage::StatusLoaded(result))
    })
}

fn load_datasets_task() -> Task<Message> {
    Task::perform(list_datasets_async(), |result| {
        Message::Knowledge(KnowledgeToolMessage::DatasetsLoaded(result))
    })
}

fn load_documents_task(dataset_id: String) -> Task<Message> {
    Task::perform(list_documents_async(dataset_id.clone()), move |result| {
        Message::Knowledge(KnowledgeToolMessage::DocumentsLoaded(dataset_id.clone(), result))
    })
}

async fn status_async() -> Result<KnowledgeRuntimeStatus, String> {
    crate::app::gateway_client()?.knowledge_status().await
}

async fn list_datasets_async() -> Result<Vec<KnowledgeDatasetDto>, String> {
    crate::app::gateway_client()?.knowledge_datasets_list().await
}

async fn list_documents_async(dataset_id: String) -> Result<Vec<KnowledgeDocumentDto>, String> {
    crate::app::gateway_client()?.knowledge_documents_list(&dataset_id).await
}

async fn create_dataset_async(
    body: KnowledgeDatasetCreateRequest,
) -> Result<KnowledgeDatasetDto, String> {
    crate::app::gateway_client()?.knowledge_dataset_create(&body).await
}

async fn delete_dataset_async(dataset_id: String) -> Result<KnowledgeDatasetDto, String> {
    crate::app::gateway_client()?.knowledge_dataset_delete(&dataset_id).await
}

async fn create_document_async(
    dataset_id: String,
    body: KnowledgeDocumentCreateRequest,
) -> Result<KnowledgeDocumentDto, String> {
    crate::app::gateway_client()?.knowledge_document_create(&dataset_id, &body).await
}

async fn delete_document_async(document_id: String) -> Result<KnowledgeDocumentDto, String> {
    crate::app::gateway_client()?.knowledge_document_delete(&document_id).await
}

async fn retrieve_async(
    body: KnowledgeRetrieveRequest,
) -> Result<KnowledgeRetrieveResponse, String> {
    crate::app::gateway_client()?.knowledge_retrieve(&body).await
}

fn set_success(app: &mut App, message: impl Into<String>) {
    app.knowledge.notification = Some(message.into());
    app.knowledge.notification_is_error = false;
}

fn set_error(app: &mut App, message: impl Into<String>) {
    app.knowledge.notification = Some(message.into());
    app.knowledge.notification_is_error = true;
}
