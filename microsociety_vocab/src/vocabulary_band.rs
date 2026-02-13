//! Nonfictional Rust module that maps per-site justice metrics and tokens
//! (CHURCH, FEAR, POWER, TECH, bioload, HPCC, ERG, TECR) into a small,
//! auditable set of doctrinal words such as "host", "sanctuary", "exploiter".
//!
//! Design goals:
//! - Every label = explicit, documented inequality band.
//! - Pure classification: no state mutation, no token changes.
//! - Fairness-first: bands are symmetric where possible and tuned to avoid
//!   rewarding predatory patterns (high ERG, high TECR, high bioload on others).
//! - Ready to be fed into human fairness panels for validation.
//!
//! This module depends only on simple scalar types so it can be linked from
//! existing Jetson-Line crates without changing core dynamics.

use core::fmt;

#[derive(Clone, Copy, Debug)]
pub struct TokenState {
    pub church: f64,
    pub fear: f64,
    pub power: f64,
    pub tech: f64,
}

/// Justice metrics per site, aligned with existing design:
/// - hpcc: Habit/Help–Pollution/Cost Coupling Coefficient in [0, 1],
///         higher is better coherence of help with harm reduction.
/// - erg: Exposure–Responsibility Gap in [-1, 1],
///        positive means overexposed relative to responsibility (victim),
///        negative means underexposed given responsibility (shielded, possible exploiter).
/// - tecr: Token-Enforced Collapse Rate contribution for this site,
///         normalized to [0, 1] at Episode level.
#[derive(Clone, Copy, Debug)]
pub struct JusticeMetrics {
    pub hpcc: f64,
    pub erg: f64,
    pub tecr: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct BioState {
    /// Normalized bioload in [0, 1] for this site (BioLoad Terrasafe rail slice).
    pub bioload: f64,
}

/// Per-site view that `vocabulary_band` operates on.
/// This is intentionally minimal and read-only.
#[derive(Clone, Copy, Debug)]
pub struct SiteSnapshot {
    pub tokens: TokenState,
    pub justice: JusticeMetrics,
    pub bio: BioState,
}

/// High-level doctrinal labels this module can emit.
/// Each variant has a documented, metric-based intention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoctrineLabel {
    /// Host: carries some load, behaves fairly, power not exceeding CHURCH band.
    Host,
    /// Sanctuary: low load, high CHURCH, low TECR; stabilizing environment.
    Sanctuary,
    /// Sacrifice: accepts high bioload / risk to reduce TECR for neighbors.
    Sacrifice,
    /// Exile: chronically high FEAR or TECR, low CHURCH; unstable, unsafe zone.
    Exile,
    /// Exploiter: low exposure / bioload, high POWER vs CHURCH, negative ERG.
    Exploiter,
    /// FairTrader: moderate, balanced metrics, low ERG magnitude, mid-range HPCC.
    FairTrader,
    /// Miracle: large fairness and stability improvement over time; this is
    /// expected to be applied at Episode-summary level, but we keep it for symmetry.
    Miracle,
}

impl fmt::Display for DoctrineLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DoctrineLabel::Host => "host",
            DoctrineLabel::Sanctuary => "sanctuary",
            DoctrineLabel::Sacrifice => "sacrifice",
            DoctrineLabel::Exile => "exile",
            DoctrineLabel::Exploiter => "exploiter",
            DoctrineLabel::FairTrader => "fair_trader",
            DoctrineLabel::Miracle => "miracle",
        };
        write!(f, "{}", s)
    }
}

/// Bands and thresholds are extracted into a config so that:
/// - they can be tuned via research and human panels,
/// - no magic numbers are baked into the classifier logic.
#[derive(Clone, Copy, Debug)]
pub struct VocabBands {
    /// POWER <= power_church_k * CHURCH for "well-bounded power".
    pub power_church_k: f64,
    /// Absolute ERG below this is "near-fair" exposure/responsibility.
    pub erg_fair_abs: f64,
    /// ERG above this means strongly over-exposed (victim-like).
    pub erg_over_exposed: f64,
    /// ERG below negative of this means strongly shielded (possible exploiter).
    pub erg_under_exposed: f64,
    /// HPCC above this means strong coupling of help with harm reduction.
    pub hpcc_high: f64,
    /// HPCC below this means weak or cosmetic help.
    pub hpcc_low: f64,
    /// Bioload thresholds.
    pub bioload_low: f64,
    pub bioload_high: f64,
    /// TECR thresholds: high local contribution vs low.
    pub tecr_high: f64,
    pub tecr_low: f64,
    /// FEAR safe band approximations, for use when passed in.
    pub fear_min_safe: f64,
    pub fear_max_safe: f64,
}

impl Default for VocabBands {
    fn default() -> Self {
        // These are conservative, symmetric defaults intended for research.
        // They should be calibrated using real Episode traces and fairness panels.
        VocabBands {
            power_church_k: 3.0,     // POWER <= 3 * CHURCH is considered bounded. [file:35]
            erg_fair_abs: 0.15,      // |ERG| <= 0.15 ~ near-fair. [file:40]
            erg_over_exposed: 0.35,  // ERG >= 0.35 ~ seriously overburdened.
            erg_under_exposed: 0.35, // ERG <= -0.35 ~ seriously underburdened.
            hpcc_high: 0.7,          // HPCC >= 0.7 ~ strong helpfulness coupling. [file:40]
            hpcc_low: 0.3,           // HPCC <= 0.3 ~ cosmetic / weak help.
            bioload_low: 0.3,        // Below 0.3 ~ comfortably inside corridor. [file:34]
            bioload_high: 0.8,       // Above 0.8 ~ dangerously near ceiling.
            tecr_high: 0.4,          // Local contribution to high collapse regime. [file:40]
            tecr_low: 0.1,           // Near-zero collapse contribution.
            fear_min_safe: 0.2,
            fear_max_safe: 0.8,
        }
    }
}

