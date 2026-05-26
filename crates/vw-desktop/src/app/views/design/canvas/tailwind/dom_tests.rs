//! 设计画布 Tailwind 支持模块。
//!
//! 该模块负责把 Tailwind 风格类名转换为画布渲染可用的结构化样式，供布局、形状和文本渲染路径复用。

use super::{TailwindNode, get_node_by_path, nodes_to_html, parse_html, remove_node_by_path};
use crate::app::views::design::import::import_html_as_elements;

fn get_node_mut_by_path<'a>(
    nodes: &'a mut [TailwindNode],
    path: &[usize],
) -> Option<&'a mut TailwindNode> {
    if path.is_empty() {
        return None;
    }

    let mut current = nodes.get_mut(path[0])?;
    for &idx in &path[1..] {
        current = current.children.get_mut(idx)?;
    }
    Some(current)
}

#[test]
fn mixed_content_round_trip_preserves_nested_text_spacing() {
    let html = "<p>Hello <span class=\"font-bold\">world</span> !</p>";

    let nodes = parse_html(html);
    assert_eq!(nodes_to_html(&nodes), html);
}

#[test]
fn parses_nested_nodes_into_stable_indented_html() {
    let html = "<div><section><span>Alpha</span><span>Beta</span></section><section><span>Gamma</span></section></div>";

    let nodes = parse_html(html);
    assert_eq!(
        nodes_to_html(&nodes),
        "<div>\n  <section>\n    <span>Alpha</span>\n    <span>Beta</span>\n  </section>\n  <section>\n    <span>Gamma</span>\n  </section>\n</div>"
    );
}

#[test]
fn serializes_void_and_empty_tags_stably() {
    let html = "<div><img src=\"hero.png\" alt=\"Hero\" /><p></p><br /></div>";

    let nodes = parse_html(html);
    assert_eq!(
        nodes_to_html(&nodes),
        "<div>\n  <img alt=\"Hero\" src=\"hero.png\" />\n  <p></p>\n  <br />\n</div>"
    );
}

#[test]
fn skips_comments_and_whitespace_only_text_nodes() {
    let html = "<div>\n  <!-- ignored -->\n  <span>Alpha</span>\n  \n  <span>Beta</span>\n</div>";

    let nodes = parse_html(html);
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].children.len(), 2);
    assert_eq!(nodes[0].children[0].tag, "span");
    assert_eq!(nodes[0].children[1].tag, "span");
    assert_eq!(nodes_to_html(&nodes), "<div>\n  <span>Alpha</span>\n  <span>Beta</span>\n</div>");
}

#[test]
fn serializes_attributes_in_sorted_order() {
    let html = "<div z=\"3\" a=\"1\" class=\"p-4\"></div>";

    let nodes = parse_html(html);
    assert_eq!(nodes_to_html(&nodes), "<div a=\"1\" class=\"p-4\" z=\"3\"></div>");
}

#[test]
fn multiple_roots_keep_stable_serialization() {
    let html = "<section></section>\n<section><span>Hi</span></section>";

    let nodes = parse_html(html);
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].tag, "section");
    assert_eq!(nodes[1].tag, "section");
    assert_eq!(
        nodes_to_html(&nodes),
        "<section></section>\n<section>\n  <span>Hi</span>\n</section>"
    );
}

#[test]
fn get_node_by_path_returns_expected_nested_nodes() {
    let nodes = parse_html(
        "<div><section id=\"left\"><span>Alpha</span><span>Beta</span></section><section id=\"right\"><span>Gamma</span></section></div>",
    );

    let branch = get_node_by_path(&nodes, &[0, 0]).expect("expected left section");
    assert_eq!(branch.attributes.get("id").map(String::as_str), Some("left"));

    let leaf = get_node_by_path(&nodes, &[0, 0, 1, 0]).expect("expected Beta text node");
    assert_eq!(leaf.text.as_deref(), Some("Beta"));

    assert!(get_node_by_path(&nodes, &[0, 3]).is_none());
    assert!(get_node_by_path(&nodes, &[]).is_none());
}

#[test]
fn remove_node_by_path_removes_only_target_branch() {
    let mut nodes = parse_html(
        "<div><section id=\"left\"><span>Alpha</span><span>Beta</span></section><section id=\"right\"><span>Gamma</span></section></div>",
    );

    assert!(remove_node_by_path(&mut nodes, &[0, 0, 1]));
    assert_eq!(
        nodes_to_html(&nodes),
        "<div>\n  <section id=\"left\">\n    <span>Alpha</span>\n  </section>\n  <section id=\"right\">\n    <span>Gamma</span>\n  </section>\n</div>"
    );

    let remaining = get_node_by_path(&nodes, &[0, 1, 0, 0]).expect("expected Gamma text node");
    assert_eq!(remaining.text.as_deref(), Some("Gamma"));
    assert!(!remove_node_by_path(&mut nodes, &[0, 9]));
}

#[test]
fn import_edit_export_round_trip_keeps_dom_paths_stable() {
    let html = "<div class=\"flex gap-2\"><p class=\"text-sm\">Hello</p><img src=\"hero.png\" alt=\"Hero\" /></div>";

    let imported = import_html_as_elements(html);
    assert_eq!(imported.len(), 1);
    assert_eq!(imported[0].class.as_deref(), Some("flex gap-2"));
    assert_eq!(imported[0].children.len(), 2);
    assert_eq!(imported[0].children[0].class.as_deref(), Some("text-sm"));
    assert_eq!(imported[0].children[0].children[0].content.as_deref(), Some("Hello"));
    assert_eq!(imported[0].children[1].kind, "image");
    assert_eq!(
        imported[0].children[1]
            .fill
            .as_ref()
            .and_then(|fill| fill.get("url"))
            .and_then(|url| url.as_str()),
        Some("hero.png")
    );

    let mut nodes = parse_html(html);
    let text_path = [0, 0, 0];
    let class_path = [0, 0];

    assert_eq!(
        get_node_by_path(&nodes, &text_path).and_then(|node| node.text.as_deref()),
        Some("Hello")
    );

    get_node_mut_by_path(&mut nodes, &class_path)
        .expect("expected paragraph node")
        .attributes
        .insert("class".to_string(), "font-bold text-lg".to_string());
    get_node_mut_by_path(&mut nodes, &text_path).expect("expected text node").text =
        Some("Updated".to_string());

    assert_eq!(
        nodes_to_html(&nodes),
        "<div class=\"flex gap-2\">\n  <p class=\"font-bold text-lg\">Updated</p>\n  <img alt=\"Hero\" src=\"hero.png\" />\n</div>"
    );
}
