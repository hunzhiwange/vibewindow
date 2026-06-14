use super::*;
use iced::Element;
use iced::widget::{slider, text};

#[test]
fn labeled_option_displays_label_not_value() {
    let option = LabeledOption { value: "landlock", label: "Landlock" };

    assert_eq!(option.to_string(), "Landlock");
}

#[test]
fn sandbox_enabled_options_are_stable_and_user_facing() {
    let options = sandbox_enabled_options();
    let values: Vec<_> = options.iter().map(|option| option.value).collect();
    let labels: Vec<_> = options.iter().map(|option| option.label).collect();

    assert_eq!(values, ["auto", "true", "false"]);
    assert_eq!(labels, ["自动检测", "强制启用", "强制禁用"]);
}

#[test]
fn sandbox_backend_options_follow_enabled_policy() {
    let disabled_values: Vec<_> =
        sandbox_backend_options("false").iter().map(|option| option.value).collect();
    let forced_values: Vec<_> =
        sandbox_backend_options("true").iter().map(|option| option.value).collect();
    let auto_values: Vec<_> =
        sandbox_backend_options("auto").iter().map(|option| option.value).collect();

    assert_eq!(disabled_values, ["none"]);
    assert_eq!(forced_values, ["auto", "landlock", "firejail", "bubblewrap", "docker"]);
    assert_eq!(auto_values, ["auto", "landlock", "firejail", "bubblewrap", "docker", "none"]);
}

#[test]
fn sandbox_backend_description_covers_known_and_fallback_backends() {
    assert!(sandbox_backend_description("landlock").contains("Landlock"));
    assert!(sandbox_backend_description("firejail").contains("Firejail"));
    assert!(sandbox_backend_description("bubblewrap").contains("Bubblewrap"));
    assert!(sandbox_backend_description("docker").contains("Docker"));
    assert!(sandbox_backend_description("none").contains("不使用任何沙箱"));
    assert!(sandbox_backend_description("unknown").contains("自动选择"));
}

#[test]
fn common_rows_accept_security_controls() {
    let field: Element<'_, Message> = field_row("沙箱启用", "选择策略。", text("auto"));
    let input: Element<'_, Message> = text_row(
        "审计日志路径",
        "指定审计日志文件。",
        "audit.log",
        "audit.log",
        |_| Message::GatewayHealthTick,
    );
    let checkbox: Element<'_, Message> =
        bool_row("审计日志", "记录安全事件。", true, "开启", |_| {
            Message::GatewayHealthTick
        });
    let numeric: Element<'_, Message> = slider_row(
        "最大内存",
        "限制执行进程可使用的最大内存。",
        slider(32.0..=65_536.0, 512.0, |_| Message::GatewayHealthTick),
        "512 MB",
    );

    drop(field);
    drop(input);
    drop(checkbox);
    drop(numeric);
}
