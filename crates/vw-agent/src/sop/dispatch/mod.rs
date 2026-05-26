//! Unified SOP event dispatch helpers.
//!
//! All event sources (MQTT, webhook, cron, peripheral) route through
//! `dispatch_sop_event` so that locking, audit, and health bookkeeping
//! happen in exactly one place.

use std::sync::{Arc, Mutex};

use tracing::{debug, info, warn};

use super::audit::SopAuditLogger;
use super::engine::{SopEngine, now_iso8601};
use super::types::{SopEvent, SopRun, SopRunAction, SopTriggerSource};

// ── Dispatch result ─────────────────────────────────────────────

/// Outcome of attempting to dispatch an event to the SOP engine.
#[derive(Debug, Clone)]
pub enum DispatchResult {
    /// A new SOP run was started. `action` carries the next step the runtime
    /// must execute (or wait for approval on). Callers that cannot act on the
    /// action (e.g. headless fan-in) must still audit/log it — never silently
    /// drop.
    Started { run_id: String, sop_name: String, action: SopRunAction },
    /// A matching SOP was found but could not start (cooldown / concurrency).
    Skipped { sop_name: String, reason: String },
    /// No loaded SOP matched the event.
    NoMatch,
}

// ── Action helpers ──────────────────────────────────────────────

/// Extract the `run_id` from any `SopRunAction` variant.
fn extract_run_id_from_action(action: &SopRunAction) -> &str {
    match action {
        SopRunAction::ExecuteStep { run_id, .. }
        | SopRunAction::WaitApproval { run_id, .. }
        | SopRunAction::Completed { run_id, .. }
        | SopRunAction::Failed { run_id, .. } => run_id,
    }
}

/// Short label for logging which action was returned.
fn action_label(action: &SopRunAction) -> &'static str {
    match action {
        SopRunAction::ExecuteStep { .. } => "ExecuteStep",
        SopRunAction::WaitApproval { .. } => "WaitApproval",
        SopRunAction::Completed { .. } => "Completed",
        SopRunAction::Failed { .. } => "Failed",
    }
}

// ── Core dispatch ───────────────────────────────────────────────

/// Dispatch an incoming event to the SOP engine.
///
/// Pattern (batch lock — exactly 2 acquisitions):
/// 1. Lock → `match_trigger` → collect SOP names → drop lock
/// 2. Lock → for each name: `start_run` → collect results → drop lock
/// 3. Async (no lock): audit each started run
#[tracing::instrument(skip(engine, audit), fields(source = %event.source, topic = ?event.topic))]
pub async fn dispatch_sop_event(
    engine: &Arc<Mutex<SopEngine>>,
    audit: &SopAuditLogger,
    event: SopEvent,
) -> Vec<DispatchResult> {
    // Phase 1: match
    let matched_names: Vec<String> = match engine.lock() {
        Ok(eng) => eng.match_trigger(&event).iter().map(|s| s.name.clone()).collect(),
        Err(e) => {
            crate::app::agent::health::mark_component_error(
                "sop_dispatch",
                format!("lock poisoned: {e}"),
            );
            warn!("SOP dispatch: engine lock poisoned during match phase: {e}");
            return vec![];
        }
    };

    if matched_names.is_empty() {
        debug!("SOP dispatch: no match for event");
        return vec![DispatchResult::NoMatch];
    }

    info!("SOP dispatch: {} SOP(s) matched: {:?}", matched_names.len(), matched_names);

    // Phase 2: start runs
    let mut results = Vec::new();
    let mut started_runs: Vec<SopRun> = Vec::new();

    {
        let mut eng = match engine.lock() {
            Ok(e) => e,
            Err(e) => {
                crate::app::agent::health::mark_component_error(
                    "sop_dispatch",
                    format!("lock poisoned: {e}"),
                );
                warn!("SOP dispatch: engine lock poisoned during start phase: {e}");
                return vec![];
            }
        };

        for sop_name in &matched_names {
            match eng.start_run(sop_name, event.clone()) {
                Ok(action) => {
                    // Extract run_id from the action (authoritative source)
                    let run_id = extract_run_id_from_action(&action).to_string();
                    // Snapshot the run for audit (must be done under lock)
                    if let Some(run) = eng.active_runs().get(&run_id) {
                        started_runs.push(run.clone());
                    }
                    info!(
                        "SOP dispatch: started '{}' run {run_id} (action: {})",
                        sop_name,
                        action_label(&action),
                    );
                    results.push(DispatchResult::Started {
                        run_id,
                        sop_name: sop_name.clone(),
                        action,
                    });
                }
                Err(e) => {
                    info!("SOP dispatch: skipped '{}': {e}", sop_name);
                    results.push(DispatchResult::Skipped {
                        sop_name: sop_name.clone(),
                        reason: e.to_string(),
                    });
                }
            }
        }
    } // lock dropped

    // Phase 3: audit (async, no lock)
    for run in &started_runs {
        if let Err(e) = audit.log_run_start(run).await {
            warn!("SOP dispatch: audit log failed for run {}: {e}", run.run_id);
        }
    }

    crate::app::agent::health::mark_component_ok("sop_dispatch");
    results
}

