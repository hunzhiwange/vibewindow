use super::*;
use serde_json::json;

#[test]
fn patch_serializes_hash_and_files() {
    let patch = Patch { hash: "abc123".to_string(), files: vec!["/tmp/file.rs".to_string()] };

    let value = serde_json::to_value(&patch).unwrap();
    assert_eq!(value["hash"], "abc123");
    assert_eq!(value["files"][0], "/tmp/file.rs");
}

#[test]
fn errors_display_underlying_messages() {
    let io = Error::from(std::io::Error::other("disk said no"));
    assert_eq!(io.to_string(), "disk said no");

    let utf8 = Error::from(String::from_utf8(vec![0xff]).unwrap_err());
    assert!(utf8.to_string().contains("invalid utf-8"));

    let git = Error::Git("git failed".to_string());
    assert_eq!(git.to_string(), "git failed");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn extra_helpers_include_base_fields_and_process_output() {
    let base = extra([("hash", json!("abc")), ("count", json!(2))]);
    assert_eq!(base["hash"], "abc");
    assert_eq!(base["count"], 2);

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("printf stdout; printf stderr >&2; exit 7")
        .output()
        .unwrap();
    let fields = extra_from_output(&output, [("op", json!("diff"))]);

    assert_eq!(fields["op"], "diff");
    assert_eq!(fields["exit_code"], 7);
    assert_eq!(fields["stdout"], "stdout");
    assert_eq!(fields["stderr"], "stderr");
}
