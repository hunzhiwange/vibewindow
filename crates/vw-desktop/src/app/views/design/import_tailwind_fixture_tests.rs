//! 设计导入测试模块，验证 Tailwind 和通用导入路径生成稳定的设计元素树。

use super::import_html_as_elements;
use crate::app::views::design::canvas::tailwind::dom::{TailwindNode, parse_html};
use crate::app::views::design::export::generate_html;
use crate::app::views::design::models::{DesignDoc, DesignElement};

#[derive(Clone, Copy)]
enum DemoFixture {
    BasicLayoutText,
    MediaContent,
    IconSurface,
}

impl DemoFixture {
    fn label(self) -> &'static str {
        match self {
            Self::BasicLayoutText => "basic_layout_text",
            Self::MediaContent => "media_content",
            Self::IconSurface => "icon_surface",
        }
    }

    fn json(self) -> &'static str {
        match self {
            Self::BasicLayoutText => include_str!("../../../../../../assets/tailwind.json"),
            Self::MediaContent => include_str!("../../../../../../assets/tailwind3.json"),
            Self::IconSurface => include_str!("../../../../../../assets/tailwind4.json"),
        }
    }
}

fn load_fixture(fixture: DemoFixture) -> DesignDoc {
    serde_json::from_str(fixture.json())
        .unwrap_or_else(|error| panic!("fixture {} should deserialize: {error}", fixture.label()))
}

fn fixture_root_element(doc: &DesignDoc) -> &DesignElement {
    let element = doc.children.first().expect("fixture should contain one top-level element");
    assert!(element.kind.eq_ignore_ascii_case("tailwind"));
    element
}

fn fixture_html(doc: &DesignDoc) -> &str {
    fixture_root_element(doc)
        .content
        .as_deref()
        .expect("tailwind fixture should contain html content")
}

fn count_tag(nodes: &[TailwindNode], target: &str) -> usize {
    fn walk(node: &TailwindNode, target: &str, count: &mut usize) {
        if node.tag == target {
            *count += 1;
        }
        for child in &node.children {
            walk(child, target, count);
        }
    }

    let mut count = 0;
    for node in nodes {
        walk(node, target, &mut count);
    }
    count
}

fn any_class_contains(nodes: &[TailwindNode], needle: &str) -> bool {
    fn walk(node: &TailwindNode, needle: &str) -> bool {
        if node.attributes.get("class").is_some_and(|class| class.contains(needle)) {
            return true;
        }
        node.children.iter().any(|child| walk(child, needle))
    }

    nodes.iter().any(|node| walk(node, needle))
}

fn any_text_contains(nodes: &[TailwindNode], needle: &str) -> bool {
    fn walk(node: &TailwindNode, needle: &str) -> bool {
        if node.text.as_deref().is_some_and(|text| text.contains(needle)) {
            return true;
        }
        node.children.iter().any(|child| walk(child, needle))
    }

    nodes.iter().any(|node| walk(node, needle))
}

fn count_kind(elements: &[DesignElement], target: &str) -> usize {
    fn walk(element: &DesignElement, target: &str, count: &mut usize) {
        if element.kind == target {
            *count += 1;
        }
        for child in &element.children {
            walk(child, target, count);
        }
    }

    let mut count = 0;
    for element in elements {
        walk(element, target, &mut count);
    }
    count
}

fn any_element_text_contains(elements: &[DesignElement], needle: &str) -> bool {
    fn walk(element: &DesignElement, needle: &str) -> bool {
        if element.content.as_deref().is_some_and(|text| text.contains(needle)) {
            return true;
        }
        element.children.iter().any(|child| walk(child, needle))
    }

    elements.iter().any(|element| walk(element, needle))
}

