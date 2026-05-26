use super::core::Agent;

#[test]
fn run_single_method_is_available_on_agent_type() {
    let method = Agent::run_single;

    let _ = method;
}
