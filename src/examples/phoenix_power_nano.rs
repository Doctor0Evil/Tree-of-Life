use crate::societal::model::{
    GovernanceEnvelope, SocialImpactVector, SocietalState, TechDomain, TechScenario,
};
use crate::societal::engine::step_societal_state;

/// Phoenix 2026 profile bands, copied from transhumanprofile2026.aln.[file:16]
fn phoenix_wordmath_targets() -> (f32, f32, f32, f32, f32) {
    // target.y 0.15, target.z 0.10, target.T 0.05, target.K 0.85, target.E 0.85
    (0.15, 0.10, 0.05, 0.85, 0.85)
}

/// Social-impact target from auorgintegratedcitizenduties2026.[file:16]
fn phoenix_socialimpact_target() -> SocialImpactVector {
    // Santistigma 0.80, Snonexclusion 0.85, Speacekeeping 0.80, Seco 0.70
    SocialImpactVector {
        antistigma: 0.80,
        nonexclusion: 0.85,
        peacekeeping: 0.80,
        eco: 0.70,
    }
}

fn phoenix_governance_envelope() -> GovernanceEnvelope {
    let (y, z, t, k, e) = phoenix_wordmath_targets();
    GovernanceEnvelope {
        neurorights_alignment: 0.9,
        nonexclusion_enforced: 0.9,
        auditability: 0.85,
        y_repetition: y,
        z_drift: z,
        t_toxicity: t,
        k_kindness: k,
        e_evidentiality: e,
    }
}

/// Construct a Phoenix-2026 nano-upgrade scenario for the power grid.
pub fn phoenix_power_grid_nano_scenario_fast() -> (SocietalState, TechScenario) {
    let governance = phoenix_governance_envelope();
    let target_impact = phoenix_socialimpact_target();

    let initial_state = SocietalState {
        stability: 0.65,
        unrest_risk: 0.20,
        fragility: 0.35,
        realized_impact: target_impact,
    };

    let scenario = TechScenario {
        domain: TechDomain::Power,
        // 10% of grid nodes initially upgraded.
        adoption_rate: 0.10,
        // Fast rollout: aggressive deployment over 1–2 years.
        rollout_speed: 0.30,
        // Moderate centralization: some big utilities dominate.
        centralization: 0.60,
        governance,
        target_impact,
    };

    (initial_state, scenario)
}

pub fn phoenix_power_grid_nano_scenario_slow() -> (SocietalState, TechScenario) {
    let governance = phoenix_governance_envelope();
    let target_impact = phoenix_socialimpact_target();

    let initial_state = SocietalState {
        stability: 0.65,
        unrest_risk: 0.20,
        fragility: 0.35,
        realized_impact: target_impact,
    };

    let scenario = TechScenario {
        domain: TechDomain::Power,
        adoption_rate: 0.05,
        // Slow rollout: phased pilots and audits.
        rollout_speed: 0.10,
        // Slightly lower centralization due to community microgrids.
        centralization: 0.45,
        governance,
        target_impact,
    };

    (initial_state, scenario)
}

/// Run one scenario for N steps, returning a Vec of SocietalState for charting.
pub fn run_steps(
    mut state: SocietalState,
    mut scenario: TechScenario,
    steps: usize,
) -> Vec<SocietalState> {
    let mut out = Vec::with_capacity(steps);
    for _ in 0..steps {
        let (next_state, next_scenario) = step_societal_state(&state, &scenario);
        out.push(next_state);
        state = next_state;
        scenario = next_scenario;
    }
    out
}