fn collect_image_urls(elements: &[DesignElement]) -> Vec<String> {
    fn walk(element: &DesignElement, urls: &mut Vec<String>) {
        if element.kind == "image"
            && let Some(url) =
                element.fill.as_ref().and_then(|fill| fill.get("url")).and_then(|url| url.as_str())
        {
            urls.push(url.to_string());
        }
        for child in &element.children {
            walk(child, urls);
        }
    }

    let mut urls = Vec::new();
    for element in elements {
        walk(element, &mut urls);
    }
    urls
}

#[test]
fn basic_layout_text_fixture_regression_is_stable() {
    let doc = load_fixture(DemoFixture::BasicLayoutText);
    let html = fixture_html(&doc);
    let nodes = parse_html(html);

    assert_eq!(nodes.len(), 1);
    assert_eq!(count_tag(&nodes, "div"), 1);
    assert!(any_class_contains(&nodes, "flex flex-col items-center gap-6"));
    assert!(any_text_contains(&nodes, "我是tailwindcss"));

    let imported = import_html_as_elements(html);
    assert_eq!(imported.len(), 1);
    assert_eq!(count_kind(&imported, "frame"), 1);
    assert_eq!(count_kind(&imported, "text"), 1);
    assert!(any_element_text_contains(&imported, "我是tailwindcss"));
    assert!(
        imported[0]
            .class
            .as_deref()
            .is_some_and(|class| class.contains("rounded-2xl") && class.contains("bg-blue-100"))
    );

    let exported = generate_html(&doc);
    assert!(exported.contains(
        "flex flex-col items-center gap-6 p-7 md:flex-row md:gap-8 rounded-2xl bg-blue-100"
    ));
    assert!(exported.contains("我是tailwindcss"));
}

#[test]
fn media_content_fixture_regression_is_stable() {
    let doc = load_fixture(DemoFixture::MediaContent);
    let html = fixture_html(&doc);
    let nodes = parse_html(html);

    assert_eq!(count_tag(&nodes, "img"), 1);
    assert_eq!(count_tag(&nodes, "a"), 1);
    assert_eq!(count_tag(&nodes, "br"), 2);
    assert!(any_class_contains(&nodes, "object-cover object-center"));
    assert!(any_text_contains(&nodes, "Our competitive advantage"));

    let imported = import_html_as_elements(html);
    let image_urls = collect_image_urls(&imported);
    assert_eq!(image_urls.len(), 1);
    assert!(image_urls[0].contains("images.unsplash.com"));
    assert_eq!(count_kind(&imported, "image"), 1);
    assert!(count_kind(&imported, "text") >= 4);
    assert!(any_element_text_contains(&imported, "Our competitive advantage"));

    let exported = generate_html(&doc);
    assert!(exported.contains("https://images.unsplash.com/photo-1610465299996-30f240ac2b1c"));
    assert!(exported.contains("object-cover object-center"));
    assert!(exported.contains("Our competitive advantage"));
}

#[test]
fn icon_surface_fixture_regression_is_stable() {
    let doc = load_fixture(DemoFixture::IconSurface);
    let html = fixture_html(&doc);
    let nodes = parse_html(html);

    assert_eq!(count_tag(&nodes, "svg"), 7);
    assert_eq!(count_tag(&nodes, "path"), 7);
    assert_eq!(count_tag(&nodes, "button"), 1);
    assert!(any_class_contains(&nodes, "rounded-lg border bg-white shadow-sm"));
    assert!(any_text_contains(&nodes, "Enterprise solutions"));

    let imported = import_html_as_elements(html);
    assert_eq!(count_kind(&imported, "path"), 7);
    assert!(count_kind(&imported, "frame") >= 20);
    assert!(any_element_text_contains(&imported, "Enterprise solutions"));
    assert!(any_element_text_contains(&imported, "Sign up"));

    let exported = generate_html(&doc);
    assert!(exported.contains("rounded-lg border bg-white shadow-sm"));
    assert!(exported.contains("Enterprise solutions"));
    assert!(exported.contains("<svg"));
}
