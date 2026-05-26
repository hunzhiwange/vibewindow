//! 设计导入测试模块，验证 Tailwind 和通用导入路径生成稳定的设计元素树。

use super::figma::{figma_json_to_design_doc, figma_json_to_design_doc_with_raw};
use serde_json::json;

#[test]
fn top_level_pages_without_explicit_group_metadata_remain_flat() {
    let json = json!({
        "document": {
            "children": [
                {
                    "name": "AI智能托管",
                    "backgroundColor": "#f2f2f2",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "id": "page-1-card",
                            "name": "卡片",
                            "size": { "x": 200.0, "y": 100.0 },
                            "transform": { "x": 10.0, "y": 20.0 }
                        }
                    ]
                },
                {
                    "name": "草稿",
                    "backgroundColor": "#f2f2f2",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "id": "page-2-card",
                            "name": "面板",
                            "size": { "x": 300.0, "y": 160.0 },
                            "transform": { "x": 24.0, "y": 32.0 }
                        }
                    ]
                },
                {
                    "name": "控件",
                    "backgroundColor": "#ffffff",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "id": "page-3-card",
                            "name": "按钮",
                            "size": { "x": 120.0, "y": 48.0 },
                            "transform": { "x": 18.0, "y": 12.0 }
                        }
                    ]
                }
            ]
        }
    });

    let doc = figma_json_to_design_doc(json).expect("should import top-level pages");

    assert_eq!(doc.children.len(), 3);
    assert_eq!(doc.children[0].name.as_deref(), Some("AI智能托管"));
    assert_eq!(doc.children[1].name.as_deref(), Some("草稿"));
    assert_eq!(doc.children[2].name.as_deref(), Some("控件"));
    assert!(doc.children.iter().all(|element| element.group_id == 0));
    assert_eq!(doc.groups.len(), 1);
    assert_eq!(doc.groups[0].id, 0);
}

#[test]
fn explicit_group_nodes_are_converted_to_group_metadata() {
    let json = json!({
        "document": {
            "children": [
                {
                    "type": "GROUP",
                    "name": "业务线",
                    "children": [
                        {
                            "type": "CANVAS",
                            "name": "页面A",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "group-a-card",
                                    "name": "卡片A",
                                    "size": { "x": 160.0, "y": 80.0 },
                                    "transform": { "x": 8.0, "y": 16.0 }
                                }
                            ]
                        },
                        {
                            "type": "CANVAS",
                            "name": "页面B",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "group-b-card",
                                    "name": "卡片B",
                                    "size": { "x": 180.0, "y": 90.0 },
                                    "transform": { "x": 12.0, "y": 18.0 }
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });

    let doc = figma_json_to_design_doc(json).expect("should import explicit groups");

    assert_eq!(doc.children.len(), 2);
    assert_eq!(doc.groups.len(), 1);
    assert_eq!(doc.groups[0].name, "业务线");
    assert!(doc.children.iter().all(|element| element.group_id == 0));
}

#[test]
fn implicit_top_level_group_containers_are_converted_to_group_metadata() {
    let json = json!({
        "document": {
            "children": [
                {
                    "name": "AI智能托管",
                    "backgroundColor": "#f2f2f2",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "name": "External Symbols",
                            "visible": false,
                            "children": [
                                {
                                    "id": "symbol-card",
                                    "name": "隐藏组件",
                                    "size": { "x": 64.0, "y": 32.0 },
                                    "transform": { "x": 0.0, "y": 0.0 }
                                }
                            ]
                        },
                        {
                            "name": "订单自动审核托管_默认",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "screen-a-card",
                                    "name": "卡片A",
                                    "size": { "x": 200.0, "y": 100.0 },
                                    "transform": { "x": 10.0, "y": 20.0 }
                                }
                            ]
                        },
                        {
                            "name": "订货单托管",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "screen-b-card",
                                    "name": "卡片B",
                                    "size": { "x": 220.0, "y": 120.0 },
                                    "transform": { "x": 20.0, "y": 40.0 }
                                }
                            ]
                        }
                    ]
                },
                {
                    "name": "草稿",
                    "backgroundColor": "#f2f2f2",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "name": "删除机器人",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "draft-card",
                                    "name": "卡片C",
                                    "size": { "x": 180.0, "y": 90.0 },
                                    "transform": { "x": 24.0, "y": 36.0 }
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });

    let doc = figma_json_to_design_doc(json).expect("should import inferred groups");

    assert_eq!(doc.groups.len(), 2);
    assert_eq!(doc.groups[0].name, "AI智能托管");
    assert_eq!(doc.groups[1].name, "草稿");
    assert_eq!(doc.children.len(), 3);
    assert_eq!(doc.children[0].name.as_deref(), Some("订单自动审核托管_默认"));
    assert_eq!(doc.children[1].name.as_deref(), Some("订货单托管"));
    assert_eq!(doc.children[2].name.as_deref(), Some("删除机器人"));
    assert_eq!(doc.children[0].group_id, 0);
    assert_eq!(doc.children[1].group_id, 0);
    assert_eq!(doc.children[2].group_id, 1);
}

