use serde::{Deserialize, Serialize};
use aln_core::CapabilityState;
use biophysical::BiophysicalEnvelopeSnapshot; // your existing type

const ROH_CEILING: f32 = 0.30;

#[inline]
fn clamp01(x: f32) -> f32 {
    if x.is_nan() {
        0.0
    } else if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeOfLifeInput {
    pub capability_state: CapabilityState,
    pub roh_score: f32, // already enforced elsewhere
    pub epoch_index: u64,
    pub epoch_index_max: u64,
    pub evolve_index: Option<u64>,
    pub snapshot: BiophysicalEnvelopeSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeOfLifeView {
    pub blood: f32,
    pub oxygen: f32,
    pub wave: f32,
    pub h2o: f32,
    pub time: f32,
    pub decay: f32,
    pub lifeforce: f32,
    pub brain: f32,
    pub smart: f32,
    pub evolve: f32,
    pub power: f32,
    pub tech: f32,
    pub fear: f32,
    pub pain: f32,
    pub nano: f32,
}

impl TreeOfLifeView {
    pub fn from_input(input: &TreeOfLifeInput) -> Self {
        let s = &input.snapshot;

        // 1. Core normalization from envelope snapshot
        let hr_norm = clamp01((s.hrbpm_normalized.unwrap_or(0.0) - 0.0) / 1.0);
        let hrv_norm = clamp01((s.hrvrmssd_normalized.unwrap_or(0.0) - 0.0) / 1.0);
        let eeg_alpha_cve = clamp01(s.eeg_alpha_cve_normalized.unwrap_or(0.5));

        let wave = {
            let pa = clamp01(s.eeg_alpha_power_norm.unwrap_or(0.0));
            let pb = clamp01(s.eeg_beta_power_norm.unwrap_or(0.0));
            let pg = clamp01(s.eeg_gamma_power_norm.unwrap_or(0.0));
            clamp01((pa + pb + pg + eeg_alpha_cve) / 4.0)
        };

        // 2. RoH-derived DECAY/LIFEFORCE
        let decay = clamp01(input.roh_score / ROH_CEILING);
        let lifeforce = 1.0 - decay;

        // 3. TIME and NANO
        let time = if input.epoch_index_max > 0 {
            clamp01(input.epoch_index as f32 / input.epoch_index_max as f32)
        } else {
            0.0
        };

        let nano = match input.evolve_index {
            Some(idx) => {
                // normalize against a large constant; can be made configurable
                clamp01(idx as f32 / 100_000.0)
            }
            None => 0.0,
        };

        // 4. FEAR/PAIN from WARN/RISK fractions already computed in snapshot
        let fear = {
            let ew = clamp01(s.eda_warn_fraction.unwrap_or(0.0));
            let er = clamp01(s.eda_risk_fraction.unwrap_or(0.0));
            let hw = clamp01(s.hr_warn_fraction.unwrap_or(0.0));
            let hr = clamp01(s.hr_risk_fraction.unwrap_or(0.0));
            clamp01((ew + er + hw + hr) / 4.0)
        };

        let pain = {
            let mw = clamp01(s.motion_warn_fraction.unwrap_or(0.0));
            let mr = clamp01(s.motion_risk_fraction.unwrap_or(0.0));
            let motion_stress = (mw + mr) * 0.5;
            clamp01(0.5 * fear + 0.5 * motion_stress)
        };

        // 5. BRAIN/SMART/EVOLVE/POWER/TECH use existing lattice + counts
        let brain = match input.capability_state {
            CapabilityState::CapModelOnly => 0.25,
            CapabilityState::CapLabBench => 0.5,
            CapabilityState::CapControlledHuman => 0.75,
            CapabilityState::CapGeneralUse => 1.0,
        };

        let evolve = nano; // here: same index, different semantic label

        let smart = clamp01(0.5 * brain + 0.5 * evolve);

        let power = {
            let warn_frac = clamp01(s.warn_axis_fraction.unwrap_or(0.0));
            let risk_frac = clamp01(s.risk_axis_fraction.unwrap_or(0.0));
            clamp01(0.4 * warn_frac + 0.6 * risk_frac)
        };

        let tech = {
            let axis_count_norm = clamp01(s.active_axis_count.unwrap_or(0) as f32 / 32.0);
            clamp01(0.5 * brain + 0.5 * axis_count_norm)
        };

        TreeOfLifeView {
            blood: clamp01(1.0 - hr_norm),
            oxygen: hrv_norm,
            wave,
            h2o: 0.5, // neutral placeholder
            time,
            decay,
            lifeforce,
            brain,
            smart,
            evolve,
            power,
            tech,
            fear,
            pain,
            nano,
        }
    }
}

// Optional: local RoH guard (observer-only; should align with kernel checks)
pub fn roh_within_ceiling(roh_score: f32) -> bool {
    roh_score.is_finite() && roh_score >= 0.0 && roh_score <= ROH_CEILING
}
