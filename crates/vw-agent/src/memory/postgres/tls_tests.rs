use super::*;
use rustls::client::danger::ServerCertVerifier;
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use std::time::Duration;

#[test]
fn no_cert_verifier_advertises_signature_schemes() {
    let schemes = NoCertVerifier.supported_verify_schemes();
    assert!(!schemes.is_empty());
}

#[test]
fn no_cert_verifier_accepts_any_server_certificate() {
    let cert = CertificateDer::from(vec![0, 1, 2, 3]);
    let server_name = ServerName::try_from("postgres.example.test").unwrap();
    let now = UnixTime::since_unix_epoch(Duration::from_secs(1_700_000_000));

    let result = NoCertVerifier.verify_server_cert(&cert, &[], &server_name, &[], now);

    assert!(result.is_ok());
}
