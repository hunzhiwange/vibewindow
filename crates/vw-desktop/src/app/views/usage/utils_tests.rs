use crate::app::Message;
use crate::app::assets::Icon;
use crate::app::message::view::ViewMessage;

#[test]
fn display_path_keeps_file_and_immediate_parent() {
    assert_eq!(super::display_path("/tmp/project/report.json"), "…/project/report.json");
    assert_eq!(super::display_path("README.md"), "README.md");
    assert_eq!(super::display_path("/README.md"), "README.md");
}

#[test]
fn fmt_ms_formats_epoch_minutes_and_rejects_overflow() {
    assert_eq!(super::fmt_ms(0), "1970-01-01 00:00");
    assert_eq!(super::fmt_ms(65_000), "1970-01-01 00:01");
    assert_eq!(super::fmt_ms(u64::MAX), "暂无");
}

#[test]
fn fmt_usd_rounds_to_four_decimals() {
    assert_eq!(super::fmt_usd(0.00123456), "US$0.0012");
    assert_eq!(super::fmt_usd(12.5), "US$12.5000");
}

#[test]
fn simple_usage_elements_can_be_constructed() {
    let _title = super::section_title("用量");
    let _kv = super::kv("总 token", "42".to_string());
    let _svg = super::icon_svg(Icon::Copy);
    let _btn = super::icon_btn(Icon::FolderOpen, "打开", Message::None);
}

#[test]
fn kv_path_handles_missing_and_present_paths() {
    let _missing = super::kv_path("快照", None);
    let _present = super::kv_path("快照", Some("/tmp/session/start.snap".to_string()));
    let _open_msg = Message::View(ViewMessage::OpenPathInFinder("/tmp/session".to_string()));
}
