use super::*;
use iced::Element;
use iced::widget::text;
use vw_config_types::automation::ResearchTrigger;

#[test]
fn trigger_label_maps_all_trigger_variants() {
    assert_eq!(trigger_label(ResearchTrigger::Never), "从不");
    assert_eq!(trigger_label(ResearchTrigger::Always), "总是");
    assert_eq!(trigger_label(ResearchTrigger::Keywords), "关键词");
    assert_eq!(trigger_label(ResearchTrigger::Length), "长度");
    assert_eq!(trigger_label(ResearchTrigger::Question), "问句");
}

#[test]
fn parse_trigger_label_maps_known_labels_and_defaults_to_never() {
    assert_eq!(parse_trigger_label("总是"), ResearchTrigger::Always);
    assert_eq!(parse_trigger_label("关键词"), ResearchTrigger::Keywords);
    assert_eq!(parse_trigger_label("长度"), ResearchTrigger::Length);
    assert_eq!(parse_trigger_label("问句"), ResearchTrigger::Question);
    assert_eq!(parse_trigger_label("从不"), ResearchTrigger::Never);
    assert_eq!(parse_trigger_label("unknown"), ResearchTrigger::Never);
}

#[test]
fn field_and_text_rows_accept_research_controls() {
    let field: Element<'_, Message> =
        field_row("触发方式", "定义进入 Research 阶段的条件。", text("从不"));
    let text: Element<'_, Message> = text_row(
        "关键词",
        "仅在触发方式为关键词时生效。",
        "find",
        "search",
        |_| Message::GatewayHealthTick,
    );

    drop(field);
    drop(text);
}
