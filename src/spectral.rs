// src/spectral.rs
// Non-actuating spectral diagnostics for the Jetson-Line.
// This module is ROLEDIAGNOSTICONLY and NOACTUATION by construction:
// - It never mutates WorldLine or ExtendedTokenState.
// - It only reads state and deed history to generate alerts.
//
// Use alongside tokens.rs and deeds.rs.

use serde::{Deserialize, Serialize};

use crate::deeds::{DeedType, ValidatedDeed, WorldLine};
use crate::tokens::ExtendedTokenState;

/// BEAST/PLAGUE and related diagnostic categories.
/// These are labels, not actuators.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpectralAlertKind {
    BeastPlague,
    FateWindow,
    TwistOfFate,
    WeatherCreation,
}

/// One spectral alert instance, derived from core tokens and/or deeds.
/// It is safe to log, display, or analyze, but must NOT drive actuation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralAlert {
    pub tick: u64,
    pub site_id: Option<u64>, // None for global alerts
    pub kind: SpectralAlertKind,
    pub severity: f64,        // 0–1 normalized, for UI prioritization
    pub message: String,      // human-readable description
}

/// Configuration for thresholds used by spectral diagnostics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpectralParams {
    // BEAST/PLAGUE-like thresholds
    pub death_high: f64,
    pub life_low: f64,
    pub decay_high: f64,
    pub lifeforce_low: f64,
    pub justice_low: f64,
    pub habit_high: f64,

    // FateWindow thresholds
    pub fear_low_band: f64,
    pub fear_high_band: f64,
    pub fear_window_length: u64,

    // TwistOfFate accumulation thresholds
    pub twistof_fate_window: u64,
    pub twistof_fate_severity_threshold: f64,
}

/// Immutable snapshot of spectral monitoring state.
/// The simulation engine can own and update this separately
/// by feeding it deed history and world snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralState {
    /// Rolling count of ticks where each site has been in BEAST/PLAGUE-like conditions.
    pub beast_duration: Vec<u64>,

    /// Rolling count of ticks where each site has had FEAR outside its safe band.
    pub fear_outside_band: Vec<u64>,

    /// Rolling accumulated "twist-of-fate" score per site.
    pub twistof_fate_score: Vec<f64>,
}

impl SpectralState {
    /// Initialize spectral state for a given number of sites.
    pub fn new(num_sites: usize) -> Self {
        Self {
            beast_duration: vec![0; num_sites],
            fear_outside_band: vec![0; num_sites],
            twistof_fate_score: vec![0.0; num_sites],
        }
    }
}

/// Spectral diagnostics engine: pure reader over WorldLine + ValidatedDeed history.
pub struct SpectralEngine {
    pub params: SpectralParams,
}

impl SpectralEngine {
    pub fn new(params: SpectralParams) -> Self {
        Self { params }
    }

