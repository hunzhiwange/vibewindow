//! 知识库工作台视图。

use crate::app::assets::Icon;
use crate::app::components::system_settings_common::{
    danger_action_btn_style, icon_svg, primary_action_btn_style, rounded_action_btn_style,
    settings_checkbox_style, settings_error_banner, settings_muted_text_style, settings_panel,
    settings_panel_style, settings_pick_list_menu_style, settings_pick_list_style,
    settings_success_banner, settings_text_editor_style, settings_text_input_style,
    settings_value_badge,
};
use crate::app::message::KnowledgeToolMessage;
use crate::app::state::{EmbeddingRouteDraft, KnowledgeDetailTab};
use crate::app::{App, Message};
use iced::widget::{
    button, checkbox, column, container, pick_list, responsive, row, scrollable, text, text_editor,
    text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};
use std::fmt;
use vw_gateway_client::{
    KnowledgeChunkDto, KnowledgeChunkingMode, KnowledgeDatasetDto, KnowledgeDocumentDto,
    KnowledgeIndexingMode, KnowledgeRetrievalMode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct KnowledgeEmbeddingModelOption {
    value: String,
    label: String,
}

impl fmt::Display for KnowledgeEmbeddingModelOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    let hero = build_hero(app);
    let workspace = responsive(move |size| build_workspace(app, size));

    let mut content = column![hero].spacing(14).width(Length::Fill).height(Length::Fill);
    if let Some(notification) = &app.knowledge.notification {
        let banner = if app.knowledge.notification_is_error {
            settings_error_banner(notification)
        } else {
            settings_success_banner(notification)
        };
        content = content.push(banner);
    }
    content = content.push(container(workspace).width(Length::Fill).height(Length::Fill));

    container(content.padding([18, 24]))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(palette.background.base.color.into()),
                ..Default::default()
            }
        })
        .into()
}

