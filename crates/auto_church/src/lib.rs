use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Short role and archetype identifiers used in logs and UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Archetype {
    Teacher,
    Learner,
    Mentor,
    Follower,
    Believer,
    Preacher,
}

/// High‑level impact domain of a deed, aligned with your eco/NPO focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImpactDomain {
    HomelessnessRelief,
    EcologicalSustainability,
    MathEducation,
    ScienceResearch,
    GeometryLearning,
    CivicDuty,
    MediationPeace,
}

/// Raw log record of a “good deed” coming from upstream systems.
/// This is the only input AutoChurch uses; no access to capability state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodDeedLog {
    pub subject_id: String,
    pub archetype: Archetype,
    pub domain: ImpactDomain,
    /// Wall‑clock duration of the deed, if known (e.g., hours volunteered).
    pub duration: Duration,
    /// Optional scalar 0.0–1.0 reflecting envelope‑safe effort (e.g., BLOOD/LIFEFORCE).
    /// This must be pre‑computed by Tree‑of‑Life / envelope kernels.
    pub effort_safe_scalar: f32,
    /// Whether the deed was validated by a multi‑sig quorum (Org, Mentor, Regulator).
    pub multisig_validated: bool,
    /// Optional flag if the deed contributed to reversing harm (forgiveness weight).
    pub remediation: bool,
}

/// Advisory token balances for a subject.
/// All values are diagnostic only, never used for capability control.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutoChurchBalance {
    /// Time‑protected reflection of good deeds (CHURCH).
    pub church: f32,
    /// Advisory compute/energy budget hints.
    pub pwr_budget: f32,
    /// Advisory chat/AI‑time budget hints.
    pub chat_budget: f32,
    /// Advisory “tech innovation” intensity metric.
    pub tech_score: f32,
    /// Optional moral position 0.0–1.0, for HUDs only.
    pub moral_position: f32,
}

/// Compact configuration for minting and budgeting.
/// Loaded from JSON/ALN policy files by sovereignty core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoChurchConfig {
    /// Base CHURCH minted per validated hour of deed.
    pub base_mint_per_hour: f32,
    /// Bonus multiplier for remediation (forgiveness‑seeking deeds).
    pub remediation_bonus: f32,
    /// Domain‑specific multipliers (ecology, homelessness, etc.).
    pub domain_weight: HashMap<ImpactDomain, f32>,
    /// Archetype‑specific multipliers (teacher, learner, mentor, etc.).
    pub archetype_weight: HashMap<Archetype, f32>,
    /// Conversion ratios from CHURCH to advisory budgets.
    pub church_to_pwr: f32,
    pub church_to_chat: f32,
    pub church_to_tech: f32,
    /// Sliding window half‑life (in hours) for time‑protection decay.
    pub half_life_hours: f32,
}