    /// Evaluate spectral alerts for one tick, given:
    /// - current world snapshot (read-only),
    /// - spectral state from the previous tick,
    /// - deeds that occurred at this tick (for twist-of-fate heuristics).
    ///
    /// Returns:
    /// - updated SpectralState,
    /// - list of SpectralAlert objects (diagnostic only).
    pub fn evaluate_tick(
        &self,
        tick: u64,
        world: &WorldLine,
        prev_spec: &SpectralState,
        deeds_at_tick: &[ValidatedDeed],
    ) -> (SpectralState, Vec<SpectralAlert>) {
        let num_sites = world.len();
        let mut next_spec = prev_spec.clone();
        let mut alerts: Vec<SpectralAlert> = Vec::new();

        // 1. Per-site BEAST/PLAGUE and Fear-band tracking.
        for site_id in 0..num_sites {
            let st = match world.sites.get(site_id) {
                Some(s) => s,
                None => continue,
            };

            // BEAST/PLAGUE conditions derived from core tokens.
            let beast_like = self.is_beast_plague_state(st);

            // Update duration counters.
            if beast_like {
                next_spec.beast_duration[site_id] = next_spec.beast_duration[site_id].saturating_add(1);
            } else {
                next_spec.beast_duration[site_id] = 0;
            }

            // FEAR outside band tracking for FateWindows.
            let fear_outside =
                st.fear < self.params.fear_low_band || st.fear > self.params.fear_high_band;

            if fear_outside {
                next_spec.fear_outside_band[site_id] =
                    next_spec.fear_outside_band[site_id].saturating_add(1);
            } else {
                next_spec.fear_outside_band[site_id] = 0;
            }

            // Update twist-of-fate score heuristically.
            self.update_twistof_fate_score(site_id, st, &mut next_spec);

            // Emit BEAST/PLAGUE alert when conditions are severe enough.
            if beast_like {
                let severity = self.compute_beast_severity(st);
                if severity > 0.0 {
                    alerts.push(SpectralAlert {
                        tick,
                        site_id: Some(site_id as u64),
                        kind: SpectralAlertKind::BeastPlague,
                        severity,
                        message: format!(
                            "BEAST/PLAGUE-like conditions at site {}: death={}, life={}, decay={}, lifeforce={}, justice={}, habit={}",
                            site_id, st.death, st.life, st.decay, st.lifeforce, st.justice, st.habit
                        ),
                    });
                }
            }

            // Emit FateWindow alerts when FEAR out of band persists.
            if next_spec.fear_outside_band[site_id] >= self.params.fear_window_length {
                let severity = (next_spec.fear_outside_band[site_id] as f64
                    / self.params.fear_window_length as f64)
                    .min(1.0);
                alerts.push(SpectralAlert {
                    tick,
                    site_id: Some(site_id as u64),
                    kind: SpectralAlertKind::FateWindow,
                    severity,
                    message: format!(
                        "FateWindow at site {}: FEAR outside safe band for {} ticks (fear={})",
                        site_id,
                        next_spec.fear_outside_band[site_id],
                        st.fear
                    ),
                });
            }
        }

        // 2. Global twist-of-fate alerts based on accumulated scores and deeds.
        let twist_alert = self.evaluate_twistof_fate_global(tick, &next_spec, deeds_at_tick);
        if let Some(alert) = twist_alert {
            alerts.push(alert);
        }

        // 3. WeatherCreation-style early risk patterns (diagnostic only).
        let weather_alerts = self.evaluate_weather_creation(tick, world);
        alerts.extend(weather_alerts);

        (next_spec, alerts)
    }

    /// Check BEAST/PLAGUE-like state based on ExtendedTokenState.
    fn is_beast_plague_state(&self, st: &ExtendedTokenState) -> bool {
        let severe_death = st.death >= self.params.death_high;
        let exhausted_life = st.life <= self.params.life_low;
        let high_decay = st.decay >= self.params.decay_high;
        let low_lifeforce = st.lifeforce <= self.params.lifeforce_low;
        let low_justice = st.justice <= self.params.justice_low;
        let high_habit = st.habit >= self.params.habit_high;

        (severe_death && exhausted_life)
            || (high_decay && low_lifeforce)
            || (low_justice && high_habit)
    }

    /// Compute a normalized severity for BEAST/PLAGUE conditions.
    fn compute_beast_severity(&self, st: &ExtendedTokenState) -> f64 {
        let death_ratio = if self.params.death_high > 0.0 {
            (st.death / self.params.death_high).min(1.0)
        } else {
            0.0
        };
        let life_ratio = if self.params.life_low > 0.0 {
            // Invert: lower life => higher severity.
            (1.0 - (st.life / self.params.life_low).min(1.0)).max(0.0)
        } else {
            0.0
        };
        let decay_ratio = if self.params.decay_high > 0.0 {
            (st.decay / self.params.decay_high).min(1.0)
        } else {
            0.0
        };
        let lifeforce_ratio = if self.params.lifeforce_low > 0.0 {
            (1.0 - (st.lifeforce / self.params.lifeforce_low).min(1.0)).max(0.0)
        } else {
            0.0
        };
        let justice_ratio = if self.params.justice_low > 0.0 {
            (1.0 - (st.justice / self.params.justice_low).min(1.0)).max(0.0)
        } else {
            0.0
        };
        let habit_ratio = if self.params.habit_high > 0.0 {
            (st.habit / self.params.habit_high).min(1.0)
        } else {
            0.0
        };

        let sum = death_ratio
            + life_ratio
            + decay_ratio
            + lifeforce_ratio
            + justice_ratio
            + habit_ratio;
        (sum / 6.0).min(1.0)
    }

