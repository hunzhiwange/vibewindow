#[cfg(not(target_arch = "wasm32"))]
#[test]
fn checksum_hex_is_stable_sha256_hex() {
    assert_eq!(
        super::checksum_hex("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
