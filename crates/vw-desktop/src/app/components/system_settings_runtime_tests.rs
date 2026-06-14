use super::*;
use iced::Element;
use iced::widget::text;

#[test]
fn labeled_option_displays_label_not_value() {
    let option = LabeledOption { value: "xhigh", label: "超高" };

    assert_eq!(option.to_string(), "超高");
}

#[test]
fn runtime_rows_accept_controls_and_callbacks() {
    let field: Element<'_, Message> = field_row("运行时类型", "选择执行环境。", text("native"));
    let input: Element<'_, Message> =
        text_row("镜像", "默认执行镜像。", "alpine:3.20", "ubuntu:24.04", |_| {
            Message::GatewayHealthTick
        });
    let hint: Element<'_, Message> = hint_row("建议仅放行明确需要挂载的根目录。");

    drop(field);
    drop(input);
    drop(hint);
}

#[test]
fn runtime_reasoning_options_cover_auto_enabled_disabled_and_levels() {
    let source = include_str!("system_settings_runtime.rs");

    assert!(source.contains("value: \"auto\", label: \"自动\""));
    assert!(source.contains("value: \"true\", label: \"启用\""));
    assert!(source.contains("value: \"false\", label: \"禁用\""));
    assert!(source.contains("value: \"minimal\", label: \"最小\""));
    assert!(source.contains("value: \"xhigh\", label: \"超高\""));
}
