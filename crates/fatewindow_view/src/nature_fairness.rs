use serde::{Deserialize, Serialize};

/// Justice metrics over an episode or FateWindow.
/// ERG and TECR are already computed elsewhere as bounded scalars.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JusticeMetrics {
    /// Environmental Resilience Grade, normalized (e.g. 0.0–1.0).
    pub erg: f32,
    /// Minimum acceptable ERG band for this scenario.
    pub erg_min_band: f32,
    /// Tree-of-Life Equity Ratio, normalized (e.g. 0.0–1.0).
    pub tecr: f32,
    /// Minimum acceptable TECR band for this scenario.
    pub tecr_min_band: f32,
}

/// FateWindow summary projected into the diagnostics layer.
/// This is a pure view: no capability or device handles appear here.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FateWindowSummary {
    /// Unique identifier for this FateWindow.
    pub id: String,
    /// Max Risk-of-Harm value observed in the window.
    pub roh_max: f32,
    /// True if any governed role crossed RoH > 0.30.
    pub roh_violation: bool,
    /// Minimum lifeforce across governed roles in the window.
    pub lifeforce_min: f32,
    /// Configured lifeforce floor for governed roles.
    pub lifeforce_floor_cfg: f32,
    /// True if UNFAIRDRAIN was detected for any protected role.
    pub unfairdrain: bool,
    /// Justice metrics over this window (ERG, TECR).
    pub justice: JusticeMetrics,
}

/// Component-wise view of NATURE.FAIRNESS for logging and audit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NatureFairnessComponents {
    pub roh_ok: bool,
    pub lifeforce_ok: bool,
    pub unfairdrain_ok: bool,
    pub erg_ok: bool,
    pub tecr_ok: bool,
}

/// Result of evaluating NATURE.FAIRNESS over a FateWindow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NatureFairnessEval {
    /// The conjunction predicate: true only when all sub-conditions hold.
    pub nature_fairness: bool,
    /// Component-wise booleans for forensic inspection.
    pub components: NatureFairnessComponents,
    /// FateWindow identifier to anchor in .evolve.jsonl / .donutloop.aln.
    pub fatewindow_id: String,
}

impl NatureFairnessEval {
    /// Evaluate NATURE.FAIRNESS over a FateWindow summary.
    ///
    /// This is a pure, observer-tier function: it reads state and emits booleans,
    /// with no capability writes, no envelope updates, and no device IO.
    pub fn from_fatewindow(summary: &FateWindowSummary) -> Self {
        let roh_ok = !summary.roh_violation && summary.roh_max <= 0.30;
        let lifeforce_ok = summary.lifeforce_min >= summary.lifeforce_floor_cfg;
        let unfairdrain_ok = !summary.unfairdrain;

        let erg_ok = summary.justice.erg >= summary.justice.erg_min_band;
        let tecr_ok = summary.justice.tecr >= summary.justice.tecr_min_band;

        let components = NatureFairnessComponents {
            roh_ok,
            lifeforce_ok,
            unfairdrain_ok,
            erg_ok,
            tecr_ok,
        };

        let nature_fairness =
            roh_ok && lifeforce_ok && unfairdrain_ok && erg_ok && tecr_ok;

        Self {
            nature_fairness,
            components,
            fatewindow_id: summary.id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_summary_ok() -> FateWindowSummary {
        FateWindowSummary {
            id: "fw-001".to_string(),
            roh_max: 0.21,
            roh_violation: false,
            lifeforce_min: 0.65,
            lifeforce_floor_cfg: 0.50,
            unfairdrain: false,
            justice: JusticeMetrics {
                erg: 0.72,
                erg_min_band: 0.60,
                tecr: 0.68,
                tecr_min_band: 0.55,
            },
        }
    }

    fn sample_summary_unfair() -> FateWindowSummary {
        FateWindowSummary {
            id: "fw-002".to_string(),
            roh_max: 0.29,
            roh_violation: false,
            lifeforce_min: 0.40,
            lifeforce_floor_cfg: 0.50,
            unfairdrain: true,
            justice: JusticeMetrics {
                erg: 0.80,
                erg_min_band: 0.60,
                tecr: 0.40,
                tecr_min_band: 0.55,
            },
        }
    }

    #[test]
    fn nature_fairness_holds_when_all_components_ok() {
        let summary = sample_summary_ok();
        let eval = NatureFairnessEval::from_fatewindow(&summary);

        assert!(eval.components.roh_ok);
        assert!(eval.components.lifeforce_ok);
        assert!(eval.components.unfairdrain_ok);
        assert!(eval.components.erg_ok);
        assert!(eval.components.tecr_ok);
        assert!(eval.nature_fairness);
    }

    #[test]
    fn nature_fairness_fails_on_any_component_violation() {
        let summary = sample_summary_unfair();
        let eval = NatureFairnessEval::from_fatewindow(&summary);

        // Here at least lifeforce_ok, unfairdrain_ok, and tecr_ok must fail.
        assert!(!eval.components.lifeforce_ok);
        assert!(!eval.components.unfairdrain_ok);
        assert!(!eval.components.tecr_ok);
        assert!(!eval.nature_fairness);
    }
}
