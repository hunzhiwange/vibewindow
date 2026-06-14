use super::update;
use crate::app::App;
use crate::app::views::design::models::ColorFormat;
use crate::apps::mindmap::message::types::MindMapMessage;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapColorTarget,
    MindMapDiagramType, MindMapDoodleStroke, MindMapLayoutFormat, MindMapTab, OrgChartLayoutFormat,
    TimelineLayoutFormat, TreeLayoutFormat,
};
use iced::widget::text_editor;
use iced::{Color, Point, Vector};
use serde_json::json;

fn app_with_tab() -> App {
    let mut app = App::new().0;
    app.window_size = (800.0, 600.0);
    app.mindmap_tabs.push(MindMapTab::new(
        "tab-1".to_string(),
        "Tab".to_string(),
        None,
        model::default_doc(),
    ));
    app.mindmap_active_tab_id = Some("tab-1".to_string());
    app
}

#[test]
fn update_dispatches_file_and_canvas_messages() {
    let mut app = app_with_tab();

    let _ = update(&mut app, MindMapMessage::New);
    let _ = update(&mut app, MindMapMessage::Open);
    let _ = update(
        &mut app,
        MindMapMessage::FileOpened(Ok((Some("/tmp/map.md".to_string()), "# Imported".to_string()))),
    );
    let _ =
        update(&mut app, MindMapMessage::LoadPersistedFinished(Ok(Some(json!({ "bad": true })))));
    let _ = update(&mut app, MindMapMessage::Save);
    let _ = update(&mut app, MindMapMessage::SaveFinished(Err("save failed".to_string())));
    let _ = update(&mut app, MindMapMessage::SaveAs);
    let _ = update(&mut app, MindMapMessage::SaveAsJson);
    let _ = update(&mut app, MindMapMessage::ToggleExportMenu);
    let _ = update(&mut app, MindMapMessage::ExportPng);
    let _ = update(&mut app, MindMapMessage::ExportJpeg);
    let _ = update(&mut app, MindMapMessage::ExportSvg);
    let _ = update(&mut app, MindMapMessage::ExportFinished(Err("export failed".to_string())));
    let _ = update(&mut app, MindMapMessage::FileSaved(Some("/tmp/saved.md".to_string())));
    let _ = update(&mut app, MindMapMessage::FileSaved(None));

    let _ = update(&mut app, MindMapMessage::PanBy(Vector::new(1.0, 2.0)));
    let _ = update(&mut app, MindMapMessage::Zoom(1.2, Some(Point::new(100.0, 100.0))));
    let _ = update(&mut app, MindMapMessage::ZoomSet(2.0));
    let _ = update(&mut app, MindMapMessage::ZoomFit);
    let _ = update(&mut app, MindMapMessage::ToggleZoomMenu);
    let _ = update(&mut app, MindMapMessage::SelectNode(vec![0]));
    let _ = update(&mut app, MindMapMessage::ClearSelection);
    let _ = update(
        &mut app,
        MindMapMessage::NodeDragStart(vec![0], Point::new(1.0, 2.0), Point::new(3.0, 4.0)),
    );
    let _ = update(&mut app, MindMapMessage::NodeDragged(vec![0], Vector::new(5.0, 6.0)));
    let _ = update(&mut app, MindMapMessage::SetCanvasTool(MindMapCanvasTool::Pen));
    let _ = update(&mut app, MindMapMessage::SetDoodleColor(0x12345678));
    let _ = update(&mut app, MindMapMessage::SetDoodleWidth(8.0));
    let _ = update(
        &mut app,
        MindMapMessage::DoodleCommit(MindMapDoodleStroke {
            points_world: vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
            rgba: 0x12345678,
            width_px: 8.0,
        }),
    );
    let _ = update(&mut app, MindMapMessage::DoodleErase(Point::new(1.0, 1.0), 1.0));

    assert!(app.error_message.as_deref().is_some_and(|message| message.contains("export failed")));
    assert!(app.active_mindmap_tab().is_some());
}