#[test]
fn mixed_grouped_and_ungrouped_pages_keep_distinct_group_ids() {
    let json = json!({
        "document": {
            "children": [
                {
                    "type": "GROUP",
                    "name": "业务线",
                    "children": [
                        {
                            "type": "CANVAS",
                            "name": "页面A",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "group-a-card",
                                    "name": "卡片A",
                                    "size": { "x": 160.0, "y": 80.0 },
                                    "transform": { "x": 8.0, "y": 16.0 }
                                }
                            ]
                        }
                    ]
                },
                {
                    "name": "散页",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "id": "ungrouped-card",
                            "name": "卡片B",
                            "size": { "x": 180.0, "y": 90.0 },
                            "transform": { "x": 12.0, "y": 18.0 }
                        }
                    ]
                }
            ]
        }
    });

    let doc = figma_json_to_design_doc(json).expect("should keep grouped and ungrouped pages");

    assert_eq!(doc.groups.len(), 2);
    assert_eq!(doc.groups[0].id, 0);
    assert_eq!(doc.groups[0].name, "业务线");
    assert_eq!(doc.groups[1].id, 1);
    assert_eq!(doc.groups[1].name, "散页");
    assert_eq!(doc.children.len(), 2);
    assert_eq!(doc.children[0].group_id, 0);
    assert_eq!(doc.children[1].group_id, 1);
}

