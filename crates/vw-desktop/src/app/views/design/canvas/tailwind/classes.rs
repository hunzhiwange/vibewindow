//! 设计画布 Tailwind 支持模块。
//!
//! 该模块负责把 Tailwind 风格类名转换为画布渲染可用的结构化样式，供布局、形状和文本渲染路径复用。

use std::collections::HashSet;

use super::colors::TailwindColors;

const LAYOUT_CLASSES: &[&str] = &[
    "flex",
    "flex-row",
    "flex-row-reverse",
    "flex-col",
    "flex-col-reverse",
    "items-center",
    "items-start",
    "items-end",
    "justify-center",
    "justify-between",
    "justify-start",
    "justify-end",
    "grid",
    "block",
    "inline-block",
    "inline",
    "inline-flex",
    "hidden",
    "relative",
    "absolute",
];

const FLEX_ITEM_CLASSES: &[&str] =
    &["flex-1", "flex-auto", "flex-none", "grow", "grow-0", "shrink", "shrink-0"];

const TEXT_SIZE_CLASSES: &[&str] =
    &["text-sm", "text-base", "text-lg", "text-xl", "text-2xl", "text-3xl"];

const TEXT_WEIGHT_CLASSES: &[&str] = &["font-light", "font-normal", "font-semibold", "font-bold"];

const TEXT_ALIGN_CLASSES: &[&str] =
    &["text-left", "text-center", "text-right", "text-justify", "text-start", "text-end"];

const TEXT_STYLE_CLASSES: &[&str] = &[
    "italic",
    "not-italic",
    "underline",
    "line-through",
    "no-underline",
    "uppercase",
    "lowercase",
    "capitalize",
];

const OPACITY_CLASSES: &[&str] =
    &["opacity-0", "opacity-25", "opacity-50", "opacity-75", "opacity-100"];

fn extend_transform_classes(classes: &mut Vec<String>) {
    for scale in TailwindColors::SPACING_SCALE_TOKENS {
        classes.push(format!("translate-x-{}", scale));
        classes.push(format!("-translate-x-{}", scale));
        classes.push(format!("translate-y-{}", scale));
        classes.push(format!("-translate-y-{}", scale));
    }
}

const TRACKING_CLASSES: &[&str] = &[
    "tracking-tighter",
    "tracking-tight",
    "tracking-normal",
    "tracking-wide",
    "tracking-wider",
    "tracking-widest",
];

const LEADING_CLASSES: &[&str] = &[
    "leading-none",
    "leading-tight",
    "leading-snug",
    "leading-normal",
    "leading-relaxed",
    "leading-loose",
];

const BORDER_RADIUS_CLASSES: &[&str] = &[
    "rounded-none",
    "rounded-xs",
    "rounded-sm",
    "rounded",
    "rounded-md",
    "rounded-lg",
    "rounded-xl",
    "rounded-2xl",
    "rounded-3xl",
    "rounded-4xl",
    "rounded-full",
];

const BORDER_STYLE_CLASSES: &[&str] = &[
    "border-solid",
    "border-dashed",
    "border-dotted",
    "border-double",
    "border-hidden",
    "border-none",
];

const SHADOW_CLASSES: &[&str] = &["shadow-sm", "shadow", "shadow-md", "shadow-lg", "shadow-none"];

const OUTLINE_WIDTH_CLASSES: &[&str] =
    &["outline", "outline-0", "outline-1", "outline-2", "outline-4", "outline-8", "outline-none"];

const OUTLINE_STYLE_CLASSES: &[&str] =
    &["outline-solid", "outline-dashed", "outline-dotted", "outline-double"];

const OUTLINE_OFFSET_CLASSES: &[&str] = &[
    "outline-offset-0",
    "outline-offset-1",
    "outline-offset-2",
    "outline-offset-4",
    "outline-offset-8",
    "-outline-offset-1",
    "-outline-offset-2",
    "-outline-offset-4",
    "-outline-offset-8",
];

const BORDER_WIDTH_CLASSES: &[&str] = &[
    "border",
    "border-0",
    "border-2",
    "border-4",
    "border-8",
    "border-t",
    "border-r",
    "border-b",
    "border-l",
    "border-t-0",
    "border-r-0",
    "border-b-0",
    "border-l-0",
    "border-t-2",
    "border-r-2",
    "border-b-2",
    "border-l-2",
    "border-t-4",
    "border-r-4",
    "border-b-4",
    "border-l-4",
    "border-t-8",
    "border-r-8",
    "border-b-8",
    "border-l-8",
    "border-x",
    "border-y",
    "border-x-2",
    "border-y-2",
    "border-x-4",
    "border-y-4",
    "border-s",
    "border-e",
    "border-s-2",
    "border-e-2",
    "border-s-4",
    "border-e-4",
];