#[test]
fn update_dispatches_node_metadata_color_theme_and_markdown_messages() {
    let mut app = app_with_tab();

    let _ = update(&mut app, MindMapMessage::ClosePickers);
    let _ = update(&mut app, MindMapMessage::ToggleActionMenu);
    let _ = update(&mut app, MindMapMessage::TogglePriorityPicker);
    let _ = update(&mut app, MindMapMessage::SelectNode(vec![]));
    let _ = update(&mut app, MindMapMessage::SetNodePriority(3));
    let _ = update(&mut app, MindMapMessage::ClearNodePriority);
    let _ = update(&mut app, MindMapMessage::ToggleNodeUrlEditor);
    let _ = update(&mut app, MindMapMessage::NodeUrlChanged("https://example.com".to_string()));
    let _ = update(&mut app, MindMapMessage::SaveNodeUrl);
    let _ = update(&mut app, MindMapMessage::ClearNodeUrl);
    let _ = update(&mut app, MindMapMessage::OpenNodeUrl);
    let _ = update(&mut app, MindMapMessage::OpenNodeUrlAt(vec![99]));

    let _ = update(&mut app, MindMapMessage::ToggleNodeTextEditor);
    let _ = update(&mut app, MindMapMessage::NodeTextChanged("Edited".to_string()));
    let _ = update(&mut app, MindMapMessage::NodeTextEditorAction(text_editor::Action::SelectAll));
    let _ = update(&mut app, MindMapMessage::NodeTextEditorEnter { shift: true });
    let _ = update(&mut app, MindMapMessage::SaveNodeText);

    let _ = update(
        &mut app,
        MindMapMessage::OpenColorPicker(MindMapColorTarget::Background, Color::WHITE),
    );
    let _ = update(&mut app, MindMapMessage::ColorPickerChanged(Color::BLACK));
    let _ = update(&mut app, MindMapMessage::ColorPickerFormatChanged(ColorFormat::Hsl));
    let _ = update(&mut app, MindMapMessage::ResetColorTarget(MindMapColorTarget::Background));
    let _ = update(&mut app, MindMapMessage::SetBackground(Some(0x11223344)));

    let _ = update(&mut app, MindMapMessage::ToggleDiagramTypePicker);
    let _ = update(&mut app, MindMapMessage::SelectDiagramType(MindMapDiagramType::OrgChart));
    let _ = update(&mut app, MindMapMessage::SetDiagramType(MindMapDiagramType::Fishbone));
    let _ = update(&mut app, MindMapMessage::SetLayoutFormat(MindMapLayoutFormat::LeftAligned));
    let _ =
        update(&mut app, MindMapMessage::SetOrgChartLayoutFormat(OrgChartLayoutFormat::LeftRight));
    let _ =
        update(&mut app, MindMapMessage::SetFishboneLayoutFormat(FishboneLayoutFormat::HeadLeft));
    let _ = update(&mut app, MindMapMessage::SetTimelineLayoutFormat(TimelineLayoutFormat::AllUp));
    let _ =
        update(&mut app, MindMapMessage::SetBracketLayoutFormat(BracketLayoutFormat::BraceLeft));
    let _ = update(&mut app, MindMapMessage::SetTreeLayoutFormat(TreeLayoutFormat::RightAligned));

    let _ = update(&mut app, MindMapMessage::ToggleThemePanel);
    let _ = update(&mut app, MindMapMessage::SetThemeGroup("retro".to_string()));
    let _ = update(&mut app, MindMapMessage::SetThemeVariant("classic".to_string(), 2));
    let _ = update(&mut app, MindMapMessage::SaveThemeToCustom);
    let _ = update(&mut app, MindMapMessage::DeleteCustomTheme(0));
    let _ = update(&mut app, MindMapMessage::CancelThemeBackground);
    let _ = update(&mut app, MindMapMessage::SetEdgeStyle(EdgeStyle::Dashed));
    let _ = update(&mut app, MindMapMessage::SetNodeBorderStyle(EdgeStyle::Dotted));

    let _ = update(&mut app, MindMapMessage::ToggleMarkdownImport);
    let _ = update(
        &mut app,
        MindMapMessage::MarkdownImportEditorAction(text_editor::Action::SelectAll),
    );
    let _ = update(&mut app, MindMapMessage::ApplyMarkdownImport);

    let tab = app.active_mindmap_tab().unwrap();
    assert_eq!(tab.node_border_style, EdgeStyle::Solid);
    assert!(!tab.follow_theme_background);
}

#[test]
fn update_dispatches_node_editing_messages() {
    let mut app = app_with_tab();

    let _ = update(&mut app, MindMapMessage::SelectNode(vec![]));
    let _ = update(&mut app, MindMapMessage::AddChild);
    let _ = update(&mut app, MindMapMessage::SelectNode(vec![0]));
    let _ = update(&mut app, MindMapMessage::AddSibling);
    let _ = update(&mut app, MindMapMessage::AddChildAt(vec![]));
    let _ = update(&mut app, MindMapMessage::AddSiblingAt(vec![0]));
    let _ = update(&mut app, MindMapMessage::ToggleCollapseAt(vec![0]));
    let _ = update(&mut app, MindMapMessage::OpenNodeContextMenu(vec![0], Point::new(10.0, 20.0)));
    let _ = update(&mut app, MindMapMessage::CloseContextMenu);
    let _ = update(&mut app, MindMapMessage::SelectNode(vec![0]));
    let _ = update(&mut app, MindMapMessage::CopyNode);
    let _ = update(&mut app, MindMapMessage::PasteNode);
    let _ = update(&mut app, MindMapMessage::DuplicateNode);
    let _ = update(&mut app, MindMapMessage::CutNode);
    let _ = update(&mut app, MindMapMessage::DeleteNode);
    let _ = update(&mut app, MindMapMessage::Undo);
    let _ = update(&mut app, MindMapMessage::Redo);

    assert!(app.active_mindmap_tab().unwrap().doc.text.len() > 0);
}
