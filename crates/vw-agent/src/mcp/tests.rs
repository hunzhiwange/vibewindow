use super::*;
use crate::tools::ToolRuntimeContext;

#[test]
fn server_can_be_constructed_with_runtime_context() {
    let _server = AgentToolServer::new(ToolRuntimeContext::for_specs());
}
