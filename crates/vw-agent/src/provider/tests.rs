#[test]
fn provider_compat_modules_are_reexported() {
    let _ = std::any::type_name::<super::auth::Method>();
}

