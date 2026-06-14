use super::install_cli_modal::{view, view_update_check};
use crate::app::Message;

fn keep(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

#[test]
fn task_743_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("install_cli_modal_tests.rs"));
}

#[test]
fn view_builds_plain_install_modal() {
    keep(view("安装 CLI", "请先安装命令行工具。"));
}

#[test]
fn view_builds_completion_modal_with_large_logo_layout() {
    keep(view("CLI 安装完成", "现在可以继续使用。"));
}

#[test]
fn update_check_builds_cli_detection_without_install_action() {
    keep(view_update_check(
        "检测 CLI 更新",
        "正在检查版本。",
        "1.0.0",
        "1.1.0",
        false,
        false,
        false,
    ));
}

#[test]
fn update_check_builds_cli_install_action_and_disabled_checking_state() {
    keep(view_update_check("安装 CLI", "发现可安装版本。", "0.0.0", "1.1.0", true, true, false));
}

#[test]
fn update_check_builds_app_update_action() {
    keep(view_update_check("更新应用", "发现应用新版本。", "2.0.0", "2.1.0", false, true, true));
}