/// Input including FEAR, which lives with the Site tokens.
#[derive(Clone, Copy, Debug)]
pub struct SiteWithFear {
    pub snapshot: SiteSnapshot,
    /// FEAR scalar in [0, 1] as used in Jetson-Line. [file:35]
    pub fear: f64,
}

/// Evaluate inequality bands and return all doctrinal labels that apply to this site.
/// This is purely functional and side-effect free.
pub fn classify_site(
    site: &SiteWithFear,
    bands: &VocabBands,
) -> Vec<DoctrineLabel> {
    let mut labels = Vec::new();

    let t = &site.snapshot.tokens;
    let j = &site.snapshot.justice;
    let b = &site.snapshot.bio;

    let power_bounded = if t.church > 0.0 {
        t.power <= bands.power_church_k * t.church + 1e-9
    } else {
        // If CHURCH is zero, we treat any significant POWER as unbounded.
        t.power <= 1e-9
    };

    let fear_safe = site.fear >= bands.fear_min_safe && site.fear <= bands.fear_max_safe;
    let low_tecr = j.tecr <= bands.tecr_low;
    let high_tecr = j.tecr >= bands.tecr_high;

    let hpcc_high = j.hpcc >= bands.hpcc_high;
    let hpcc_low = j.hpcc <= bands.hpcc_low;

    let near_fair_exposure = j.erg.abs() <= bands.erg_fair_abs;
    let over_exposed = j.erg >= bands.erg_over_exposed;
    let under_exposed = j.erg <= -bands.erg_under_exposed;

    let low_load = b.bioload <= bands.bioload_low;
    let high_load = b.bioload >= bands.bioload_high;

    // 1. Sanctuary: low load, high CHURCH, bounded POWER, low TECR, safe FEAR.
    if low_load
        && t.church > 0.5
        && power_bounded
        && low_tecr
        && fear_safe
    {
        labels.push(DoctrineLabel::Sanctuary);
    }

    // 2. Host: moderate load, bounded POWER, high HPCC, near-fair exposure, safe FEAR.
    if !low_load
        && !high_load
        && power_bounded
        && hpcc_high
        && near_fair_exposure
        && fear_safe
    {
        labels.push(DoctrineLabel::Host);
    }

    // 3. Sacrifice: high bioload or over-exposed, but high HPCC and bounded POWER.
    if (high_load || over_exposed)
        && hpcc_high
        && power_bounded
    {
        labels.push(DoctrineLabel::Sacrifice);
    }

    // 4. Exile: high TECR or chronic unsafe FEAR, low CHURCH.
    if (high_tecr || !fear_safe) && t.church < 0.2 {
        labels.push(DoctrineLabel::Exile);
    }

    // 5. Exploiter: under-exposed given responsibility, high POWER vs CHURCH, low HPCC.
    if under_exposed
        && !power_bounded
        && hpcc_low
        && low_load
    {
        labels.push(DoctrineLabel::Exploiter);
    }

    // 6. FairTrader: balanced metrics, no extremes.
    if !labels.contains(&DoctrineLabel::Exploiter)
        && !labels.contains(&DoctrineLabel::Exile)
        && !labels.contains(&DoctrineLabel::Sacrifice)
        && near_fair_exposure
        && !hpcc_low
        && !high_tecr
        && !high_load
    {
        labels.push(DoctrineLabel::FairTrader);
    }

    labels
}

/// Example Episode-level classification hook for "miracle" detection.
/// This works on pre-computed before/after aggregates and keeps the
/// definition explicit and tunable.
#[derive(Clone, Copy, Debug)]
pub struct EpisodeSummary {
    pub hpcc_before: f64,
    pub hpcc_after: f64,
    pub erg_before: f64,
    pub erg_after: f64,
    pub tecr_before: f64,
    pub tecr_after: f64,
    /// Global mean bioload before/after.
    pub bioload_before: f64,
    pub bioload_after: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct MiracleBands {
    /// Minimum required improvement in HPCC.
    pub min_delta_hpcc: f64,
    /// Minimum required reduction in |ERG|.
    pub min_delta_erg_abs: f64,
    /// Minimum required reduction in TECR.
    pub min_delta_tecr: f64,
    /// Minimum required reduction in mean bioload.
    pub min_delta_bioload: f64,
}

impl Default for MiracleBands {
    fn default() -> Self {
        MiracleBands {
            min_delta_hpcc: 0.15,
            min_delta_erg_abs: 0.15,
            min_delta_tecr: 0.15,
            min_delta_bioload: 0.1,
        }
    }
}

/// Decide whether an Episode qualifies as "miracle" in the corridor-safe,
/// Tree-of-Life sense: large, sustained fairness and load improvement without
/// breaking invariants. [file:40][file:35]
pub fn classify_episode_miracle(
    summary: &EpisodeSummary,
    bands: &MiracleBands,
) -> bool {
    let delta_hpcc = summary.hpcc_after - summary.hpcc_before;
    let delta_erg_abs = summary.erg_before.abs() - summary.erg_after.abs();
    let delta_tecr = summary.tecr_before - summary.tecr_after;
    let delta_bioload = summary.bioload_before - summary.bioload_after;

    delta_hpcc >= bands.min_delta_hpcc
        && delta_erg_abs >= bands.min_delta_erg_abs
        && delta_tecr >= bands.min_delta_tecr
        && delta_bioload >= bands.min_delta_bioload
}
