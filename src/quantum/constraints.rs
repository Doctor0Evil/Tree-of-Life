use crate::aln_core::{CapabilityState, JurisdictionTags};
use crate::roh_model::RoHScore;
use crate::fate_window::FateWindowId;

/// Static, read-only constraints for one synthesis run.
#[derive(Debug, Clone)]
pub struct QuantumSynthesisConstraints {
    pub roh_ceiling: f32,              // must be <= 0.30
    pub rod_budget: f32,               // < 1.0
    pub max_depth: u32,
    pub max_entangling_density: f32,   // 0.0–1.0
    pub max_meas_per_window: u32,
    pub roh_penalty_coeff: f32,
    pub rod_penalty_coeff: f32,
}

#[derive(Debug)]
pub enum SynthViolation {
    RoHCeilingWouldBeExceeded,
    RODBudgetWouldBeExceeded,
    DepthLimitExceeded,
    EntanglingDensityExceeded,
    MeasurementCadenceExceeded,
}

#[derive(Debug)]
pub struct ProjectedQuantumPlanMetrics {
    pub projected_roh_after: RoHScore, // normalized to 0.30
    pub projected_rod: f32,            // 0.0–1.0
    pub circuit_depth: u32,
    pub entangling_density: f32,
    pub meas_per_window: u32,
}

/// Pure helper; reads QUANTUM-SYNTHESIS-CONSTRAINTS shard, never writes.
pub fn load_constraints(
    cap: CapabilityState,
    juris: &JurisdictionTags,
    fate_window: FateWindowId,
    rod_budget: f32,
) -> QuantumSynthesisConstraints {
    // Implementation: decode the ALN shard row for this (cap, juris, fate_window),
    // apply synth_max_* formulas, and clamp to RoH ≤ 0.30, ROD < 1.0.
    unimplemented!()
}

/// Compile-time gate: called by the quantum circuit synthesizer
/// before committing a schedule.
pub fn check_plan_against_constraints(
    constraints: &QuantumSynthesisConstraints,
    metrics: &ProjectedQuantumPlanMetrics,
) -> Result<(), SynthViolation> {
    if metrics.projected_roh_after.value() > constraints.roh_ceiling {
        return Err(SynthViolation::RoHCeilingWouldBeExceeded);
    }
    if metrics.projected_rod >= constraints.rod_budget {
        return Err(SynthViolation::RODBudgetWouldBeExceeded);
    }
    if metrics.circuit_depth > constraints.max_depth {
        return Err(SynthViolation::DepthLimitExceeded);
    }
    if metrics.entangling_density > constraints.max_entangling_density {
        return Err(SynthViolation::EntanglingDensityExceeded);
    }
    if metrics.meas_per_window > constraints.max_meas_per_window {
        return Err(SynthViolation::MeasurementCadenceExceeded);
    }
    Ok(())
}
