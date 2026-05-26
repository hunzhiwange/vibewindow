use super::core::Agent;

#[test]
fn builder_returns_empty_builder_that_fails_without_required_parts() {
    let error = match Agent::builder().build() {
        Ok(_) => panic!("builder should reject missing required dependencies"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("tools are required"));
}