fn build_hero(app: &App) -> Element<'_, Message> {
    let doc_count: u64 = app.knowledge.datasets.iter().map(|dataset| dataset.document_count).sum();
    let chunk_count: u64 = app.knowledge.datasets.iter().map(|dataset| dataset.chunk_count).sum();
    let vector_label = app
        .knowledge
        .runtime_status
        .as_ref()
        .map(|status| if status.vector { "向量可用" } else { "仅全文" })
        .unwrap_or("检测中");
    let refresh_enabled = !app.knowledge.is_busy();

    container(
        column![
            row![
                column![
                    row![icon_svg(Icon::Journals, 20.0), text("知识库").size(20),]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    text("管理知识库、文档入库与召回测试。")
                        .size(12)
                        .style(settings_muted_text_style),
                ]
                .spacing(4)
                .width(Length::Fill),
                button(row![icon_svg(Icon::ArrowRepeat, 14.0), text("刷新").size(13)].spacing(8))
                    .on_press_maybe(
                        refresh_enabled
                            .then_some(Message::Knowledge(KnowledgeToolMessage::Refresh))
                    )
                    .padding([9, 12])
                    .style(knowledge_toolbar_button_style),
                button(row![icon_svg(Icon::X, 13.0), text("清除提示").size(13)].spacing(8))
                    .on_press(Message::Knowledge(KnowledgeToolMessage::ClearNotification))
                    .padding([9, 12])
                    .style(knowledge_toolbar_button_style),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                settings_value_badge(format!("{} 个知识库", app.knowledge.datasets.len())),
                settings_value_badge(format!("{doc_count} 个文档")),
                settings_value_badge(format!("{chunk_count} 个分段")),
                settings_value_badge(vector_label),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(12),
    )
    .padding([18, 20])
    .width(Length::Fill)
    .style(settings_panel_style)
    .into()
}

fn build_workspace<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let sidebar = build_sidebar(app);
    let detail = build_detail(app, size);

    if size.width >= 1000.0 {
        row![
            container(sidebar).width(Length::Fixed(340.0)).height(Length::Fill),
            container(detail).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        column![
            container(sidebar)
                .width(Length::Fill)
                .height(Length::Fixed((size.height * 0.36).clamp(210.0, 340.0))),
            container(detail).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn build_sidebar(app: &App) -> Element<'_, Message> {
    let create_panel = settings_panel(
        column![
            text("创建知识库").size(14),
            text("配置分段、索引和检索参数。").size(12).style(settings_muted_text_style),
            text_input("名称", &app.knowledge.dataset_name_input)
                .on_input(|value| Message::Knowledge(KnowledgeToolMessage::DatasetNameChanged(
                    value
                )))
                .padding([9, 10])
                .size(13)
                .style(settings_text_input_style),
            text_input("描述", &app.knowledge.dataset_description_input)
                .on_input(|value| {
                    Message::Knowledge(KnowledgeToolMessage::DatasetDescriptionChanged(value))
                })
                .padding([9, 10])
                .size(13)
                .style(settings_text_input_style),
            build_chunking_mode_section(app),
            build_indexing_mode_section(app),
            build_embedding_model_section(app),
            build_retrieval_mode_section(app),
            build_dataset_param_section(app),
            button(
                container(
                    row![icon_svg(Icon::Plus, 14.0), text("创建").size(13)]
                        .spacing(8)
                        .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .center_x(Length::Fill),
            )
            .on_press_maybe(
                (!app.knowledge.creating_dataset)
                    .then_some(Message::Knowledge(KnowledgeToolMessage::CreateDataset))
            )
            .padding([10, 14])
            .width(Length::Fill)
            .style(primary_action_btn_style),
        ]
        .spacing(10),
    );

    let search = text_input("搜索知识库", &app.knowledge.dataset_search_query)
        .on_input(|value| Message::Knowledge(KnowledgeToolMessage::DatasetSearchChanged(value)))
        .padding([9, 10])
        .size(13)
        .style(settings_text_input_style);

    let query = app.knowledge.dataset_search_query.trim().to_lowercase();
    let mut list = column![].spacing(10).width(Length::Fill);
    let mut matched = 0usize;
    for dataset in app.knowledge.datasets.iter().filter(|dataset| {
        query.is_empty()
            || dataset.name.to_lowercase().contains(&query)
            || dataset.description.to_lowercase().contains(&query)
    }) {
        matched += 1;
        list = list.push(dataset_item(app, dataset));
    }

    if app.knowledge.loading_datasets {
        list = list.push(empty_hint("正在加载知识库"));
    } else if app.knowledge.datasets.is_empty() {
        list = list.push(empty_hint("暂无知识库"));
    } else if matched == 0 {
        list = list.push(empty_hint("没有匹配结果"));
    }

    column![
        create_panel,
        search,
        container(scrollable(list).height(Length::Fill)).height(Length::Fill),
    ]
    .spacing(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_chunking_mode_section(app: &App) -> Element<'_, Message> {
    column![
        section_header("分段模式", "选择文档入库结构。"),
        option_card(
            "General",
            "通用文本分块。",
            Icon::ListUl,
            app.knowledge.dataset_chunking_mode == KnowledgeChunkingMode::General,
            true,
            false,
            Message::Knowledge(KnowledgeToolMessage::DatasetChunkingModeChanged(
                KnowledgeChunkingMode::General,
            )),
        ),
        option_card(
            "Parent-Child",
            "子块检索，父块返回。",
            Icon::Columns,
            app.knowledge.dataset_chunking_mode == KnowledgeChunkingMode::ParentChild,
            true,
            false,
            Message::Knowledge(KnowledgeToolMessage::DatasetChunkingModeChanged(
                KnowledgeChunkingMode::ParentChild,
            )),
        ),
        option_card(
            "Q&A",
            "问题检索，答案返回。",
            Icon::ChatTextFill,
            app.knowledge.dataset_chunking_mode == KnowledgeChunkingMode::Qa,
            true,
            false,
            Message::Knowledge(KnowledgeToolMessage::DatasetChunkingModeChanged(
                KnowledgeChunkingMode::Qa,
            )),
        ),
    ]
    .spacing(8)
    .into()
}

fn build_indexing_mode_section(app: &App) -> Element<'_, Message> {
    let vector_available =
        app.knowledge.runtime_status.as_ref().is_some_and(|status| status.vector);
    column![
        section_header("索引模式", "高质量需要嵌入能力。"),
        option_card(
            "高质量",
            "生成向量索引。",
            Icon::Speedometer2,
            app.knowledge.dataset_indexing_mode == KnowledgeIndexingMode::HighQuality,
            vector_available,
            true,
            Message::Knowledge(KnowledgeToolMessage::DatasetIndexingModeChanged(
                KnowledgeIndexingMode::HighQuality,
            )),
        ),
        option_card(
            "经济",
            "关键词检索。",
            Icon::HddNetwork,
            app.knowledge.dataset_indexing_mode == KnowledgeIndexingMode::Economy,
            true,
            false,
            Message::Knowledge(KnowledgeToolMessage::DatasetIndexingModeChanged(
                KnowledgeIndexingMode::Economy,
            )),
        ),
    ]
    .spacing(8)
    .into()
}

fn build_embedding_model_section(app: &App) -> Element<'_, Message> {
    let options = knowledge_embedding_model_options(app);
    let selected = knowledge_embedding_model_selected(app);
    let picker = pick_list(options, Some(selected), |option| {
        Message::Knowledge(KnowledgeToolMessage::DatasetEmbeddingModelChanged(option.value))
    })
    .padding([9, 10])
    .text_size(13)
    .style(settings_pick_list_style)
    .menu_style(settings_pick_list_menu_style)
    .width(Length::Fill);

    column![section_header("Embedding 模型", "共用记忆配置里的向量化。"), picker,].spacing(8).into()
}

fn build_retrieval_mode_section(app: &App) -> Element<'_, Message> {
    let vector_available =
        app.knowledge.runtime_status.as_ref().is_some_and(|status| status.vector);
    column![
        section_header("检索设置", "选择召回策略。"),
        option_card(
            "向量检索",
            "语义相似召回。",
            Icon::Grid1x2,
            app.knowledge.dataset_retrieval_mode == KnowledgeRetrievalMode::Vector,
            vector_available,
            false,
            Message::Knowledge(KnowledgeToolMessage::DatasetRetrievalModeChanged(
                KnowledgeRetrievalMode::Vector,
            )),
        ),
        option_card(
            "全文检索",
            "按词汇召回。",
            Icon::FileText,
            app.knowledge.dataset_retrieval_mode == KnowledgeRetrievalMode::FullText,
            true,
            false,
            Message::Knowledge(KnowledgeToolMessage::DatasetRetrievalModeChanged(
                KnowledgeRetrievalMode::FullText,
            )),
        ),
        option_card(
            "混合检索",
            "全文和向量合并。",
            Icon::Grid1x2,
            app.knowledge.dataset_retrieval_mode == KnowledgeRetrievalMode::Hybrid,
            vector_available,
            true,
            Message::Knowledge(KnowledgeToolMessage::DatasetRetrievalModeChanged(
                KnowledgeRetrievalMode::Hybrid,
            )),
        ),
    ]
    .spacing(8)
    .into()
}

fn build_dataset_param_section(app: &App) -> Element<'_, Message> {
    let mut content = column![section_header("参数", "默认召回参数。")].spacing(8);
    if app.knowledge.dataset_indexing_mode == KnowledgeIndexingMode::Economy {
        content = content.push(compact_input_row(
            "关键词数量",
            &app.knowledge.dataset_keyword_count_input,
            |value| Message::Knowledge(KnowledgeToolMessage::DatasetKeywordCountChanged(value)),
        ));
    }
    content =
        content.push(compact_input_row("Top K", &app.knowledge.dataset_top_k_input, |value| {
            Message::Knowledge(KnowledgeToolMessage::DatasetTopKChanged(value))
        }));
    content = content.push(
        checkbox(app.knowledge.dataset_score_threshold_enabled)
            .label("Score 阈值")
            .on_toggle(|value| {
                Message::Knowledge(KnowledgeToolMessage::DatasetScoreThresholdEnabledChanged(value))
            })
            .style(settings_checkbox_style),
    );
    content = content.push(compact_input_row(
        "阈值",
        &app.knowledge.dataset_score_threshold_input,
        |value| Message::Knowledge(KnowledgeToolMessage::DatasetScoreThresholdChanged(value)),
    ));
    content = content.push(build_rerank_toggle(app));
    if app.knowledge.dataset_rerank_enabled {
        content = content.push(
            text_input("rerank 模型", &app.knowledge.dataset_rerank_model_input)
                .on_input(|value| {
                    Message::Knowledge(KnowledgeToolMessage::DatasetRerankModelChanged(value))
                })
                .padding([9, 10])
                .size(13)
                .style(settings_text_input_style),
        );
    }
    content.into()
}

fn build_rerank_toggle(app: &App) -> Element<'_, Message> {
    let rerank_available =
        app.knowledge.runtime_status.as_ref().is_some_and(|status| status.rerank);
    let label = if rerank_available { "Rerank 模型" } else { "Rerank 模型（未配置）" };
    let toggle = checkbox(app.knowledge.dataset_rerank_enabled && rerank_available)
        .label(label)
        .style(settings_checkbox_style);
    if rerank_available {
        toggle
            .on_toggle(|value| {
                Message::Knowledge(KnowledgeToolMessage::DatasetRerankEnabledChanged(value))
            })
            .into()
    } else {
        toggle.into()
    }
}

