use serde::{Deserialize, Serialize};
use crate::model::{SiteView, Deed, DeedKind, NeurorightsBand};
use crate::justice::{JusticeSnapshot, JusticeMetrics};
use crate::hashlink::{HashStamp, hash_verdict};
use crate::time::Tick;
use crate::config::RightToExistConfig;

/// High‑level result of the corridor guard.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorridorDecision {
    Allow,
    Downscale,
    Deny,
}

/// Reason codes for audit and judgement.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CorridorReason {
    Ok,
    BioRailExceeded,
    BioLoadExceeded,
    ThermodynamicExceeded,
    NeurorightsViolated,
    JusticeBandBreached,
}

/// Summary of predicted scalars before/after the deed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorScalars {
    // 1D biosignature rail at locus (0..1), before/after.
    pub b_before: f64,
    pub b_after: f64,

    // Territorial bioload view (0..1), before/after.
    pub bioload_before: f64,
    pub bioload_after: f64,

    // Local thermodynamic envelope proxies.
    pub temp_before: f64,
    pub temp_after: f64,
    pub heart_rate_before: f64,
    pub heart_rate_after: f64,

    // Justice / neurorights projections.
    pub hpcc_before: f64,
    pub hpcc_after: f64,
    pub erg_before: f64,
    pub erg_after: f64,
    pub tecr_before: f64,
    pub tecr_after: f64,

    // Neurorights band flag before/after.
    pub neurorights_before: NeurorightsBand,
    pub neurorights_after: NeurorightsBand,
}

/// Single, hash‑linked verdict record for Jetson‑Line + Googolswarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorVerdict {
    pub tick: Tick,
    pub primary_site: u32,
    pub deed: DeedKind,
    pub decision: CorridorDecision,
    pub reason: CorridorReason,
    pub scalars: CorridorScalars,
    pub justice: JusticeSnapshot,
    pub hash: HashStamp,
}

/// Configuration of corridor ceilings and bands.
/// Non‑negotiable, doctrine‑backed limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorLimits {
    /// Max allowed local biosignature rail in this zone, strictly < 1.0.
    pub b_max: f64,
    /// Max allowed territorial bioload.
    pub bioload_max: f64,
    /// Thermodynamic envelope caps.
    pub temp_max: f64,
    pub heart_rate_max: f64,
    /// Justice bands.
    pub hpcc_max: f64,
    pub erg_max: f64,
    pub tecr_max: f64,
}

/// Right‑to‑exist corridor guard: main entry point.
///
/// (1) Reads SiteView + proposed Deed,
/// (2) Computes predicted scalars and neurorights bands,
/// (3) Enforces inequalities and non‑actuation rules,
/// (4) Emits hash‑linked verdict for Jetson‑Line + Googolswarm.
pub fn check_right_to_exist_corridor(
    tick: Tick,
    site: &SiteView,
    deed: &Deed,
    justice_metrics: &JusticeMetrics,
    cfg: &RightToExistConfig,
) -> CorridorVerdict {
    // 1. Compute predicted scalars from existing envelopes.
    let scalars = predict_scalars(site, deed, justice_metrics, cfg);

    // 2. Apply corridor inequalities in least‑restrictive order.
    let (decision, reason) = apply_corridor_rules(&scalars, deed, cfg);

    // 3. Snapshot justice context for this tick.
    let justice_snapshot = justice_metrics.snapshot_for_site(site.index);

    // 4. Hash‑link verdict for Googolswarm‑style PoO.
    let hash = hash_verdict(tick, site.index, deed.kind, &scalars, &justice_snapshot, decision, reason);

    CorridorVerdict {
        tick,
        primary_site: site.index,
        deed: deed.kind,
        decision,
        reason,
        scalars,
        justice: justice_snapshot,
        hash,
    }
}