impl Default for AutoChurchConfig {
    fn default() -> Self {
        use Archetype::*;
        use ImpactDomain::*;

        let mut domain_weight = HashMap::new();
        domain_weight.insert(HomelessnessRelief, 1.4);
        domain_weight.insert(EcologicalSustainability, 1.6);
        domain_weight.insert(MathEducation, 1.1);
        domain_weight.insert(ScienceResearch, 1.2);
        domain_weight.insert(GeometryLearning, 1.0);
        domain_weight.insert(CivicDuty, 1.0);
        domain_weight.insert(MediationPeace, 1.5);

        let mut archetype_weight = HashMap::new();
        archetype_weight.insert(Teacher, 1.2);
        archetype_weight.insert(Learner, 1.0);
        archetype_weight.insert(Mentor, 1.3);
        archetype_weight.insert(Follower, 0.9);
        archetype_weight.insert(Believer, 1.0);
        archetype_weight.insert(Preacher, 1.1);

        AutoChurchConfig {
            base_mint_per_hour: 1.0,
            remediation_bonus: 1.5,
            domain_weight,
            archetype_weight,
            church_to_pwr: 0.5,
            church_to_chat: 0.3,
            church_to_tech: 0.2,
            half_life_hours: 720.0, // ~30 days half‑life
        }
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

/// Pure function to compute CHURCH minted for a single deed.
fn mint_church_for_deed(config: &AutoChurchConfig, deed: &GoodDeedLog) -> f32 {
    let hours = deed.duration.as_secs_f32() / 3600.0;
    if hours <= 0.0 {
        return 0.0;
    }

    let base = config.base_mint_per_hour * hours;

    let domain_mult = config
        .domain_weight
        .get(&deed.domain)
        .copied()
        .unwrap_or(1.0);

    let archetype_mult = config
        .archetype_weight
        .get(&deed.archetype)
        .copied()
        .unwrap_or(1.0);

    let remediation_mult = if deed.remediation {
        config.remediation_bonus
    } else {
        1.0
    };

    // Cap effort_safe_scalar in [0,1]; treat <0.1 as too noisy to count strongly.
    let effort = clamp01(deed.effort_safe_scalar).max(0.1);

    // Multi‑sig validation acts as a strong filter; unvalidated deeds mint very little.
    let validation_factor = if deed.multisig_validated { 1.0 } else { 0.1 };

    let raw = base * domain_mult * archetype_mult * remediation_mult * effort * validation_factor;
    // Soft cap per deed to avoid pathological outliers.
    raw.min(25.0)
}

/// Apply exponential time‑protection decay to an existing CHURCH balance.
/// This keeps old deeds present but gradually frees capacity for new stakeholders.
fn decay_church(existing_church: f32, elapsed_hours: f32, half_life_hours: f32) -> f32 {
    if half_life_hours <= 0.0 || existing_church <= 0.0 || elapsed_hours <= 0.0 {
        return existing_church.max(0.0);
    }
    let lambda = (0.5f32).ln() / half_life_hours;
    let factor = (lambda * elapsed_hours).exp();
    // factor in (0,1]; clamp to protect numeric stability.
    let factor = clamp01(factor);
    existing_church * factor
}

/// Convert CHURCH into advisory PWR/CHAT/TECH budgets.
fn derive_budgets_from_church(config: &AutoChurchConfig, church: f32) -> AutoChurchBalance {
    let church_clamped = church.max(0.0);
    let pwr = church_clamped * config.church_to_pwr;
    let chat = church_clamped * config.church_to_chat;
    let tech = church_clamped * config.church_to_tech;

    // Moral position: saturating logistic‑like mapping into [0,1].
    let mp = clamp01(church_clamped / (church_clamped + 10.0));

    AutoChurchBalance {
        church: church_clamped,
        pwr_budget: pwr,
        chat_budget: chat,
        tech_score: tech,
        moral_position: mp,
    }
}

/// Aggregate over a sequence of deeds for one subject, given an existing balance
/// and the time elapsed since that balance was last updated.
/// This function is pure and side‑effect free.
pub fn recompute_balance_for_subject(
    config: &AutoChurchConfig,
    existing_balance: &AutoChurchBalance,
    elapsed_since_last_update: Duration,
    deeds: &[GoodDeedLog],
) -> AutoChurchBalance {
    let elapsed_hours = elapsed_since_last_update.as_secs_f32() / 3600.0;

    // 1. Decay existing CHURCH for time‑protection.
    let decayed_church = decay_church(
        existing_balance.church,
        elapsed_hours,
        config.half_life_hours,
    );

    // 2. Mint new CHURCH from deeds.
    let mut minted = 0.0;
    for deed in deeds {
        minted += mint_church_for_deed(config, deed);
    }

    let total_church = decayed_church + minted;
    // 3. Derive advisory budgets.
    derive_budgets_from_church(config, total_church)
}

/// Convenience helper: aggregate balances for many subjects.
/// Upstream caller is responsible for providing per‑subject deed slices.
pub fn recompute_balances_for_pool(
    config: &AutoChurchConfig,
    existing: &HashMap<String, AutoChurchBalance>,
    elapsed_since_last_update: Duration,
    deeds_by_subject: &HashMap<String, Vec<GoodDeedLog>>,
) -> HashMap<String, AutoChurchBalance> {
    let mut out = HashMap::new();

    for (subject_id, deeds) in deeds_by_subject.iter() {
        let current = existing
            .get(subject_id)
            .cloned()
            .unwrap_or_else(AutoChurchBalance::default);

        let updated = recompute_balance_for_subject(
            config,
            &current,
            elapsed_since_last_update,
            deeds.as_slice(),
        );

        out.insert(subject_id.clone(), updated);
    }

    out
}
