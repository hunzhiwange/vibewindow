use super::proxied::proxied;

#[test]
fn proxied_reflects_proxy_environment_presence() {
    let before = proxied();
    unsafe { std::env::set_var("HTTP_PROXY", "http://127.0.0.1:1") };
    assert!(proxied());
    if !before {
        unsafe { std::env::remove_var("HTTP_PROXY") };
    }
}
