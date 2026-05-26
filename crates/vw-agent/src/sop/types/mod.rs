use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

pub use vw_config_types::automation::SopExecutionMode;

// ── Priority ────────────────────────────────────────────────────

/// SOP priority level, used for execution mode resolution and scheduling.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SopPriority {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl fmt::Display for SopPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

// ── Trigger ─────────────────────────────────────────────────────

/// What event can activate an SOP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SopTrigger {
    Mqtt {
        topic: String,
        #[serde(default)]
        condition: Option<String>,
    },
    Webhook {
        path: String,
    },
    Cron {
        expression: String,
    },
    Manual,
}

impl fmt::Display for SopTrigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mqtt { topic, .. } => write!(f, "mqtt:{topic}"),
            Self::Webhook { path } => write!(f, "webhook:{path}"),
            Self::Cron { expression } => write!(f, "cron:{expression}"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

// ── Step ────────────────────────────────────────────────────────

/// A single step in an SOP procedure, parsed from SOP.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SopStep {
    pub number: u32,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub suggested_tools: Vec<String>,
    #[serde(default)]
    pub requires_confirmation: bool,
}

// ── SOP ─────────────────────────────────────────────────────────

/// A complete Standard Operating Procedure definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sop {
    pub name: String,
    pub description: String,
    pub version: String,
    pub priority: SopPriority,
    pub execution_mode: SopExecutionMode,
    pub triggers: Vec<SopTrigger>,
    pub steps: Vec<SopStep>,
    #[serde(default = "default_cooldown_secs")]
    pub cooldown_secs: u64,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
    #[serde(skip)]
    pub location: Option<PathBuf>,
}

fn default_cooldown_secs() -> u64 {
    0
}

fn default_max_concurrent() -> u32 {
    1
}

// ── TOML manifest (internal parse target) ───────────────────────

/// Top-level SOP.toml structure.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SopManifest {
    pub sop: SopMeta,
    #[serde(default)]
    pub triggers: Vec<SopTrigger>,
}

/// The `[sop]` table in SOP.toml.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SopMeta {
    pub name: String,
    pub description: String,
    #[serde(default = "default_sop_version")]
    pub version: String,
    #[serde(default)]
    pub priority: SopPriority,
    #[serde(default)]
    pub execution_mode: Option<SopExecutionMode>,
    #[serde(default = "default_cooldown_secs")]
    pub cooldown_secs: u64,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
}

fn default_sop_version() -> String {
    "0.1.0".to_string()
}

// ── Event ────────────────────────────────────────────────────────

/// The source type of an incoming event that may trigger an SOP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SopTriggerSource {
    Mqtt,
    Webhook,
    Cron,
    Manual,
}

impl fmt::Display for SopTriggerSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mqtt => write!(f, "mqtt"),
            Self::Webhook => write!(f, "webhook"),
            Self::Cron => write!(f, "cron"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

/// An incoming event that may trigger one or more SOPs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopEvent {
    pub source: SopTriggerSource,
    /// Topic, path, or signal identifier (depends on source type).
    #[serde(default)]
    pub topic: Option<String>,
    /// Raw payload (JSON string, sensor reading, etc.).
    #[serde(default)]
    pub payload: Option<String>,
    /// When the event occurred (ISO-8601).
    pub timestamp: String,
}

// ── Run state ────────────────────────────────────────────────────

/// Status of an SOP execution run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SopRunStatus {
    Pending,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for SopRunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::WaitingApproval => write!(f, "waiting_approval"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Result status of a single step execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SopStepStatus {
    Completed,
    Failed,
    Skipped,
}

impl fmt::Display for SopStepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

/// Result of executing a single SOP step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopStepResult {
    pub step_number: u32,
    pub status: SopStepStatus,
    pub output: String,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// A full SOP execution run (from trigger to completion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopRun {
    pub run_id: String,
    pub sop_name: String,
    pub trigger_event: SopEvent,
    pub status: SopRunStatus,
    pub current_step: u32,
    pub total_steps: u32,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub step_results: Vec<SopStepResult>,
    /// ISO-8601 timestamp when the run entered WaitingApproval (for timeout tracking).
    #[serde(default)]
    pub waiting_since: Option<String>,
}

/// What the engine instructs the caller to do next after a state transition.
#[derive(Debug, Clone)]
pub enum SopRunAction {
    /// Inject this step into the agent for execution.
    ExecuteStep { run_id: String, step: SopStep, context: String },
    /// Pause and wait for operator approval before executing this step.
    WaitApproval { run_id: String, step: SopStep, context: String },
    /// The SOP run completed successfully.
    Completed { run_id: String, sop_name: String },
    /// The SOP run failed.
    Failed { run_id: String, sop_name: String, reason: String },
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
