use serde::{Deserialize, Serialize};

// Import your actual types; adjust paths to match your repo layout.
use crate::policy::capability::CapabilityState;
use crate::policy::roh::RoHScore;
use crate::biophysical::envelope::BiophysicalEnvelopeSnapshot;

/// Tree-of-Life scalar asset view (all values normalized 0.0–1.0).
///
/// This struct is pure state: it does not and MUST NOT own any authority over
/// capability, consent, or hardware. It is safe to log, visualize, and feed
/// into AI-chat explanation layers.
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

/// Per-snapshot diagnostic flags, strictly advisory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeOfLifeDiagnostics {
    /// Human-readable labels for AI-chat / HUD (e.g., "balanced", "overloaded").
    pub labels: Vec<String>,
    /// Whether the view suggests that a cooldown would be healthy.
    /// This is a suggestion ONLY and must NEVER be wired directly to capability transitions.
    pub cooldown_advised: bool,
    /// Whether fairness imbalances were observed between roles/subjects (if provided).
    pub fairness_imbalance: bool,
}

/// Input for computing a Tree-of-Life view from a single neuromorphic snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeOfLifeInput {
    pub capability_state: CapabilityState,
    pub roh_score: RoHScore,
    pub envelope: BiophysicalEnvelopeSnapshot,
    /// Optional per-session evolution index (e.g., from .evolve.jsonl).
    pub evolve_index: Option<u64>,
    /// Optional logical time index (e.g., epoch counter).
    pub epoch_index: Option<u64>,
}

/// Observer-only API for Tree-of-Life.
///
/// This module must remain side-effect free: no hardware access, no state
/// mutation outside its own structs, no capability/consent changes.
pub struct TreeOfLife;

