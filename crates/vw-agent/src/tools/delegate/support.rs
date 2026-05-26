use super::super::traits::{Tool, ToolResult};
use crate::app::agent::observability::traits::{Observer, ObserverEvent, ObserverMetric};
use async_trait::async_trait;
use std::sync::Arc;

/// 工具 Arc 引用包装器
pub(super) struct ToolArcRef {
    inner: Arc<dyn Tool>,
}

impl ToolArcRef {
    pub(super) fn new(inner: Arc<dyn Tool>) -> Self {
        Self { inner }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for ToolArcRef {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters_schema(&self) -> serde_json::Value {
        self.inner.parameters_schema()
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        self.inner.execute(args).await
    }
}

/// 空操作观察器
pub(super) struct NoopObserver;

impl Observer for NoopObserver {
    fn record_event(&self, _event: &ObserverEvent) {}

    fn record_metric(&self, _metric: &ObserverMetric) {}

    fn name(&self) -> &str {
        "noop"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[cfg(test)]
#[path = "support_tests.rs"]
mod support_tests;
