//! 设计画布 Tailwind 支持模块。
//!
//! 该模块负责把 Tailwind 风格类名转换为画布渲染可用的结构化样式，供布局、形状和文本渲染路径复用。

use super::get_tailwind_classes;
use std::collections::HashSet;

#[test]
fn completions_prune_unsupported_tokens() {
    let classes = get_tailwind_classes();

    assert!(!classes.iter().any(|class_name| class_name == "fixed"));
    assert!(!classes.iter().any(|class_name| class_name == "sticky"));
    assert!(!classes.iter().any(|class_name| class_name == "col-span-2"));
    assert!(!classes.iter().any(|class_name| class_name == "row-span-2"));
    assert!(!classes.iter().any(|class_name| class_name == "col-start-2"));
    assert!(!classes.iter().any(|class_name| class_name == "row-start-2"));
    assert!(!classes.iter().any(|class_name| class_name == "grid-cols-7"));
    assert!(!classes.iter().any(|class_name| class_name == "text-xs"));
    assert!(!classes.iter().any(|class_name| class_name == "font-medium"));
}

#[test]
fn completions_include_supported_tailwind_tokens() {
    let classes = get_tailwind_classes();

    for expected in [
        "inline-flex",
        "flex-row-reverse",
        "flex-col-reverse",
        "flex-1",
        "grow",
        "shrink-0",
        "basis-8",
        "gap-y-10",
        "-left-4",
        "rounded-4xl",
        "border-solid",
        "border-indigo-600",
        "shadow-md",
        "outline-2",
        "outline-dashed",
        "outline-blue-500",
        "bg-green-500",
        "text-start",
    ] {
        assert!(classes.iter().any(|class_name| class_name == expected), "missing {expected}");
    }
}

#[test]
fn completions_are_unique() {
    let classes = get_tailwind_classes();
    let unique = classes.iter().collect::<HashSet<_>>();

    assert_eq!(classes.len(), unique.len());
}
