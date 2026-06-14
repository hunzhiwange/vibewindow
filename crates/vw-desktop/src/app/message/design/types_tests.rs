use super::{LayerAction, VariableKindPreset};

#[test]
fn layer_action_display_returns_toolbar_glyphs() {
    assert_eq!(LayerAction::ToggleVisible.to_string(), "👁");
    assert_eq!(LayerAction::MoveUp.to_string(), "↑");
    assert_eq!(LayerAction::MoveDown.to_string(), "↓");
    assert_eq!(LayerAction::Delete.to_string(), "🗑");
}

#[test]
fn variable_kind_preset_exposes_kind_label_prefix_and_default_value() {
    let cases = [
        (VariableKindPreset::Color, "color", "Color", "--color", "#D4D4D8"),
        (VariableKindPreset::Number, "number", "Number", "--number", "0"),
        (VariableKindPreset::String, "string", "String", "--string", "value"),
    ];

    for (preset, kind, label, prefix, value) in cases {
        assert_eq!(preset.as_kind(), kind);
        assert_eq!(preset.label(), label);
        assert_eq!(preset.default_name_prefix(), prefix);
        assert_eq!(preset.default_value(), value);
    }
}