#[test]
fn inferred_groups_preserve_top_level_group_names_in_design_doc() {
    let json = json!({
        "document": {
            "children": [
                {
                    "name": "AI智能托管",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "name": "页面A",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "card-a",
                                    "name": "卡片A",
                                    "size": { "x": 100.0, "y": 80.0 },
                                    "transform": { "x": 0.0, "y": 0.0 }
                                }
                            ]
                        }
                    ]
                },
                {
                    "name": "草稿",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "name": "页面B",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "children": [
                                {
                                    "id": "card-b",
                                    "name": "卡片B",
                                    "size": { "x": 120.0, "y": 90.0 },
                                    "transform": { "x": 0.0, "y": 0.0 }
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });

    let doc =
        figma_json_to_design_doc(json).expect("should preserve inferred groups in design doc");

    assert_eq!(doc.groups.len(), 2);
    assert_eq!(doc.groups[0].name, "AI智能托管");
    assert_eq!(doc.groups[1].name, "草稿");
}

#[test]
fn instance_nodes_expand_children_from_raw_symbol_master() {
    let json = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "type": "INSTANCE",
                            "name": "顶部导航",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "size": { "x": 1440.0, "y": 48.0 }
                        }
                    ]
                }
            ]
        }
    });

    let raw = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "guid": { "sessionID": 0, "localID": 10 },
                    "transform": { "m02": 0.0, "m12": 0.0 },
                    "children": [
                        {
                            "type": { "value": "INSTANCE" },
                            "name": "顶部导航",
                            "guid": { "sessionID": 11, "localID": 705 },
                            "transform": { "m02": 0.0, "m12": 0.0 },
                            "size": { "x": 1440.0, "y": 48.0 },
                            "symbolData": {
                                "symbolID": { "sessionID": 1, "localID": 2 },
                                "symbolOverrides": [
                                    {
                                        "guidPath": { "guids": [{ "sessionID": 0, "localID": 2550 }] },
                                        "textData": { "characters": "商品销售报" },
                                        "fontName": { "family": "Roboto", "style": "Regular" }
                                    }
                                ]
                            },
                            "derivedSymbolData": [
                                {
                                    "guidPath": { "guids": [{ "sessionID": 0, "localID": 2521 }] },
                                    "fillGeometry": [
                                        { "commands": ["M", 0.0, 0.0, "L", 1440.0, 0.0, "L", 1440.0, 48.0, "L", 0.0, 48.0, "Z"] }
                                    ]
                                }
                            ]
                        }
                    ]
                },
                {
                    "type": { "value": "SYMBOL" },
                    "name": "顶部导航",
                    "guid": { "sessionID": 1, "localID": 2 },
                    "children": [
                        {
                            "type": { "value": "ROUNDED_RECTANGLE" },
                            "name": "背景",
                            "guid": { "sessionID": 1, "localID": 3 },
                            "overrideKey": { "sessionID": 0, "localID": 2521 },
                            "transform": { "m02": 0.0, "m12": 0.0 },
                            "size": { "x": 1440.0, "y": 48.0 },
                            "fillPaints": [{ "color": "#ffffff" }]
                        },
                        {
                            "type": { "value": "FRAME" },
                            "name": "Tab01",
                            "guid": { "sessionID": 1, "localID": 20 },
                            "transform": { "m02": 126.0, "m12": 14.121212 },
                            "size": { "x": 112.0, "y": 33.878788 },
                            "frameMaskDisabled": false,
                            "children": [
                                {
                                    "type": { "value": "TEXT" },
                                    "name": "商品销售报",
                                    "guid": { "sessionID": 1, "localID": 21 },
                                    "overrideKey": { "sessionID": 0, "localID": 2550 },
                                    "transform": { "m02": 16.0, "m12": 8.439394 },
                                    "size": { "x": 80.0, "y": 14.0 },
                                    "textData": { "characters": "默认文案" },
                                    "fontName": { "family": "Roboto", "style": "Regular" },
                                    "fontSize": 12.0,
                                    "fillPaints": [{ "color": "#000000" }]
                                }
                            ]
                        }
                    ]
                }
            ]
        }
    });

    let doc = figma_json_to_design_doc_with_raw(json, Some(&raw)).expect("should expand instance");

    let top_bar = &doc.children[0].children[0];
    assert_eq!(top_bar.name.as_deref(), Some("顶部导航"));
    assert_eq!(top_bar.kind, "frame");
    assert_eq!(top_bar.children.len(), 2);
    assert_eq!(top_bar.children[0].name.as_deref(), Some("背景"));
    assert_eq!(top_bar.children[0].kind, "rectangle");
    assert_eq!(top_bar.children[0].fill.as_ref(), Some(&json!("#ffffffff")));

    let tab = &top_bar.children[1];
    assert_eq!(tab.name.as_deref(), Some("Tab01"));
    assert_eq!(tab.kind, "frame");
    assert_eq!(tab.clip, Some(true));
    assert_eq!(tab.children.len(), 1);

    let label = &tab.children[0];
    assert_eq!(label.kind, "text");
    assert_eq!(label.content.as_deref(), Some("商品销售报"));
    assert_eq!(label.text_growth.as_deref(), None);
    assert_eq!(label.font_family.as_deref(), Some("Roboto"));
}