impl TreeOfLife {
    /// Compute the Tree-of-Life asset view from an input snapshot.
    ///
    /// All mappings are bounded to [0.0, 1.0] and use only information that is
    /// already present in envelopes / RoH / capability state.
    pub fn from_snapshot(input: &TreeOfLifeInput) -> TreeOfLifeView {
        // BLOOD / OXYGEN: derived from HR/HRV envelopes if available.
        let (blood, oxygen) = Self::map_cardiovascular(&input.envelope);

        // WAVE: EEG bandpower + alpha-envelope CVE.
        let wave = Self::map_wave(&input.envelope);

        // H2O: placeholder until hydration/metabolic axes exist; keep neutral 0.5.
        let h2o = 0.5;

        // TIME / DECAY: bounded functions of epoch index and RoH drift.
        let (time, decay) = Self::map_time_decay(input.epoch_index, &input.roh_score);

        // LIFEFORCE: inverted RoH within your ≤0.3 envelope, capped to [0,1].
        let lifeforce = Self::map_lifeforce(&input.roh_score);

        // BRAIN / SMART / EVOLVE: capability tier + evidence of stable operation.
        let (brain, smart, evolve) =
            Self::map_cognition_and_evolution(input.capability_state, input.evolve_index);

        // POWER / TECH: intensity and complexity of active envelopes/modules.
        let (power, tech) = Self::map_power_and_tech(&input.envelope, input.capability_state);

        // FEAR / PAIN: combinations of EDA + HRV + motion WARN/RISK.
        let (fear, pain) = Self::map_fear_and_pain(&input.envelope);

        // NANO: normalized count of discrete logged events (if you map it later).
        let nano = Self::map_nano(input.evolve_index);

        TreeOfLifeView {
            blood,
            oxygen,
            wave,
            h2o,
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

    /// Produce advisory diagnostics from a view only.
    pub fn diagnostics(view: &TreeOfLifeView) -> TreeOfLifeDiagnostics {
        let mut labels = Vec::new();

        // Simple, explicit rules:
        if view.lifeforce > 0.7 && view.fear < 0.3 && view.pain < 0.3 {
            labels.push("balanced".to_string());
        }

        if view.fear > 0.6 || view.pain > 0.6 {
            labels.push("overloaded".to_string());
        }

        if view.decay > 0.7 {
            labels.push("cooldown-recommended".to_string());
        }

        let cooldown_advised = view.fear > 0.6 || view.pain > 0.6 || view.decay > 0.7;

        TreeOfLifeDiagnostics {
            labels,
            cooldown_advised,
            // Fairness analysis is computed at a higher level where multiple
            // subjects/roles are visible; here we leave it false by default.
            fairness_imbalance: false,
        }
    }

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

    fn map_cardiovascular(env: &BiophysicalEnvelopeSnapshot) -> (f32, f32) {
        // Example: map HR to BLOOD, HRV RMSSD to OXYGEN-equivalent reserve.
        let hr_norm = env.hr_bpm_normalized.unwrap_or(0.5);
        let rmssd_norm = env.hrv_rmssd_normalized.unwrap_or(0.5);

        let blood = Self::clamp01(hr_norm);
        let oxygen = Self::clamp01(rmssd_norm);

        (blood, oxygen)
    }

    fn map_wave(env: &BiophysicalEnvelopeSnapshot) -> f32 {
        // Combine EEG alpha/beta/gamma normalized values + alpha-CVE.
        let alpha = env.eeg_alpha_power_norm.unwrap_or(0.5);
        let beta = env.eeg_beta_power_norm.unwrap_or(0.5);
        let gamma = env.eeg_gamma_power_norm.unwrap_or(0.5);
        let alpha_cve = env.eeg_alpha_cve_norm.unwrap_or(0.5);

        let wave = (alpha + beta + gamma + alpha_cve) / 4.0;
        Self::clamp01(wave)
    }

    fn map_time_decay(epoch_index: Option<u64>, roh: &RoHScore) -> (f32, f32) {
        let time = epoch_index
            .map(|e| {
                // Map a 0–10_000 epoch window into 0.0–1.0, saturating beyond.
                let t = (e as f32) / 10_000.0;
                if t > 1.0 { 1.0 } else { t }
            })
            .unwrap_or(0.0);

        // Decay: how close we are to RoH ceiling (0.3).
        let roh_norm = (roh.value / 0.3).min(1.0).max(0.0);
        let decay = roh_norm;

        (time, decay)
    }

    fn map_lifeforce(roh: &RoHScore) -> f32 {
        // Within RoH ≤ 0.3, lifeforce = 1 - (RoH / 0.3); clamp in case of overshoot.
        let roh_norm = (roh.value / 0.3).min(1.0).max(0.0);
        let lf = 1.0 - roh_norm;
        Self::clamp01(lf)
    }

    fn map_cognition_and_evolution(
        cap: CapabilityState,
        evolve_index: Option<u64>,
    ) -> (f32, f32, f32) {
        let brain = match cap {
            CapabilityState::CapModelOnly => 0.25,
            CapabilityState::CapLabBench => 0.5,
            CapabilityState::CapControlledHuman => 0.75,
            CapabilityState::CapGeneralUse => 1.0,
        };

        // SMART: treat “higher state + evidence of stable operation” as smarter,
        // but keep bounded and purely descriptive.
        let evolve_norm = evolve_index
            .map(|i| (i as f32 / 10_000.0).min(1.0))
            .unwrap_or(0.0);
        let smart = Self::clamp01(0.5 * brain + 0.5 * evolve_norm);

        // EVOLVE: normalized index of how many safe evolution steps have been logged.
        let evolve = evolve_norm;

        (brain, smart, evolve)
    }

    fn map_power_and_tech(
        env: &BiophysicalEnvelopeSnapshot,
        cap: CapabilityState,
    ) -> (f32, f32) {
        // POWER: proportion of axes in WARN/RISK vs INFO.
        let info_frac = env.info_axis_fraction.unwrap_or(1.0);
        let warn_frac = env.warn_axis_fraction.unwrap_or(0.0);
        let risk_frac = env.risk_axis_fraction.unwrap_or(0.0);

        let power = Self::clamp01(0.4 * warn_frac + 0.6 * risk_frac);

        // TECH: complexity proxy = capability tier + number of active axes.
        let cap_score = match cap {
            CapabilityState::CapModelOnly => 0.25,
            CapabilityState::CapLabBench => 0.5,
            CapabilityState::CapControlledHuman => 0.75,
            CapabilityState::CapGeneralUse => 1.0,
        };

        let axis_count_norm = env.active_axis_count
            .map(|n| (n as f32 / 32.0).min(1.0))
            .unwrap_or(0.0);

        let tech = Self::clamp01(0.5 * cap_score + 0.5 * axis_count_norm);

        (power, tech)
    }

    fn map_fear_and_pain(env: &BiophysicalEnvelopeSnapshot) -> (f32, f32) {
        // FEAR: sympathetic arousal (EDA + HR) in WARN/RISK.
        let eda_warn = env.eda_warn_fraction.unwrap_or(0.0);
        let eda_risk = env.eda_risk_fraction.unwrap_or(0.0);
        let hr_warn = env.hr_warn_fraction.unwrap_or(0.0);
        let hr_risk = env.hr_risk_fraction.unwrap_or(0.0);

        let fear = Self::clamp01(0.4 * (eda_warn + hr_warn) + 0.6 * (eda_risk + hr_risk) / 2.0);

        // PAIN: combine FEAR-like channels with motion instability.
        let motion_warn = env.motion_warn_fraction.unwrap_or(0.0);
        let motion_risk = env.motion_risk_fraction.unwrap_or(0.0);

        let pain = Self::clamp01(0.5 * fear + 0.5 * (0.4 * motion_warn + 0.6 * motion_risk));

        (fear, pain)
    }

    fn map_nano(evolve_index: Option<u64>) -> f32 {
        // Map discrete evolution events to a 0–1 metric; purely informational.
        evolve_index
            .map(|i| (i as f32 / 100_000.0).min(1.0))
            .unwrap_or(0.0)
    }
}
