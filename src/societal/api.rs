use crate::societal::engine::step_societal_state;
use crate::societal::model::{GovernanceEnvelope, SocialImpactVector, SocietalState, TechDomain, TechScenario};

#[derive(Debug)]
pub struct SimulationRequest {
    pub domain: TechDomain,
    pub initial_adoption: f32,
    pub rollout_speed: f32,
    pub centralization: f32,
    pub governance: GovernanceEnvelope,
    pub target_impact: SocialImpactVector,
    pub steps: usize,
}

#[derive(Debug)]
pub struct SimulationStep {
    pub step_index: usize,
    pub state: SocietalState,
    pub scenario: TechScenario,
    pub hex_stamp: String,
}

pub fn run_simulation(req: SimulationRequest) -> Vec<SimulationStep> {
    let mut results = Vec::with_capacity(req.steps);
    let mut state = SocietalState {
        stability: 0.6,
        unrest_risk: 0.2,
        fragility: 0.3,
        realized_impact: SocialImpactVector {
            antistigma: 0.6,
            nonexclusion: 0.6,
            peacekeeping: 0.6,
            eco: 0.6,
        },
    };

    let mut scenario = TechScenario {
        domain: req.domain,
        adoption_rate: req.initial_adoption.clamp(0.0, 1.0),
        rollout_speed: req.rollout_speed.clamp(0.0, 1.0),
        centralization: req.centralization.clamp(0.0, 1.0),
        governance: req.governance,
        target_impact: req.target_impact,
    };

    for step_index in 0..req.steps {
        let (next_state, next_scenario) = step_societal_state(&state, &scenario);
        let hex = next_state.hex_stamp();
        results.push(SimulationStep {
            step_index,
            state: next_state,
            scenario: next_scenario,
            hex_stamp: hex,
        });
        state = next_state;
        scenario = next_scenario;
    }

    results
}