#[test]
fn raw_stroke_and_clip_metadata_are_preserved_during_import() {
    let json = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "type": "RECTANGLE",
                            "name": "描边卡片",
                            "transform": { "x": 10.0, "y": 20.0 },
                            "size": { "x": 100.0, "y": 40.0 },
                            "strokePaints": [{ "color": "#eeeeee" }]
                        },
                        {
                            "type": "FRAME",
                            "name": "裁剪容器",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "size": { "x": 50.0, "y": 50.0 }
                        }
                    ]
                }
            ]
        }
    });

    let raw = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "guid": { "sessionID": 0, "localID": 1 },
                    "transform": { "m02": 0.0, "m12": 0.0 },
                    "children": [
                        {
                            "type": { "value": "RECTANGLE" },
                            "name": "描边卡片",
                            "guid": { "sessionID": 0, "localID": 2 },
                            "transform": { "m02": 10.0, "m12": 20.0 },
                            "size": { "x": 100.0, "y": 40.0 },
                            "strokeAlign": { "value": "INSIDE" },
                            "strokeWeight": 1.0,
                            "strokePaints": [{ "color": "#eeeeee" }]
                        },
                        {
                            "type": { "value": "FRAME" },
                            "name": "裁剪容器",
                            "guid": { "sessionID": 0, "localID": 3 },
                            "transform": { "m02": 0.0, "m12": 0.0 },
                            "size": { "x": 50.0, "y": 50.0 },
                            "frameMaskDisabled": false
                        }
                    ]
                }
            ]
        }
    });

    let doc =
        figma_json_to_design_doc_with_raw(json, Some(&raw)).expect("should preserve raw metadata");

    let page = &doc.children[0];
    let stroke_card = &page.children[0];
    assert_eq!(
        stroke_card.stroke.as_ref().and_then(|stroke| stroke.align.as_deref()),
        Some("inside")
    );
    assert_eq!(
        stroke_card.stroke.as_ref().and_then(|stroke| stroke.thickness.as_ref()),
        Some(&json!(1.0))
    );
    assert_eq!(
        stroke_card.stroke.as_ref().and_then(|stroke| stroke.fill.as_deref()),
        Some("#eeeeeeff")
    );

    let clip_frame = &page.children[1];
    assert_eq!(clip_frame.clip, Some(true));
}