fn section_header<'a>(title: &'a str, description: &'a str) -> Element<'a, Message> {
    column![text(title).size(12), text(description).size(11).style(settings_muted_text_style),]
        .spacing(2)
        .into()
}

fn compact_input_row<'a>(
    label: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).size(12).style(settings_muted_text_style).width(Length::Fixed(82.0)),
        text_input("", value)
            .on_input(on_input)
            .padding([8, 9])
            .size(13)
            .style(settings_text_input_style)
            .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn option_card<'a>(
    title: &'a str,
    description: &'a str,
    icon: Icon,
    active: bool,
    enabled: bool,
    recommended: bool,
    message: Message,
) -> Element<'a, Message> {
    let mut title_row = row![text(title).size(13)].spacing(6).align_y(Alignment::Center);
    if recommended {
        title_row = title_row.push(settings_value_badge("推荐"));
    }
    let content = row![
        icon_badge(icon, active),
        column![title_row, text(description).size(11).style(settings_muted_text_style),]
            .spacing(3)
            .width(Length::Fill),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    button(
        container(content)
            .padding([10, 10])
            .width(Length::Fill)
            .style(move |theme: &Theme| option_card_style(theme, active, enabled)),
    )
    .padding(0)
    .width(Length::Fill)
    .style(button::text)
    .on_press_maybe(enabled.then_some(message))
    .into()
}

