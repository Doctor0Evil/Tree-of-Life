use crate::societal::model::{
    GovernanceEnvelope, SocialImpactVector, SocietalState, TechDomain, TechScenario,
};

fn domain_risk_weight(domain: TechDomain) -> f32 {
    match domain {
        TechDomain::Power => 1.0,   // grid collapse, weapons
        TechDomain::Nano  => 0.9,   // manufacturing, environment
        TechDomain::AI    => 0.8,
        TechDomain::Bio   => 0.85,
    }
}

/// Compute the realized impact vector for this step, given governance quality
/// and rollout characteristics.
fn realized_impact(scenario: &TechScenario) -> SocialImpactVector {
    let q = scenario.governance.quality_score();
    let centralization_penalty = scenario.centralization;
    let speed_penalty = scenario.rollout_speed.max(0.0);

    // Governance pulls us toward the target; centralization and speed pull away.
    // Weight governance more as adoption increases (early: narratives dominate,
    // later: structures dominate).
    let w_gov = (scenario.adoption_rate).clamp(0.0, 1.0);
    let mut impact = scenario.target_impact;

    // Pull down non‑exclusion and antistigma when centralization is high.
    impact.nonexclusion *= (1.0 - 0.6 * centralization_penalty);
    impact.antistigma *= (1.0 - 0.3 * centralization_penalty);

    // Very fast rollout with mediocre governance degrades peacekeeping.
    let rollout_stress = speed_penalty * (1.0 - q);
    impact.peacekeeping *= (1.0 - 0.7 * rollout_stress);

    // Eco impact: domain aware; power and nano can stress ecology if misgoverned.
    let eco_stress_factor = match scenario.domain {
        TechDomain::Power | TechDomain::Nano => 0.8,
        _ => 0.4,
    } * (1.0 - q);
    impact.eco *= (1.0 - eco_stress_factor);

    // Blend with a neutral “baseline society” impact to avoid extremes.
    let baseline = SocialImpactVector {
        antistigma: 0.6,
        nonexclusion: 0.6,
        peacekeeping: 0.6,
        eco: 0.6,
    };
    baseline.average(impact, 1.0 - w_gov)
}

/// One simulation tick: returns updated societal state and next scenario.
pub fn step_societal_state(
    current: &SocietalState,
    scenario: &TechScenario,
) -> (SocietalState, TechScenario) {
    let q = scenario.governance.quality_score();
    let domain_weight = domain_risk_weight(scenario.domain);
    let impact = realized_impact(scenario);
    let impact_scalar = impact.scalar();

    // “Pressure” from rollout speed and centralization.
    let rollout_pressure = scenario.rollout_speed * scenario.adoption_rate;
    let power_concentration = scenario.centralization * domain_weight;

    // Unrest risk grows when fast rollout + concentrated power + low governance quality.
    let unrest_delta = (rollout_pressure * power_concentration * (1.0 - q)).clamp(0.0, 1.0);

    // Stability grows with good governance & high realized impact; shrinks with unrest.
    let stability_delta =
        (impact_scalar * q * (1.0 - power_concentration)).clamp(0.0, 1.0) - unrest_delta * 0.7;

    let new_unrest = (current.unrest_risk + unrest_delta * 0.3).clamp(0.0, 1.0);
    let new_stability = (current.stability + stability_delta * 0.3).clamp(0.0, 1.0);

    // Fragility spikes when centralization is high and adoption is high.
    let frag_delta = (scenario.adoption_rate * power_concentration * (1.0 - q)).clamp(0.0, 1.0);
    let new_fragility = (current.fragility + frag_delta * 0.4).clamp(0.0, 1.0);

    let next_state = SocietalState {
        stability: new_stability,
        unrest_risk: new_unrest,
        fragility: new_fragility,
        realized_impact: impact,
    };

    // Advance adoption logistically; cap in [0,1].
    let next_adoption = (scenario.adoption_rate
        + scenario.rollout_speed * (1.0 - scenario.adoption_rate))
        .clamp(0.0, 1.0);

    let next_scenario = TechScenario {
        adoption_rate: next_adoption,
        ..*scenario
    };

    (next_state, next_scenario)
}
