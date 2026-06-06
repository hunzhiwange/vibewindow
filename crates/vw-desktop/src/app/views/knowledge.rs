//! 知识库工作台视图。

use crate::app::assets::Icon;
use crate::app::components::system_settings_common::{
    danger_action_btn_style, icon_svg, primary_action_btn_style, rounded_action_btn_style,
    settings_error_banner, settings_muted_text_style, settings_panel, settings_panel_style,
    settings_success_banner, settings_text_editor_style, settings_text_input_style,
    settings_value_badge,
};
use crate::app::message::KnowledgeToolMessage;
use crate::app::state::KnowledgeDetailTab;
use crate::app::{App, Message};
use iced::widget::{
    button, column, container, responsive, row, scrollable, text, text_editor, text_input,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Size, Theme};
use vw_gateway_client::{
    KnowledgeChunkDto, KnowledgeDatasetDto, KnowledgeDocumentDto, KnowledgeIndexingMode,
    KnowledgeRetrievalMode,
};

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
            build_retrieval_mode_row(app),
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

fn build_retrieval_mode_row(app: &App) -> Element<'_, Message> {
    column![
        row![
            segment_button(
                "全文",
                app.knowledge.dataset_retrieval_mode == KnowledgeRetrievalMode::FullText,
                Message::Knowledge(KnowledgeToolMessage::DatasetRetrievalModeChanged(
                    KnowledgeRetrievalMode::FullText,
                )),
            ),
            segment_button(
                "向量",
                app.knowledge.dataset_retrieval_mode == KnowledgeRetrievalMode::Vector,
                Message::Knowledge(KnowledgeToolMessage::DatasetRetrievalModeChanged(
                    KnowledgeRetrievalMode::Vector,
                )),
            ),
            segment_button(
                "混合",
                app.knowledge.dataset_retrieval_mode == KnowledgeRetrievalMode::Hybrid,
                Message::Knowledge(KnowledgeToolMessage::DatasetRetrievalModeChanged(
                    KnowledgeRetrievalMode::Hybrid,
                )),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            segment_button(
                "经济",
                app.knowledge.dataset_indexing_mode == KnowledgeIndexingMode::Economy,
                Message::Knowledge(KnowledgeToolMessage::DatasetIndexingModeChanged(
                    KnowledgeIndexingMode::Economy,
                )),
            ),
            segment_button(
                "高质量",
                app.knowledge.dataset_indexing_mode == KnowledgeIndexingMode::HighQuality,
                Message::Knowledge(KnowledgeToolMessage::DatasetIndexingModeChanged(
                    KnowledgeIndexingMode::HighQuality,
                )),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8)
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
            kv_row("索引模式", indexing_label(&dataset.indexing_mode)),
            kv_row("检索模式", retrieval_label(&dataset.retrieval_mode)),
            kv_row("Embedding", dataset.embedding_model.as_deref().unwrap_or("使用全局配置")),
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

fn kv_row<'a>(label: &'a str, value: &'a str) -> Element<'a, Message> {
    row![
        text(label).size(13).style(settings_muted_text_style).width(Length::Fixed(120.0)),
        text(value).size(13).width(Length::Fill),
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

fn format_time(ms: u64) -> String {
    if ms == 0 { "-".to_string() } else { format!("{ms}") }
}

#[cfg(test)]
#[path = "knowledge_tests.rs"]
mod knowledge_tests;
