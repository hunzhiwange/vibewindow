use super::*;
use rustls::client::danger::ServerCertVerifier;

#[test]
fn no_cert_verifier_advertises_signature_schemes() {
    let schemes = NoCertVerifier.supported_verify_schemes();
    assert!(!schemes.is_empty());
}