/// Deterministic scalar prediction using existing 1D BioRail, BioLoad Terrasafe,
/// ThermodynamicEnvelope, and justice metrics.
/// This is a pure computation: no logging, no side‑effects.
fn predict_scalars(
    site: &SiteView,
    deed: &Deed,
    justice_metrics: &JusticeMetrics,
    cfg: &RightToExistConfig,
) -> CorridorScalars {
    // BioRail Scalar Gate: 1D biosignature rail at locus.
    let b_before = site.bio.biosignature1d;
    let b_delta = cfg.models.predict_b_delta(site, deed);
    let b_after = (b_before + b_delta).clamp(0.0, 1.0);

    // BioLoad Terrasafe Guard: territorial bioload.
    let bioload_before = site.territory_view.bioload;
    let bioload_delta = cfg.models.predict_bioload_delta(site, deed);
    let bioload_after = (bioload_before + bioload_delta).clamp(0.0, 1.0);

    // ThermodynamicEnvelope: temperature and heart‑rate proxies.
    let temp_before = site.thermo.local_temp;
    let temp_delta = cfg.models.predict_temp_delta(site, deed);
    let temp_after = temp_before + temp_delta;

    let heart_before = site.thermo.heart_rate;
    let heart_delta = cfg.models.predict_heart_delta(site, deed);
    let heart_after = heart_before + heart_delta;

    // Justice metrics (HPCC, ERG, TECR) from current episode projections.
    let (hpcc_before, erg_before, tecr_before) = justice_metrics.values_for_site(site.index);
    let (hpcc_delta, erg_delta, tecr_delta) = cfg.models.predict_justice_deltas(site, deed);
    let hpcc_after = hpcc_before + hpcc_delta;
    let erg_after = erg_before + erg_delta;
    let tecr_after = tecr_before + tecr_delta;

    // Neurorights bands before/after, derived from biosignature + RoH + context.
    let neurorights_before = cfg.models.neurorights_band(site, b_before);
    let neurorights_after = cfg.models.neurorights_band(site, b_after);

    CorridorScalars {
        b_before,
        b_after,
        bioload_before,
        bioload_after,
        temp_before,
        temp_after,
        heart_rate_before: heart_before,
        heart_rate_after: heart_after,
        hpcc_before,
        hpcc_after,
        erg_before,
        erg_after,
        tecr_before,
        tecr_after,
        neurorights_before,
        neurorights_after,
    }
}

/// Enforce corridor inequalities and non‑actuation rules.
///
/// Ordering is chosen to be maximally permissive under Tree‑of‑Life:
/// first try to treat as Allow, then Downscale, only then Deny.
fn apply_corridor_rules(
    s: &CorridorScalars,
    deed: &Deed,
    cfg: &RightToExistConfig,
) -> (CorridorDecision, CorridorReason) {
    let limits = cfg.limits_for_zone(deed.zone);

    // 1. Hard neurorights no‑actuation intervals (e.g., dreamstate).
    if cfg.neurorights.is_non_actuating_band(s.neurorights_after) {
        return (CorridorDecision::Deny, CorridorReason::NeurorightsViolated);
    }

    // 2. ThermodynamicEnvelope: ΔT and heart‑rate corridors.
    if s.temp_after > limits.temp_max || s.heart_rate_after > limits.heart_rate_max {
        // Attempt downscale if allowed by config.
        if cfg.downgrade.can_downscale(deed.kind) {
            return (CorridorDecision::Downscale, CorridorReason::ThermodynamicExceeded);
        }
        return (CorridorDecision::Deny, CorridorReason::ThermodynamicExceeded);
    }

    // 3. BioRail Scalar Gate: local biosignature corridor.
    if s.b_after > limits.b_max {
        if cfg.downgrade.can_downscale(deed.kind) {
            return (CorridorDecision::Downscale, CorridorReason::BioRailExceeded);
        }
        return (CorridorDecision::Deny, CorridorReason::BioRailExceeded);
    }

    // 4. BioLoad Terrasafe Guard: territorial bioload ceilings.
    if s.bioload_after > limits.bioload_max {
        if cfg.downgrade.can_downscale(deed.kind) {
            return (CorridorDecision::Downscale, CorridorReason::BioLoadExceeded);
        }
        return (CorridorDecision::Deny, CorridorReason::BioLoadExceeded);
    }

    // 5. Justice corridors (HPCC, ERG, TECR) as fairness guardrails.
    // These never relax safety ceilings; they only restrict additional harm
    // once structural injustice is detected.
    if s.hpcc_after > limits.hpcc_max
        || s.erg_after > limits.erg_max
        || s.tecr_after > limits.tecr_max
    {
        if cfg.downgrade.can_downscale(deed.kind) {
            return (CorridorDecision::Downscale, CorridorReason::JusticeBandBreached);
        }
        return (CorridorDecision::Deny, CorridorReason::JusticeBandBreached);
    }

    // If all corridors are satisfied, allow as‑is.
    (CorridorDecision::Allow, CorridorReason::Ok)
}