fn dataset_item<'a>(app: &'a App, dataset: &'a KnowledgeDatasetDto) -> Element<'a, Message> {
    let selected = app.knowledge.selected_dataset_id.as_deref() == Some(dataset.id.as_str());
    let title = text(&dataset.name).size(14);
    let subtitle = if dataset.description.trim().is_empty() {
        format!(
            "{} · {}",
            retrieval_label(&dataset.retrieval_mode),
            indexing_label(&dataset.indexing_mode)
        )
    } else {
        dataset.description.clone()
    };

    let content = column![
        row![
            icon_badge(Icon::Journals, selected),
            column![title, text(subtitle).size(12).style(settings_muted_text_style),]
                .spacing(4)
                .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            settings_value_badge(format!("{} 文档", dataset.document_count)),
            settings_value_badge(format!("{} 分段", dataset.chunk_count)),
        ]
        .spacing(6),
        row![
            settings_value_badge(chunking_label(&dataset.chunking_mode)),
            settings_value_badge(retrieval_label(&dataset.retrieval_mode)),
        ]
        .spacing(6),
    ]
    .spacing(10);

    button(
        container(content)
            .padding([12, 12])
            .width(Length::Fill)
            .style(move |theme: &Theme| item_style(theme, selected)),
    )
    .padding(0)
    .width(Length::Fill)
    .style(button::text)
    .on_press(Message::Knowledge(KnowledgeToolMessage::SelectDataset(dataset.id.clone())))
    .into()
}

fn build_detail<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let Some(dataset) = app.knowledge.selected_dataset() else {
        return settings_panel(
            column![
                text("选择或创建一个知识库").size(18),
                text("左侧列表会显示全部知识库。").size(13).style(settings_muted_text_style),
            ]
            .spacing(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center),
        )
        .into();
    };

    let header = row![
        column![
            text(&dataset.name).size(18),
            row![
                settings_value_badge(chunking_label(&dataset.chunking_mode)),
                settings_value_badge(indexing_label(&dataset.indexing_mode)),
                settings_value_badge(retrieval_label(&dataset.retrieval_mode)),
                settings_value_badge(format!("{} 文档", dataset.document_count)),
                settings_value_badge(format!("{} 分段", dataset.chunk_count)),
            ]
            .spacing(6),
        ]
        .spacing(8)
        .width(Length::Fill),
        button(row![icon_svg(Icon::Trash, 14.0), text("删除").size(13)].spacing(8))
            .on_press_maybe(
                (!app.knowledge.deleting)
                    .then_some(Message::Knowledge(KnowledgeToolMessage::DeleteSelectedDataset))
            )
            .padding([9, 12])
            .style(danger_action_btn_style),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let tabs = build_tabs(app);
    let body = match app.knowledge.active_tab {
        KnowledgeDetailTab::Documents => build_documents_tab(app, size),
        KnowledgeDetailTab::Retrieval => build_retrieval_tab(app),
        KnowledgeDetailTab::Settings => build_settings_tab(app, dataset),
    };

    column![settings_panel(header), tabs, body]
        .spacing(12)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn build_tabs(app: &App) -> Element<'_, Message> {
    let mut tabs = row![].spacing(8).align_y(Alignment::Center);
    for tab in KnowledgeDetailTab::ALL {
        tabs = tabs.push(segment_button(
            tab.title(),
            app.knowledge.active_tab == tab,
            Message::Knowledge(KnowledgeToolMessage::SelectTab(tab)),
        ));
    }
    settings_panel(tabs).into()
}

