#[test]
fn public_config_type_is_reexported() {
    let _ = std::any::type_name::<crate::Config>();
}