// ── Headless result processing ──────────────────────────────────

/// Process dispatch results in headless (non-agent-loop) callers.
///
/// This handles audit and logging for fan-in callers (MQTT, webhook, cron)
/// that cannot execute SOP steps interactively. For `WaitApproval` actions,
/// approval timeout polling in the scheduler handles progression.
/// For `ExecuteStep` actions, the run is started in the engine but steps
/// cannot be executed without an agent loop — this is logged as a warning.
pub async fn process_headless_results(results: &[DispatchResult]) {
    for result in results {
        match result {
            DispatchResult::Started { run_id, sop_name, action } => match action {
                SopRunAction::ExecuteStep { step, .. } => {
                    warn!(
                        "SOP headless dispatch: run {run_id} ('{sop_name}') ready for step {} \
                         '{}' but no agent loop available to execute",
                        step.number, step.title,
                    );
                }
                SopRunAction::WaitApproval { step, .. } => {
                    info!(
                        "SOP headless dispatch: run {run_id} ('{sop_name}') waiting for approval \
                         on step {} '{}'. Timeout polling will handle progression",
                        step.number, step.title,
                    );
                }
                SopRunAction::Completed { .. } => {
                    info!(
                        "SOP headless dispatch: run {run_id} ('{sop_name}') completed immediately"
                    );
                }
                SopRunAction::Failed { reason, .. } => {
                    warn!("SOP headless dispatch: run {run_id} ('{sop_name}') failed: {reason}");
                }
            },
            DispatchResult::Skipped { sop_name, reason } => {
                info!("SOP headless dispatch: skipped '{sop_name}': {reason}");
            }
            DispatchResult::NoMatch => {}
        }
    }
}

// ── Cron SOP cache + check ──────────────────────────────────────

/// Pre-parsed cron schedules for SOP triggers.
///
/// Built once at daemon startup to avoid re-parsing cron expressions
/// on every scheduler tick.
#[derive(Clone)]
pub struct SopCronCache {
    /// (sop_name, raw_expression, parsed_schedule)
    schedules: Vec<(String, String, cron::Schedule)>,
}

impl SopCronCache {
    /// Build cache from the current engine state.
    ///
    /// Locks the engine once, iterates SOPs, parses Cron trigger expressions.
    /// Invalid expressions are logged and skipped (fail-closed).
    pub fn from_engine(engine: &Arc<Mutex<SopEngine>>) -> Self {
        let mut schedules = Vec::new();
        let eng = match engine.lock() {
            Ok(e) => e,
            Err(e) => {
                warn!("SopCronCache: engine lock poisoned: {e}");
                return Self { schedules };
            }
        };

        for sop in eng.sops() {
            for trigger in &sop.triggers {
                if let super::types::SopTrigger::Cron { expression } = trigger {
                    // Normalize 5-field crontab to 6-field (prepend seconds)
                    let normalized = match crate::app::agent::cron::normalize_expression(expression)
                    {
                        Ok(n) => n,
                        Err(e) => {
                            warn!(
                                "SopCronCache: invalid cron expression '{}' in SOP '{}': {e}",
                                expression, sop.name
                            );
                            continue;
                        }
                    };
                    match normalized.parse::<cron::Schedule>() {
                        Ok(schedule) => {
                            schedules.push((sop.name.clone(), expression.clone(), schedule));
                        }
                        Err(e) => {
                            warn!(
                                "SopCronCache: failed to parse cron schedule '{}' for SOP '{}': {e}",
                                normalized, sop.name
                            );
                        }
                    }
                }
            }
        }

        info!("SopCronCache: cached {} cron schedule(s)", schedules.len());
        Self { schedules }
    }

    /// Return the cached schedules (for testing).
    #[cfg(test)]
    pub fn schedules(&self) -> &[(String, String, cron::Schedule)] {
        &self.schedules
    }
}

/// Check all cached cron SOP triggers for firings in the window
/// `(last_check, now]` and dispatch events for each.
///
/// Uses window-based evaluation so ticks between polls are never missed.
pub async fn check_sop_cron_triggers(
    engine: &Arc<Mutex<SopEngine>>,
    audit: &SopAuditLogger,
    cache: &SopCronCache,
    last_check: &mut chrono::DateTime<chrono::Utc>,
) -> Vec<DispatchResult> {
    let now = chrono::Utc::now();
    let mut all_results = Vec::new();

    for (_sop_name, expression, schedule) in &cache.schedules {
        // Check if any occurrence fell in the window (last_check, now].
        // At-most-once semantics: even if multiple ticks of the same expression
        // fell in the window (e.g., scheduler delayed), we fire only once.
        // This is intentional — SOP triggers should not retroactively batch-fire.
        let mut upcoming = schedule.after(last_check);
        if let Some(next) = upcoming.next() {
            if next <= now {
                // This expression fired in the window
                let event = SopEvent {
                    source: SopTriggerSource::Cron,
                    topic: Some(expression.clone()),
                    payload: None,
                    timestamp: now_iso8601(),
                };
                let results = dispatch_sop_event(engine, audit, event).await;
                all_results.extend(results);
            }
        }
    }

    *last_check = now;
    all_results
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