fn build_documents_tab<'a>(app: &'a App, size: Size) -> Element<'a, Message> {
    let upload = build_upload_panel(app);
    let documents = build_documents_table(app);
    if size.width >= 1220.0 {
        row![
            container(upload).width(Length::Fixed(360.0)).height(Length::Fill),
            container(documents).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    } else {
        column![
            container(upload).width(Length::Fill).height(Length::Fixed(280.0)),
            container(documents).width(Length::Fill).height(Length::Fill),
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }
}

fn build_upload_panel(app: &App) -> Element<'_, Message> {
    let editor = text_editor(&app.knowledge.document_content_editor)
        .on_action(|action| Message::Knowledge(KnowledgeToolMessage::DocumentContentAction(action)))
        .placeholder("粘贴文本、Markdown 或说明文档内容")
        .height(Length::Fill)
        .style(settings_text_editor_style);

    settings_panel(
        column![
            text("添加文本").size(14),
            text_input("文档名称", &app.knowledge.document_name_input)
                .on_input(|value| Message::Knowledge(KnowledgeToolMessage::DocumentNameChanged(
                    value
                )))
                .padding([9, 10])
                .size(13)
                .style(settings_text_input_style),
            container(editor).height(Length::Fill).width(Length::Fill),
            row![
                button(row![icon_svg(Icon::CloudUpload, 14.0), text("入库").size(13)].spacing(8))
                    .on_press_maybe(
                        (!app.knowledge.creating_document)
                            .then_some(Message::Knowledge(KnowledgeToolMessage::CreateDocument))
                    )
                    .padding([10, 14])
                    .style(primary_action_btn_style),
                button(text("清空").size(13))
                    .on_press(Message::Knowledge(KnowledgeToolMessage::ClearDocumentDraft))
                    .padding([10, 14])
                    .style(rounded_action_btn_style),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(10)
        .height(Length::Fill),
    )
    .into()
}

fn build_documents_table(app: &App) -> Element<'_, Message> {
    let search = text_input("搜索文档", &app.knowledge.document_search_query)
        .on_input(|value| Message::Knowledge(KnowledgeToolMessage::DocumentSearchChanged(value)))
        .padding([9, 10])
        .size(13)
        .style(settings_text_input_style);

    let header = row![
        text("名称").size(12).width(Length::FillPortion(4)),
        text("分段").size(12).width(Length::FillPortion(1)),
        text("状态").size(12).width(Length::FillPortion(1)),
        text("更新时间").size(12).width(Length::FillPortion(2)),
        text("操作").size(12).width(Length::Fixed(86.0)),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let query = app.knowledge.document_search_query.trim().to_lowercase();
    let mut rows = column![container(header).padding([8, 10]).style(chrome_style)].spacing(6);
    let mut matched = 0usize;
    for document in app
        .knowledge
        .documents
        .iter()
        .filter(|document| query.is_empty() || document.name.to_lowercase().contains(&query))
    {
        matched += 1;
        rows = rows.push(document_row(app, document));
    }
    if app.knowledge.loading_documents {
        rows = rows.push(empty_hint("正在加载文档"));
    } else if app.knowledge.documents.is_empty() {
        rows = rows.push(empty_hint("暂无文档"));
    } else if matched == 0 {
        rows = rows.push(empty_hint("没有匹配文档"));
    }

    settings_panel(
        column![search, scrollable(rows).height(Length::Fill)].spacing(10).height(Length::Fill),
    )
    .into()
}

fn document_row<'a>(app: &'a App, document: &'a KnowledgeDocumentDto) -> Element<'a, Message> {
    row![
        text(&document.name).size(13).width(Length::FillPortion(4)),
        text(document.chunk_count.to_string()).size(13).width(Length::FillPortion(1)),
        container(settings_value_badge(if document.enabled { "可用" } else { "停用" }))
            .width(Length::FillPortion(1)),
        text(format_time(document.updated_at_ms))
            .size(13)
            .style(settings_muted_text_style)
            .width(Length::FillPortion(2)),
        button(icon_svg(Icon::Trash, 13.0))
            .on_press_maybe((!app.knowledge.deleting).then_some(Message::Knowledge(
                KnowledgeToolMessage::DeleteDocument(document.id.clone())
            )))
            .padding([7, 9])
            .style(danger_action_btn_style)
            .width(Length::Fixed(46.0)),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding([9, 10])
    .into()
}

fn build_retrieval_tab(app: &App) -> Element<'_, Message> {
    let results = if app.knowledge.retrieve_results.is_empty() {
        empty_hint("召回结果会显示在这里")
    } else {
        let mut list = column![].spacing(10);
        for chunk in &app.knowledge.retrieve_results {
            list = list.push(chunk_card(chunk));
        }
        scrollable(list).height(Length::Fill).into()
    };

    settings_panel(
        column![
            row![
                text_input("输入查询文本", &app.knowledge.retrieve_query_input)
                    .on_input(|value| {
                        Message::Knowledge(KnowledgeToolMessage::RetrieveQueryChanged(value))
                    })
                    .padding([10, 12])
                    .size(13)
                    .style(settings_text_input_style)
                    .width(Length::Fill),
                text_input("Top K", &app.knowledge.retrieve_top_k_input)
                    .on_input(|value| {
                        Message::Knowledge(KnowledgeToolMessage::RetrieveTopKChanged(value))
                    })
                    .padding([10, 12])
                    .size(13)
                    .style(settings_text_input_style)
                    .width(Length::Fixed(90.0)),
                button(row![icon_svg(Icon::Search, 14.0), text("测试").size(13)].spacing(8))
                    .on_press_maybe(
                        (!app.knowledge.retrieving)
                            .then_some(Message::Knowledge(KnowledgeToolMessage::Retrieve))
                    )
                    .padding([10, 14])
                    .style(primary_action_btn_style),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                checkbox(app.knowledge.retrieve_score_threshold_enabled)
                    .label("Score 阈值")
                    .on_toggle(|value| {
                        Message::Knowledge(
                            KnowledgeToolMessage::RetrieveScoreThresholdEnabledChanged(value),
                        )
                    })
                    .style(settings_checkbox_style),
                text_input("0.15", &app.knowledge.retrieve_score_threshold_input)
                    .on_input(|value| {
                        Message::Knowledge(KnowledgeToolMessage::RetrieveScoreThresholdChanged(
                            value,
                        ))
                    })
                    .padding([9, 10])
                    .size(13)
                    .style(settings_text_input_style)
                    .width(Length::Fixed(110.0)),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            container(results).height(Length::Fill).width(Length::Fill),
        ]
        .spacing(12)
        .height(Length::Fill),
    )
    .into()
}

fn build_settings_tab<'a>(app: &'a App, dataset: &'a KnowledgeDatasetDto) -> Element<'a, Message> {
    let mut status_items = column![].spacing(10);
    if let Some(status) = &app.knowledge.runtime_status {
        status_items = status_items
            .push(kv_row("全文检索", support_label(status.full_text)))
            .push(kv_row("向量检索", support_label(status.vector)))
            .push(kv_row("混合检索", support_label(status.hybrid)))
            .push(kv_row("Rerank", support_label(status.rerank)));
        for (key, value) in &status.notes {
            status_items = status_items.push(kv_row(key, value));
        }
    } else {
        status_items = status_items.push(kv_row("运行状态", "检测中"));
    }

    settings_panel(
        column![
            kv_row("知识库 ID", &dataset.id),
            kv_row("分段模式", chunking_label(&dataset.chunking_mode)),
            kv_row("索引模式", indexing_label(&dataset.indexing_mode)),
            kv_row("检索模式", retrieval_label(&dataset.retrieval_mode)),
            kv_row("关键词数量", dataset.keyword_count.to_string()),
            kv_row("Top K", dataset.top_k.to_string()),
            kv_row("Score 阈值", score_threshold_label(dataset)),
            kv_row("Rerank 开关", enabled_label(dataset.rerank_enabled)),
            kv_row("Embedding", dataset_embedding_model_label(app, dataset)),
            kv_row("Rerank", dataset.rerank_model.as_deref().unwrap_or("未启用")),
            container(status_items).padding([8, 0]),
        ]
        .spacing(12),
    )
    .into()
}

fn chunk_card(chunk: &KnowledgeChunkDto) -> Element<'_, Message> {
    let score = chunk.score.map(|value| format!("{value:.3}")).unwrap_or_else(|| "-".to_string());
    container(
        column![
            row![
                text(&chunk.title).size(14).width(Length::Fill),
                settings_value_badge(format!("score {score}")),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            text(&chunk.content).size(13),
        ]
        .spacing(8),
    )
    .padding([12, 14])
    .width(Length::Fill)
    .style(chrome_style)
    .into()
}

fn kv_row<'a>(label: &'a str, value: impl ToString + 'a) -> Element<'a, Message> {
    row![
        text(label).size(13).style(settings_muted_text_style).width(Length::Fixed(120.0)),
        text(value.to_string()).size(13).width(Length::Fill),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn segment_button<'a>(label: &'a str, active: bool, message: Message) -> Element<'a, Message> {
    button(text(label).size(12))
        .on_press(message)
        .padding([7, 11])
        .style(move |theme: &Theme, status| segment_style(theme, status, active))
        .into()
}

fn icon_badge(icon: Icon, active: bool) -> Element<'static, Message> {
    container(icon_svg(icon, 16.0))
        .padding(8)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(if active {
                    theme.palette().primary.scale_alpha(0.18)
                } else {
                    palette.background.weak.color.scale_alpha(0.55)
                })),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: if active {
                        theme.palette().primary
                    } else {
                        palette.background.strong.color
                    },
                },
                ..Default::default()
            }
        })
        .into()
}