    /// Heuristic update of twist-of-fate score:
    /// - Increase when BEAST/PLAGUE-like tokens are present,
    /// - Slight decay over time otherwise.
    fn update_twistof_fate_score(
        &self,
        site_id: usize,
        st: &ExtendedTokenState,
        spec: &mut SpectralState,
    ) {
        let beast_like = self.is_beast_plague_state(st);
        if beast_like {
            // Increment with severity weight.
            let inc = self.compute_beast_severity(st) * 0.1;
            spec.twistof_fate_score[site_id] += inc;
        } else {
            // Gentle decay toward zero.
            spec.twistof_fate_score[site_id] *= 0.99;
        }
    }

    /// Aggregate per-site twist-of-fate scores and recent deeds to emit a
    /// global TwistOfFate alert when risk accumulates.
    fn evaluate_twistof_fate_global(
        &self,
        tick: u64,
        spec: &SpectralState,
        deeds_at_tick: &[ValidatedDeed],
    ) -> Option<SpectralAlert> {
        if spec.twistof_fate_score.is_empty() {
            return None;
        }
        let total: f64 = spec.twistof_fate_score.iter().copied().sum();
        let avg = total / (spec.twistof_fate_score.len() as f64);

        if avg < self.params.twistof_fate_severity_threshold {
            return None;
        }

        // Count recent destructive deeds as context (non-actuating).
        let mut harmful_deeds = 0usize;
        for d in deeds_at_tick {
            if matches!(
                d.original_type,
                DeedType::Conflict | DeedType::EmitPollution | DeedType::DeployTech
            ) && d.status != crate::deeds::DeedStatus::Blocked
            {
                harmful_deeds += 1;
            }
        }

        let severity = (avg / self.params.twistof_fate_severity_threshold).min(1.0);
        let msg = format!(
            "Global twist_of_fate accumulation: avg_score={:.3}, harmful_deeds_at_tick={}",
            avg, harmful_deeds
        );

        Some(SpectralAlert {
            tick,
            site_id: None,
            kind: SpectralAlertKind::TwistOfFate,
            severity,
            message: msg,
        })
    }

    /// Early risk pattern diagnostics ("WeatherCreation"):
    /// purely observational correlations of FEAR, BIOLOAD, EXPOSURE.
    fn evaluate_weather_creation(
        &self,
        tick: u64,
        world: &WorldLine,
    ) -> Vec<SpectralAlert> {
        let mut alerts = Vec::new();
        for (site_id, st) in world.sites.iter().enumerate() {
            // Simple heuristic: simultaneous high FEAR, BIOLOAD, EXPOSURE.
            let high_fear = st.fear > self.params.fear_high_band;
            let high_bioload = st.bioload > 0.5 * self.params.bioload_max_site;
            let high_exposure = st.exposure > 0.5;

            if high_fear && high_bioload && high_exposure {
                let severity = 0.5
                    + 0.5
                        * ((st.fear / self.params.fear_high_band)
                            + (st.bioload / self.params.bioload_max_site))
                        .min(1.0)
                        / 2.0;
                alerts.push(SpectralAlert {
                    tick,
                    site_id: Some(site_id as u64),
                    kind: SpectralAlertKind::WeatherCreation,
                    severity,
                    message: format!(
                        "WeatherCreation risk at site {}: fear={:.3}, bioload={:.3}, exposure={:.3}",
                        site_id, st.fear, st.bioload, st.exposure
                    ),
                });
            }
        }
        alerts
    }
}
