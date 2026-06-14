use super::icons::{
    acp_agent_icon, auto_icon, default_acp_icon, icon_svg, image_icon, is_dark_mode, max_icon,
    normalize_acp_agent_icon_name, provider_logo_handle, raw_svg_handle, svg_icon,
    themed_svg_handle,
};
use crate::app::Message;
use crate::app::assets::{self, Icon};
use iced::{Element, Length, Theme};

fn assert_size(element: &Element<'_, Message>, width: Length, height: Length) {
    let size = element.as_widget().size();

    assert_eq!(size.width, width);
    assert_eq!(size.height, height);
}

#[test]
fn icon_builders_use_requested_square_size() {
    let icon: Element<'_, Message> = icon_svg(Icon::Gear, 18.0).into();
    let themed: Element<'_, Message> = themed_svg_handle(auto_icon(), 19.0).into();
    let raw: Element<'_, Message> = raw_svg_handle(max_icon(), 20.0).into();
    let svg = svg_icon(Icon::Plus, 21.0);
    let image = image_icon(Icon::Logo, 22.0);

    assert_size(&icon, Length::Fixed(18.0), Length::Fixed(18.0));
    assert_size(&themed, Length::Fixed(19.0), Length::Fixed(19.0));
    assert_size(&raw, Length::Fixed(20.0), Length::Fixed(20.0));
    assert_size(&svg, Length::Fixed(21.0), Length::Fixed(21.0));
    assert_size(&image, Length::Fixed(22.0), Length::Fixed(22.0));
}

#[test]
fn provider_and_builtin_logo_handles_accept_known_and_unknown_ids() {
    let known: Element<'_, Message> =
        themed_svg_handle(provider_logo_handle("openai"), 12.0).into();
    let unknown: Element<'_, Message> =
        themed_svg_handle(provider_logo_handle("unknown-provider"), 13.0).into();

    assert_size(&known, Length::Fixed(12.0), Length::Fixed(12.0));
    assert_size(&unknown, Length::Fixed(13.0), Length::Fixed(13.0));
}

#[test]
fn theme_darkness_uses_palette_luminance_threshold() {
    assert!(is_dark_mode(&Theme::Dark));
    assert!(!is_dark_mode(&Theme::Light));
}

#[test]
fn acp_agent_icon_normalizes_supported_aliases() {
    let cases = [
        ("AgentClientProtocol-Claude", "claude"),
        (" Claude Code ", "claude"),
        ("Auggie CLI", "auggie"),
        ("Codex-CLI", "codex"),
        ("GitHub Copilot", "copilot"),
        ("Factory Droid", "droid"),
        ("Gemini CLI", "gemini"),
        ("Kiro Agent", "kiro"),
        ("KiloCode", "kilocode"),
        ("Kimi Code CLI", "kimi"),
        ("open-code", "opencode"),
        ("OpenClaw", "openclaw"),
        ("pi-acp", "pi"),
        ("Qoder CLI", "qoder"),
        ("Qwen Code", "qwen"),
        ("Trae CLI", "trae"),
        ("unknown", ""),
    ];

    for (input, expected) in cases {
        assert_eq!(normalize_acp_agent_icon_name(input), expected);
    }
}

#[test]
fn acp_agent_icon_renders_all_supported_agents_and_default() {
    for agent in [
        "auggie", "claude", "codex", "copilot", "cursor", "droid", "gemini", "kiro", "kilocode",
        "kimi", "opencode", "openclaw", "pi", "qoder", "qwen", "trae", "unknown",
    ] {
        let element = acp_agent_icon(agent, 16.0);

        assert_size(&element, Length::Fixed(16.0), Length::Fixed(16.0));
    }
}

#[test]
fn default_acp_icon_uses_requested_size() {
    let element = default_acp_icon(24.0);

    assert_size(&element, Length::Fixed(24.0), Length::Fixed(24.0));
}

#[test]
fn raw_svg_handle_accepts_asset_handle_without_theme_style() {
    let element: Element<'_, Message> =
        raw_svg_handle(assets::get_icon(Icon::AppCodex), 15.0).into();

    assert_size(&element, Length::Fixed(15.0), Length::Fixed(15.0));
}