fn empty_hint<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(13).style(settings_muted_text_style))
        .padding([18, 16])
        .width(Length::Fill)
        .center_x(Length::Fill)
        .style(chrome_style)
        .into()
}

fn item_style(theme: &Theme, selected: bool) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let base = if selected {
        theme.palette().primary.scale_alpha(0.10)
    } else {
        palette.background.base.color
    };
    iced::widget::container::Style {
        background: Some(base.into()),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: if selected { theme.palette().primary } else { palette.background.strong.color },
        },
        ..Default::default()
    }
}

fn option_card_style(theme: &Theme, active: bool, enabled: bool) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    let background = if active {
        theme.palette().primary.scale_alpha(0.10)
    } else {
        palette.background.weak.color.scale_alpha(if enabled { 0.42 } else { 0.22 })
    };
    let border_color = if active {
        theme.palette().primary
    } else {
        palette.background.strong.color.scale_alpha(if enabled { 0.80 } else { 0.42 })
    };
    iced::widget::container::Style {
        background: Some(background.into()),
        border: Border {
            radius: 8.0.into(),
            width: if active { 2.0 } else { 1.0 },
            color: border_color,
        },
        text_color: (!enabled).then_some(theme.palette().text.scale_alpha(0.52)),
        ..Default::default()
    }
}

