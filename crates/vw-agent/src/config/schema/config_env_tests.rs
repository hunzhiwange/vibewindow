use super::{config_env::apply_env_overrides, Config};

#[test]
fn apply_env_overrides_has_stable_in_place_signature() {
    let apply: fn(&mut Config) = apply_env_overrides;

    let _ = apply;
}
