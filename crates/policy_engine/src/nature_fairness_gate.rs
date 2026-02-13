use serde::{Deserialize, Serialize};

/// Justice metrics already computed at Episode / corridor level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JusticeMetrics {
    pub erg: f32,
    pub erg_min_band: f32,
    pub tecr: f32,
    pub tecr_max_band: f32,
}

/// FateWindow / colonization-episode summary projected into diagnostics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FateWindowSummary {
    pub id: String,
    pub valid: bool,
    pub roh_max: f32,
    pub roh_ceiling: f32,       // usually 0.30 for governed humans
    pub lifeforce_min: f32,
    pub lifeforce_floor: f32,
    pub unfairdrain: bool,
    pub justice: JusticeMetrics,
}

/// Component-level evaluation of the NATURE.FAIRNESS corridor.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NatureFairnessComponents {
    pub roh_ok: bool,
    pub lifeforce_ok: bool,
    pub unfairdrain_ok: bool,
    pub erg_ok: bool,
    pub tecr_ok: bool,
}

/// Diagnostic evaluation result: no actuation, just evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatureFairnessEval {
    pub fatewindow_id: String,
    pub nature_fairness: bool,
    pub components: NatureFairnessComponents,
}

/// Pure predicate: evaluate the corridor over a FateWindow.
pub fn eval_nature_fairness(summary: &FateWindowSummary) -> NatureFairnessEval {
    let roh_ok =
        summary.valid && summary.roh_max <= summary.roh_ceiling;
    let lifeforce_ok =
        summary.lifeforce_min >= summary.lifeforce_floor;
    let unfairdrain_ok = !summary.unfairdrain;
    let erg_ok =
        summary.justice.erg >= summary.justice.erg_min_band;
    let tecr_ok =
        summary.justice.tecr <= summary.justice.tecr_max_band;

    let components = NatureFairnessComponents {
        roh_ok,
        lifeforce_ok,
        unfairdrain_ok,
        erg_ok,
        tecr_ok,
    };

    let nature_fairness =
        roh_ok && lifeforce_ok && unfairdrain_ok && erg_ok && tecr_ok;

    NatureFairnessEval {
        fatewindow_id: summary.id.clone(),
        nature_fairness,
        components,
    }
}

/// Read-only gate used by colonization policy compilers.
/// This function MUST NOT touch CapabilityState or devices. It simply
/// returns whether a neuromorphic colonization policy is certifiable
/// under the NATURE.FAIRNESS corridor.
pub fn policy_is_nature_fair(summary: &FateWindowSummary) -> bool {
    eval_nature_fairness(summary).nature_fairness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_passes_when_corridor_holds() {
        let summary = FateWindowSummary {
            id: "fw-colony-001".into(),
            valid: true,
            roh_max: 0.24,
            roh_ceiling: 0.30,
            lifeforce_min: 0.62,
            lifeforce_floor: 0.50,
            unfairdrain: false,
            justice: JusticeMetrics {
                erg: 0.72,
                erg_min_band: 0.60,
                tecr: 0.08,
                tecr_max_band: 0.15,
            },
        };
        assert!(policy_is_nature_fair(&summary));
    }

    #[test]
    fn policy_fails_on_unfairdrain_or_high_tecr() {
        let summary = FateWindowSummary {
            id: "fw-colony-002".into(),
            valid: true,
            roh_max: 0.27,
            roh_ceiling: 0.30,
            lifeforce_min: 0.55,
            lifeforce_floor: 0.50,
            unfairdrain: true, // unfair drain detected
            justice: JusticeMetrics {
                erg: 0.70,
                erg_min_band: 0.60,
                tecr: 0.25,      // collapse rate too high
                tecr_max_band: 0.15,
            },
        };
        let eval = eval_nature_fairness(&summary);
        assert!(!eval.components.unfairdrain_ok);
        assert!(!eval.components.tecr_ok);
        assert!(!eval.nature_fairness);
        assert!(!policy_is_nature_fair(&summary));
    }
}
