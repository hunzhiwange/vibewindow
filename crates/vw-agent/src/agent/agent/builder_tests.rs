use super::builder::AgentBuilder;

#[test]
fn build_reports_first_missing_required_dependency() {
    let error = match AgentBuilder::new().build() {
        Ok(_) => panic!("builder should reject missing required dependencies"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("tools are required"));
}