#[test]
fn raw_rgba_colors_and_path_groups_align_more_closely_with_expected_output() {
    let json = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "type": "FRAME",
                            "name": "我的",
                            "transform": { "x": 1392.0, "y": 16.0 },
                            "size": { "x": 16.0, "y": 16.0 }
                        },
                        {
                            "type": "TEXT",
                            "name": "标签",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "size": { "x": 80.0, "y": 14.0 },
                            "textData": { "characters": "商品销售报" },
                            "fontSize": 12.0,
                            "fontName": { "family": "Roboto", "style": "Regular" },
                            "fillPaints": [{ "color": { "r": 0.0, "g": 0.0, "b": 0.0, "a": 1.0 } }]
                        }
                    ]
                }
            ]
        }
    });

    let raw = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "guid": { "sessionID": 0, "localID": 1 },
                    "transform": { "m02": 0.0, "m12": 0.0 },
                    "children": [
                        {
                            "type": { "value": "FRAME" },
                            "name": "我的",
                            "guid": { "sessionID": 1, "localID": 4 },
                            "transform": { "m02": 1392.0, "m12": 16.0 },
                            "size": { "x": 16.0, "y": 16.0 },
                            "children": [
                                {
                                    "type": { "value": "ROUNDED_RECTANGLE" },
                                    "name": "矩形",
                                    "guid": { "sessionID": 1, "localID": 5 },
                                    "transform": { "m02": 0.0, "m12": 0.0 },
                                    "size": { "x": 16.0, "y": 16.0 }
                                },
                                {
                                    "type": { "value": "VECTOR" },
                                    "name": "我的",
                                    "guid": { "sessionID": 1, "localID": 6 },
                                    "transform": { "m02": 0.0, "m12": 0.0 },
                                    "size": { "x": 15.2498, "y": 16.0 },
                                    "fillGeometry": [
                                        { "commands": ["M", 0.0, 0.0, "L", 10.0, 0.0, "L", 10.0, 5.0, "Z"] }
                                    ],
                                    "children": [
                                        {
                                            "type": { "value": "VECTOR" },
                                            "name": "路径",
                                            "guid": { "sessionID": 1, "localID": 7 },
                                            "transform": { "m02": 0.0, "m12": 0.0 },
                                            "size": { "x": 10.0, "y": 5.0 },
                                            "fillGeometry": [
                                                { "commands": ["M", 0.0, 0.0, "L", 10.0, 0.0, "L", 10.0, 5.0, "Z"] }
                                            ]
                                        }
                                    ]
                                }
                            ]
                        },
                        {
                            "type": { "value": "TEXT" },
                            "name": "标签",
                            "guid": { "sessionID": 0, "localID": 2 },
                            "transform": { "m02": 0.0, "m12": 0.0 },
                            "size": { "x": 80.0, "y": 14.0 },
                            "textData": { "characters": "商品销售报" },
                            "fontSize": 12.0,
                            "fontName": { "family": "Roboto", "style": "Regular" },
                            "fillPaints": [{ "color": { "r": 0.0, "g": 0.0, "b": 0.0, "a": 1.0 } }]
                        }
                    ]
                }
            ]
        }
    });

    let doc = figma_json_to_design_doc_with_raw(json, Some(&raw))
        .expect("should align closer to expected output");

    let page = &doc.children[0];
    assert_eq!(page.children[0].kind, "group");
    assert_eq!(page.children[0].children.len(), 0);

    let label = &page.children[1];
    assert_eq!(label.fill.as_ref(), Some(&json!("#000000ff")));
    assert_eq!(label.text_growth.as_deref(), None);
}

#[test]
fn raw_line_height_objects_and_text_colors_are_normalized() {
    let json = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "transform": { "x": 0.0, "y": 0.0 },
                    "children": [
                        {
                            "type": "TEXT",
                            "name": "说明",
                            "transform": { "x": 0.0, "y": 0.0 },
                            "size": { "x": 100.0, "y": 20.0 },
                            "textData": { "characters": "用于定义托管规则" },
                            "fontSize": 14.0,
                            "fontName": { "family": "Roboto", "style": "Regular" }
                        }
                    ]
                }
            ]
        }
    });

    let raw = json!({
        "document": {
            "children": [
                {
                    "name": "页面",
                    "guid": { "sessionID": 0, "localID": 1 },
                    "transform": { "m02": 0.0, "m12": 0.0 },
                    "children": [
                        {
                            "type": { "value": "TEXT" },
                            "name": "说明",
                            "guid": { "sessionID": 0, "localID": 2 },
                            "transform": { "m02": 0.0, "m12": 0.0 },
                            "size": { "x": 100.0, "y": 20.0 },
                            "textData": { "characters": "用于定义托管规则" },
                            "fontSize": 14.0,
                            "fontName": { "family": "Roboto", "style": "Regular" },
                            "lineHeight": { "value": 22.0, "units": { "value": "PIXELS" } },
                            "fillPaints": [{ "color": { "r": 0.4, "g": 0.4, "b": 0.4, "a": 1.0 } }]
                        }
                    ]
                }
            ]
        }
    });

    let doc = figma_json_to_design_doc_with_raw(json, Some(&raw))
        .expect("should normalize text metadata");
    let text = &doc.children[0].children[0];
    assert_eq!(text.fill.as_ref(), Some(&json!("#666666ff")));
    assert_eq!(text.line_height.as_ref(), Some(&json!(22.0 / 14.0)));
}
