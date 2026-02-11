use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TreeAssets {
    pub life_force: f64,  // [0,1]
    pub smart: f64,       // [0,1]
    pub evolve: f64,      // [0,1]
    pub fear: f64,        // [0,1]
    pub reason: f64,      // [0,1] transparent reasoning depth
    pub forthgive: u32,   // count of shared acts (from logs)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Scenario {
    pub total_humans: u32,
    pub disagreeing: u32,
    pub reasoning: u32,
    pub hidden_concepts: bool,
    pub neuro_consent: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FairnessResult {
    pub satisfaction: f64,   // [0,1]
    pub unfair_drain: bool,  // true = unfair imbalance
    pub church_earn: u32,    // advisory token proposal
    pub mp_score: f64,       // [0,1] moral position
}

/// SATISFACTION: weighted average of TREE-like assets.
/// LIFEFORCE 0.3, SMART 0.2, EVOLVE 0.2, REASON 0.2, FORTHGIVE 0.1 (normalized).
pub fn sat_calc(assets: &TreeAssets) -> f64 {
    let forth_norm = (assets.forthgive as f64 / 100.0).clamp(0.0, 1.0);
    let sat = 0.3 * assets.life_force
        + 0.2 * assets.smart
        + 0.2 * assets.evolve
        + 0.2 * assets.reason
        + 0.1 * forth_norm;
    sat.clamp(0.0, 1.0)
}

/// True if there *is* an unfair drain (hidden power + no consent).
pub fn unfair_drain_predicate(scenario: &Scenario) -> bool {
    scenario.hidden_concepts && !scenario.neuro_consent
}

/// Fairness diagnostics over a pool of humans.
/// Pure: reads assets, returns advisory flags and aggregates.
pub fn analyze_fairness(
    scenario: &Scenario,
    assets_map: &HashMap<u32, TreeAssets>,
) -> FairnessResult {
    let unfair = unfair_drain_predicate(scenario);

    // Compute per-human SATISFACTION under two regimes:
    //  - unfair: apply conceptual penalties at the KO level (mp),
    //  - fair: credit FORTHGIVE via church_earn.
    let sats: Vec<(f64, u32)> = assets_map
        .par_iter()
        .map(|(_, assets)| {
            let mut church_delta = 0u32;
            let sat_assets = if unfair {
                TreeAssets {
                    fear: (assets.fear + 0.1).clamp(0.0, 1.0),
                    reason: (assets.reason - 0.15).clamp(0.0, 1.0),
                    ..assets.clone()
                }
            } else {
                church_delta = 1;
                TreeAssets {
                    forthgive: assets.forthgive.saturating_add(1),
                    ..assets.clone()
                }
            };
            (sat_calc(&sat_assets), church_delta)
        })
        .collect();

    let len = sats.len().max(1) as f64;
    let total_sat: f64 = sats.iter().map(|(s, _)| *s).sum::<f64>() / len;
    let church_earn: u32 = sats.iter().map(|(_, c)| *c).sum();

    let mp = if unfair { 0.4 } else { 0.8 };

    FairnessResult {
        satisfaction: total_sat,
        unfair_drain: unfair,
        church_earn,
        mp_score: mp,
    }
}