fn chrome_style(theme: &Theme) -> iced::widget::container::Style {
    let palette = theme.extended_palette();
    iced::widget::container::Style {
        background: Some(palette.background.weak.color.scale_alpha(0.42).into()),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: palette.background.strong.color.scale_alpha(0.80),
        },
        ..Default::default()
    }
}

fn segment_style(
    theme: &Theme,
    status: iced::widget::button::Status,
    active: bool,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let hovered = matches!(status, iced::widget::button::Status::Hovered);
    let background = if active {
        theme.palette().primary.scale_alpha(0.88)
    } else if hovered {
        palette.background.strong.color.scale_alpha(0.62)
    } else {
        palette.background.weak.color.scale_alpha(0.42)
    };
    iced::widget::button::Style {
        background: Some(background.into()),
        text_color: if active { Color::WHITE } else { theme.palette().text },
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: if active { theme.palette().primary } else { palette.background.strong.color },
        },
        ..Default::default()
    }
}

fn knowledge_toolbar_button_style(
    theme: &Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let is_dark = is_dark_palette(theme);
    let disabled = matches!(status, iced::widget::button::Status::Disabled);
    let background = match status {
        iced::widget::button::Status::Hovered => {
            if is_dark {
                palette.background.weak.color
            } else {
                Color::WHITE.scale_alpha(0.94)
            }
        }
        iced::widget::button::Status::Pressed => {
            if is_dark {
                palette.background.strong.color.scale_alpha(0.94)
            } else {
                palette.background.weak.color.scale_alpha(0.90)
            }
        }
        iced::widget::button::Status::Disabled => {
            if is_dark {
                palette.background.base.color.scale_alpha(0.32)
            } else {
                palette.background.weak.color.scale_alpha(0.52)
            }
        }
        _ => {
            if is_dark {
                palette.background.base.color.scale_alpha(0.68)
            } else {
                Color::WHITE.scale_alpha(0.72)
            }
        }
    };
    let border_color = if is_dark {
        palette.background.strong.color.scale_alpha(if disabled { 0.44 } else { 0.92 })
    } else {
        Color::from_rgba8(15, 23, 42, if disabled { 0.08 } else { 0.14 })
    };

    iced::widget::button::Style {
        background: Some(Background::Color(background)),
        text_color: theme.palette().text.scale_alpha(if disabled { 0.42 } else { 0.92 }),
        border: Border { radius: 12.0.into(), width: 1.0, color: border_color },
        ..Default::default()
    }
}

fn is_dark_palette(theme: &Theme) -> bool {
    let bg = theme.palette().background;
    bg.r + bg.g + bg.b < 1.5
}