const DIVIDE_CLASSES: &[&str] = &[
    "divide-x",
    "divide-y",
    "divide-x-2",
    "divide-y-2",
    "divide-x-4",
    "divide-y-4",
    "divide-x-reverse",
    "divide-y-reverse",
];

const SIZE_SPECIAL_CLASSES: &[&str] = &[
    "w-full",
    "h-full",
    "w-screen",
    "h-screen",
    "w-auto",
    "h-auto",
    "max-w-screen-sm",
    "max-w-screen-md",
    "max-w-screen-lg",
    "max-w-screen-xl",
    "max-w-screen-2xl",
];

fn extend_position_classes(classes: &mut Vec<String>) {
    for scale in TailwindColors::SPACING_SCALE_TOKENS {
        for side in ["top", "right", "bottom", "left"] {
            classes.push(format!("{}-{}", side, scale));
            classes.push(format!("-{}-{}", side, scale));
        }
    }
}

fn extend_spacing_classes(classes: &mut Vec<String>) {
    for scale in TailwindColors::SPACING_SCALE_TOKENS {
        classes.push(format!("p-{}", scale));
        classes.push(format!("m-{}", scale));
        classes.push(format!("gap-{}", scale));
        classes.push(format!("gap-x-{}", scale));
        classes.push(format!("gap-y-{}", scale));

        for prefix in ["px", "py", "mx", "my", "pt", "pr", "pb", "pl", "mt", "mr", "mb", "ml"] {
            classes.push(format!("{}-{}", prefix, scale));
        }
    }

    classes.push("mx-auto".to_string());
    classes.push("my-auto".to_string());
}

fn extend_flex_item_classes(classes: &mut Vec<String>) {
    classes.extend(FLEX_ITEM_CLASSES.iter().map(|class_name| (*class_name).to_string()));

    for scale in TailwindColors::SPACING_SCALE_TOKENS {
        classes.push(format!("basis-{}", scale));
    }
}

fn extend_color_classes(classes: &mut Vec<String>) {
    for token in TailwindColors::TEXT_COLOR_TOKENS {
        classes.push(format!("text-{}", token));
    }
    for token in TailwindColors::BACKGROUND_COLOR_TOKENS {
        classes.push(format!("bg-{}", token));
    }
    for token in TailwindColors::BORDER_COLOR_TOKENS {
        classes.push(format!("border-{}", token));
        classes.push(format!("outline-{}", token));
    }
}

fn extend_size_classes(classes: &mut Vec<String>) {
    for scale in TailwindColors::SPACING_SCALE_TOKENS {
        classes.push(format!("w-{}", scale));
        classes.push(format!("h-{}", scale));
    }

    classes.extend(SIZE_SPECIAL_CLASSES.iter().map(|class_name| (*class_name).to_string()));
}

/// 公开的 get_tailwind_classes 函数。
///
/// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
pub fn get_tailwind_classes() -> Vec<String> {
    let mut classes = Vec::new();

    classes.extend(LAYOUT_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(
        TailwindColors::GRID_COLUMN_VALUES.iter().map(|columns| format!("grid-cols-{}", columns)),
    );

    extend_position_classes(&mut classes);
    extend_spacing_classes(&mut classes);
    extend_flex_item_classes(&mut classes);
    classes.extend(TEXT_SIZE_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(TEXT_WEIGHT_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(TEXT_ALIGN_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(TEXT_STYLE_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(OPACITY_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(TRACKING_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(LEADING_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    extend_transform_classes(&mut classes);
    extend_color_classes(&mut classes);
    classes.extend(BORDER_RADIUS_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(BORDER_STYLE_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(BORDER_WIDTH_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(DIVIDE_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(SHADOW_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(OUTLINE_WIDTH_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(OUTLINE_STYLE_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    classes.extend(OUTLINE_OFFSET_CLASSES.iter().map(|class_name| (*class_name).to_string()));
    extend_size_classes(&mut classes);

    let mut seen = HashSet::new();
    classes.retain(|class_name| seen.insert(class_name.clone()));

    classes
}

#[cfg(test)]
#[path = "classes_tests.rs"]
mod tests;
