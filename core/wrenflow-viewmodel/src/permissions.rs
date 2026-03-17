//! Permission management — single source of truth for all permission states.
//!
//! One `PermissionManager` per app instance. Native UI observes the snapshot.
//! One polling timer drives `refresh()`. No scattered timers.

use std::collections::HashMap;
use std::sync::Arc;
use wrenflow_core::platform::{OsPermissionStatus, PermissionKind, PermissionState, PlatformHost};

/// Which permissions are required vs optional.
#[derive(Debug, Clone)]
pub struct PermissionRequirements {
    pub required: Vec<PermissionKind>,
    pub optional: Vec<PermissionKind>,
}

impl PermissionRequirements {
    /// All tracked permission kinds (required + optional).
    pub fn all_kinds(&self) -> Vec<PermissionKind> {
        let mut v = self.required.clone();
        v.extend(self.optional.iter());
        v
    }
}

/// Immutable snapshot of all permission states. Cheaply cloneable for UI binding.
#[derive(Debug, Clone, PartialEq)]
pub struct PermissionSnapshot {
    pub states: HashMap<PermissionKind, PermissionState>,
}

impl PermissionSnapshot {
    pub fn get(&self, kind: PermissionKind) -> PermissionState {
        self.states.get(&kind).copied().unwrap_or(PermissionState::Unknown)
    }

    pub fn all_required_satisfied(&self, reqs: &PermissionRequirements) -> bool {
        reqs.required.iter().all(|k| self.get(*k).is_satisfied())
    }

    pub fn missing_required(&self, reqs: &PermissionRequirements) -> Vec<PermissionKind> {
        reqs.required.iter().filter(|k| !self.get(**k).is_satisfied()).copied().collect()
    }
}

/// Single source of truth. Call `refresh()` on a timer (~1s).
pub struct PermissionManager {
    host: Arc<dyn PlatformHost>,
    states: HashMap<PermissionKind, PermissionState>,
    pub requirements: PermissionRequirements,
}

impl PermissionManager {
    pub fn new(host: Arc<dyn PlatformHost>, requirements: PermissionRequirements) -> Self {
        let mut states = HashMap::new();
        for kind in requirements.all_kinds() {
            states.insert(kind, PermissionState::Unknown);
        }
        Self { host, states, requirements }
    }

    pub fn snapshot(&self) -> PermissionSnapshot {
        PermissionSnapshot { states: self.states.clone() }
    }

    pub fn is_ready(&self) -> bool {
        self.snapshot().all_required_satisfied(&self.requirements)
    }

    /// Poll all permissions from the OS. Returns true if anything changed.
    pub fn refresh(&mut self) -> bool {
        let mut changed = false;
        for kind in self.states.keys().copied().collect::<Vec<_>>() {
            let os_status = self.host.get_permission(kind);
            let new_state = map_os_status(os_status);

            let old = self.states.get(&kind).copied();
            // Don't overwrite Requesting with NotGranted while the OS prompt is showing
            if old == Some(PermissionState::Requesting) && new_state == PermissionState::NotGranted {
                continue;
            }
            if old != Some(new_state) {
                self.states.insert(kind, new_state);
                changed = true;
            }
        }
        changed
    }

    /// User clicked "Grant Access". Transitions to Requesting, then calls OS.
    pub fn request(&mut self, kind: PermissionKind) {
        self.states.insert(kind, PermissionState::Requesting);
        self.host.request_permission(kind);
    }
}

fn map_os_status(status: OsPermissionStatus) -> PermissionState {
    match status {
        OsPermissionStatus::Granted => PermissionState::Granted,
        OsPermissionStatus::NotGranted => PermissionState::NotGranted,
        OsPermissionStatus::Denied => PermissionState::Denied,
        OsPermissionStatus::NotApplicable => PermissionState::NotApplicable,
    }
}

/// Top-level app readiness — derived from permission state + pipeline state.
#[derive(Debug, Clone, PartialEq)]
pub enum AppReadiness {
    /// First launch, wizard not completed.
    Setup,
    /// Required permissions missing.
    NeedsPermissions(Vec<PermissionKind>),
    /// Ready to record.
    Ready,
}

/// Pure function — computes readiness from current state.
pub fn compute_readiness(
    setup_completed: bool,
    snapshot: &PermissionSnapshot,
    requirements: &PermissionRequirements,
) -> AppReadiness {
    if !setup_completed {
        return AppReadiness::Setup;
    }
    let missing = snapshot.missing_required(requirements);
    if !missing.is_empty() {
        return AppReadiness::NeedsPermissions(missing);
    }
    AppReadiness::Ready
}