fn indexing_label(mode: &KnowledgeIndexingMode) -> &'static str {
    match mode {
        KnowledgeIndexingMode::Economy => "经济",
        KnowledgeIndexingMode::HighQuality => "高质量",
    }
}

fn chunking_label(mode: &KnowledgeChunkingMode) -> &'static str {
    match mode {
        KnowledgeChunkingMode::General => "General",
        KnowledgeChunkingMode::ParentChild => "Parent-Child",
        KnowledgeChunkingMode::Qa => "Q&A",
    }
}

fn retrieval_label(mode: &KnowledgeRetrievalMode) -> &'static str {
    match mode {
        KnowledgeRetrievalMode::FullText => "全文检索",
        KnowledgeRetrievalMode::Vector => "向量检索",
        KnowledgeRetrievalMode::Hybrid => "混合检索",
    }
}

fn support_label(value: bool) -> &'static str {
    if value { "可用" } else { "不可用" }
}

fn enabled_label(value: bool) -> &'static str {
    if value { "启用" } else { "关闭" }
}

fn knowledge_embedding_model_options(app: &App) -> Vec<KnowledgeEmbeddingModelOption> {
    let shared = shared_embedding_model_option(
        &app.memory_settings.embedding_provider,
        &app.memory_settings.embedding_model,
        app.memory_settings.embedding_dimensions,
        &app.embedding_routes_settings.routes,
    );
    let mut options = vec![shared.clone()];
    let current = app.knowledge.dataset_embedding_model_input.trim();
    if !current.is_empty() && current != shared.value {
        options.push(KnowledgeEmbeddingModelOption {
            value: current.to_string(),
            label: format!("当前输入：{current}"),
        });
    }
    options
}

fn knowledge_embedding_model_selected(app: &App) -> KnowledgeEmbeddingModelOption {
    let shared = shared_embedding_model_option(
        &app.memory_settings.embedding_provider,
        &app.memory_settings.embedding_model,
        app.memory_settings.embedding_dimensions,
        &app.embedding_routes_settings.routes,
    );
    let current = app.knowledge.dataset_embedding_model_input.trim();
    if current.is_empty() || current == shared.value {
        return shared;
    }
    KnowledgeEmbeddingModelOption {
        value: current.to_string(),
        label: format!("当前输入：{current}"),
    }
}

fn shared_embedding_model_option(
    provider: &str,
    model: &str,
    dimensions: u32,
    routes: &[EmbeddingRouteDraft],
) -> KnowledgeEmbeddingModelOption {
    let summary = shared_embedding_model_summary(provider, model, dimensions, routes);
    KnowledgeEmbeddingModelOption {
        value: String::new(), label: format!("记忆配置：{summary}")
    }
}

fn shared_embedding_model_summary(
    provider: &str,
    model: &str,
    dimensions: u32,
    routes: &[EmbeddingRouteDraft],
) -> String {
    let provider = provider.trim();
    let model = model.trim();
    if model.is_empty() {
        return "未配置向量化".to_string();
    }
    if let Some(hint) = model.strip_prefix("hint:").map(str::trim).filter(|value| !value.is_empty())
    {
        if let Some(route) = routes.iter().find(|route| route.pattern.trim() == hint) {
            let route_model = route.model.trim();
            let route_provider = route.provider.trim();
            let route_dimensions =
                route.dimensions.trim().parse::<u32>().ok().unwrap_or(dimensions);
            if !route_provider.is_empty() && !route_model.is_empty() && route_dimensions > 0 {
                return format!("{hint} -> {route_model} / {route_dimensions}维");
            }
        }
        if provider.is_empty() || provider == "none" {
            return "未配置向量化".to_string();
        }
        return format!("{model} / {}维", dimensions);
    }
    if provider.is_empty() || provider == "none" {
        return "未配置向量化".to_string();
    }
    format!("{model} / {}维", dimensions)
}

fn dataset_embedding_model_label(app: &App, dataset: &KnowledgeDatasetDto) -> String {
    dataset
        .embedding_model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            shared_embedding_model_option(
                &app.memory_settings.embedding_provider,
                &app.memory_settings.embedding_model,
                app.memory_settings.embedding_dimensions,
                &app.embedding_routes_settings.routes,
            )
            .label
        })
}

fn score_threshold_label(dataset: &KnowledgeDatasetDto) -> String {
    if dataset.score_threshold_enabled {
        format!("{:.2}", dataset.score_threshold)
    } else {
        "关闭".to_string()
    }
}

fn format_time(ms: u64) -> String {
    if ms == 0 { "-".to_string() } else { format!("{ms}") }
}

#[cfg(test)]
#[path = "knowledge_tests.rs"]
mod knowledge_tests;
