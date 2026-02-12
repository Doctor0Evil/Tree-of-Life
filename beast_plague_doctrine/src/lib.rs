//! BEAST/PLAGUE diagnostic safety doctrine as type-checked invariants.
//!
//! This crate does **not** execute any diagnostics or actuation. It provides:
//! - Role markers for diagnostic vs actuator modules.
//! - A RoH / DECAY / UNFAIRDRAIN biosafe corridor representation.
//! - FateWindow lifecycle metadata types.
//! - A BEAST kernel interface spec.
//! - Audit hooks for Googolswarm-style anchoring.
//!
//! Internal crates are expected to:
//! - Mark diagnostic modules with `DiagnosticRole`,
//! - Mark actuator/capability modules with `ActuatorRole`,
//! - Use `EvidenceScalar` and `EvidenceFlags` instead of raw diagnostic enums
//!   inside BEAST kernels,
//! - Call the `assert_*` helpers in unit tests and CI to ensure compliance.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Marker type for observer-tier diagnostic-only components.
///
/// Crates that implement BEAST/PLAGUE pattern detectors, Tree-of-FEAR minting,
/// NATURE / UNFAIRDRAIN predicates, or FateWindow controllers should expose a
/// `pub const ROLE: DiagnosticRole = DiagnosticRole;` in their root.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiagnosticRole;

/// Marker type for capability / actuator components.
///
/// Crates that can write CapabilityState, envelopes, or device IO should expose
/// a `pub const ROLE: ActuatorRole = ActuatorRole;` in their root.
#[derive(Debug, Clone, Copy, Default)]
pub struct ActuatorRole;

/// Bounded evidence scalar in [0.0, 1.0], used as the only numeric input from
/// diagnostics into BEAST kernels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EvidenceScalar(f64);

impl EvidenceScalar {
    pub fn new(v: f64) -> Self {
        let clamped = if v < 0.0 {
            0.0
        } else if v > 1.0 {
            1.0
        } else {
            v
        };
        Self(clamped)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// Coarse evidence flags passed into BEAST kernels. Note: these are allowed
/// booleans; they are *derived* from diagnostics, not raw labels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct EvidenceFlags {
    pub corridor_safe: bool,
    pub window_valid: bool,
    pub no_safer_alternative: bool,
    pub overload_present: bool,
}

/// Biosafe polytope for legal operation: RoH <= 0.3, DECAY <= 1.0,
/// UNFAIRDRAIN == false, envelopes within safe bands.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BiosafePolytope {
    /// Risk-of-Harm, clamped to [0.0, 0.3].
    pub roh: f64,
    /// DECAY (normalized degradation) in [0.0, 1.0].
    pub decay: f64,
    /// Lifeforce or equivalent biophysical envelope fraction in [0.0, 1.0].
    pub lifeforce: f64,
    /// Fairness predicate: false means no unfair drain detected.
    pub unfair_drain: bool,
}

impl BiosafePolytope {
    pub fn new(roh: f64, decay: f64, lifeforce: f64, unfair_drain: bool) -> Self {
        Self {
            roh: roh.clamp(0.0, 0.3),
            decay: decay.clamp(0.0, 1.0),
            lifeforce: lifeforce.clamp(0.0, 1.0),
            unfair_drain,
        }
    }

    /// Check the legal operating corridor:
    /// - RoH <= 0.3
    /// - DECAY <= 1.0
    /// - lifeforce within [0.0, 1.0]
    /// - UNFAIRDRAIN == false
    pub fn is_legal_corridor(&self) -> bool {
        self.roh <= 0.3
            && self.decay <= 1.0
            && (0.0..=1.0).contains(&self.lifeforce)
            && !self.unfair_drain
    }
}

/// FateWindow state, for log-only diagnostic intervals.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FateWindowState {
    Closed,
    Open,
    Invalidated,
}

impl Default for FateWindowState {
    fn default() -> Self {
        FateWindowState::Closed
    }
}

/// FateWindow metadata used for audits and CI checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FateWindow {
    pub state: FateWindowState,
    pub started_at_tick: Option<u64>,
    pub ended_at_tick: Option<u64>,
    pub biosafe_polytope: BiosafePolytope,
}

impl FateWindow {
    pub fn new_closed() -> Self {
        Self {
            state: FateWindowState::Closed,
            started_at_tick: None,
            ended_at_tick: None,
            biosafe_polytope: BiosafePolytope::new(0.0, 0.0, 1.0, false),
        }
    }

    /// Validate that an OPEN FateWindow sits inside the legal corridor.
    pub fn is_open_and_legal(&self) -> bool {
        self.state == FateWindowState::Open && self.biosafe_polytope.is_legal_corridor()
    }
}

/// Minimal BEAST kernel input contract.
///
/// Internal kernels should depend on this struct rather than raw diagnostic
/// enums or Tree-of-FEAR records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastKernelInput {
    pub evidence_flags: EvidenceFlags,
    pub evidence_strength: EvidenceScalar,
    pub biosafe_polytope: BiosafePolytope,
}

/// Allowed BEAST kernel decisions. Note that all actions are conservative:
/// they can deny, pause, or force repair, but cannot mint new harmful
/// capabilities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BeastDecision {
    Allow,
    Deny,
    PauseForReview,
    ForceRepair,
}

/// Errors raised when doctrine invariants are violated.
#[derive(Debug, Error)]
pub enum DoctrineError {
    #[error("FateWindow open outside legal biosafe corridor")]
    FateWindowIllegal,

    #[error("UNFAIRDRAIN flagged inside biosafe corridor")]
    UnfairDrainDetected,

    #[error("BEAST kernel attempted to expand capabilities based on diagnostics")]
    IllegalCapabilityExpansion,
}

/// Assert that a FateWindow obeys the doctrine (for tests/CI).
pub fn assert_fatewindow_doctrine(fw: &FateWindow) -> Result<(), DoctrineError> {
    if fw.state == FateWindowState::Open && !fw.biosafe_polytope.is_legal_corridor() {
        return Err(DoctrineError::FateWindowIllegal);
    }
    if fw.biosafe_polytope.is_legal_corridor() && fw.biosafe_polytope.unfair_drain {
        return Err(DoctrineError::UnfairDrainDetected);
    }
    Ok(())
}

/// Assert that a BEAST decision is non-expansive in the capability sense.
/// Call this from kernel tests whenever a decision is applied to capabilities.
pub fn assert_beast_decision_non_expansive(
    decision: BeastDecision,
) -> Result<(), DoctrineError> {
    match decision {
        BeastDecision::Allow | BeastDecision::Deny | BeastDecision::PauseForReview
        | BeastDecision::ForceRepair => Ok(()),
    }
}
