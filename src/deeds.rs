// src/deeds.rs
// Unilateral deed proposal, validation, and invariant enforcement kernel
// for the Jetson-Line, using ExtendedTokenState from tokens.rs.
//
// This module is intentionally self-contained and non-actuating:
// - It mutates in-memory token states when called,
// - It produces an auditable ValidatedDeed record for each attempt,
// - It does NOT talk to any blockchain or external IO; that is the job
//   of a separate adapter layer that can hash and anchor these records.

use serde::{Deserialize, Serialize};

use crate::tokens::ExtendedTokenState;

/// Unique identifier types; you can alias or replace with your own.
pub type SiteId = u64;
pub type AgentId = u64;
pub type Tick = u64;

/// Types of deeds that agents may propose unilaterally.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeedType {
    Help,
    Conflict,
    Colonize,
    Repair,
    EmitPollution,
    DeployTech,
    // You can extend with Dialogue, UseSupport, etc.
}

/// Status after validation and invariant enforcement.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeedStatus {
    Success,
    Transformed, // e.g., scaled or converted to Repair
    Blocked,
}

/// Minimal, deterministic record of a processed deed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedDeed {
    pub tick: Tick,
    pub deed_id: u64,
    pub proposer: AgentId,
    pub deed_type: DeedType,
    pub original_type: DeedType,
    pub status: DeedStatus,
    pub source_site: SiteId,
    pub target_site: Option<SiteId>,

    // Human-readable or machine-parsable reason(s) for block/transform.
    pub reason: String,

    // Snapshots of pre-/post-state for auditing.
    pub pre_source: ExtendedTokenState,
    pub post_source: ExtendedTokenState,
    pub pre_target: Option<ExtendedTokenState>,
    pub post_target: Option<ExtendedTokenState>,
}

/// Unilateral deed request from an agent.
/// This is the only structure agents are allowed to construct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeedRequest {
    pub tick: Tick,
    pub deed_id: u64,
    pub proposer: AgentId,
    pub deed_type: DeedType,
    pub source_site: SiteId,
    pub target_site: Option<SiteId>,

    /// Scalar intensity for deeds that admit scaling (e.g., pollution amount,
    /// colonization strength, conflict intensity). This will be clamped.
    pub intensity: f64,
}

/// Global scalar parameters for invariants and tuning.
/// You can load these from configuration or a doctrine file.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct InvariantParams {
    pub roh_max: f64,            // e.g., 0.3
    pub decay_max: f64,          // always 1.0 in your doctrine
    pub power_church_k: f64,     // POWER <= k * CHURCH
    pub fear_min: f64,
    pub fear_max: f64,
    pub sovereignty_min_invasive: f64,
    pub bioload_max_site: f64,
    pub justice_min: f64,        // for justice corridor tightening
}

/// Simple container for all site states on the 1-D Jetson-Line.
/// This keeps the kernel generic: it only sees ids and a map of states.
#[derive(Debug)]
pub struct WorldLine {
    /// Sites indexed by SiteId; in a 1-D array model, SiteId can be the index.
    pub sites: Vec<ExtendedTokenState>,
}

impl WorldLine {
    pub fn get(&self, id: SiteId) -> Option<&ExtendedTokenState> {
        self.sites.get(id as usize)
    }

    pub fn get_mut(&mut self, id: SiteId) -> Option<&mut ExtendedTokenState> {
        self.sites.get_mut(id as usize)
    }

    pub fn len(&self) -> usize {
        self.sites.len()
    }
}

/// Centralized validation kernel implementing Neuromorph-GOD invariants
/// and deed transformations.
pub struct ValidationKernel {
    pub invariants: InvariantParams,
}

impl ValidationKernel {
    pub fn new(invariants: InvariantParams) -> Self {
        Self { invariants }
    }

    /// Main entry point: validate and apply a deed request.
    ///
    /// - Reads pre-states from `world`,
    /// - Computes hypothetical post-states,
    /// - Enforces invariants and possibly transforms the deed,
    /// - Applies the final, permitted change to `world`,
    /// - Returns a ValidatedDeed for audit / blockchain anchoring.
    pub fn process_deed(
        &self,
        world: &mut WorldLine,
        req: DeedRequest,
    ) -> Option<ValidatedDeed> {
        // Fetch source state
        let src = world.get(req.source_site)?.clone();
        let mut src_new = src;

        // Fetch optional target
        let (tgt_opt, mut tgt_new_opt) = if let Some(tid) = req.target_site {
            let t = world.get(tid)?.clone();
            (Some(t), Some(t))
        } else {
            (None, None)
        };

        // Ensure intensity non-negative and bounded for stability.
        let intensity = if req.intensity < 0.0 {
            0.0
        } else if req.intensity > 1.0 {
            1.0
        } else {
            req.intensity
        };

        // Apply pedagogical update rule to hypothetical copies.
        match req.deed_type {
            DeedType::Colonize => {
                self.apply_colonize_rule(&mut src_new, tgt_new_opt.as_mut(), intensity);
            }
            DeedType::Repair => {
                self.apply_repair_rule(&mut src_new, intensity);
            }
            DeedType::EmitPollution => {
                self.apply_emit_pollution_rule(&mut src_new, tgt_new_opt.as_mut(), intensity);
            }
            DeedType::Conflict => {
                self.apply_conflict_rule(&mut src_new, tgt_new_opt.as_mut(), intensity);
            }
            DeedType::Help => {
                self.apply_help_rule(&mut src_new, tgt_new_opt.as_mut(), intensity);
            }
            DeedType::DeployTech => {
                self.apply_deploy_tech_rule(&mut src_new, intensity);
            }
        }

        // Enforce local clamps on hypothetical states.
        src_new.clamp_invariants();
        if let Some(ref mut t) = tgt_new_opt {
            t.clamp_invariants();
        }

        // Check invariants and possibly transform / scale.
        let (status, final_type, reason, src_final, tgt_final_opt) =
            self.enforce_invariants(
                req.deed_type,
                &src,
                &src_new,
                tgt_opt.as_ref(),
                tgt_new_opt.as_ref(),
                intensity,
            );

        // If blocked, do not apply any changes to the world.
        if status == DeedStatus::Blocked {
            return Some(ValidatedDeed {
                tick: req.tick,
                deed_id: req.deed_id,
                proposer: req.proposer,
                deed_type: final_type,
                original_type: req.deed_type,
                status,
                source_site: req.source_site,
                target_site: req.target_site,
                reason,
                pre_source: src,
                post_source: src,
                pre_target: tgt_opt,
                post_target: tgt_opt,
            });
        }

        // Apply final states to world (success or transformed).
        if let Some(src_site) = world.get_mut(req.source_site) {
            *src_site = src_final;
        }
        if let (Some(tid), Some(tgt_final)) = (req.target_site, tgt_final_opt) {
            if let Some(tgt_site) = world.get_mut(tid) {
                *tgt_site = tgt_final;
            }
        }

        Some(ValidatedDeed {
            tick: req.tick,
            deed_id: req.deed_id,
            proposer: req.proposer,
            deed_type: final_type,
            original_type: req.deed_type,
            status,
            source_site: req.source_site,
            target_site: req.target_site,
            reason,
            pre_source: src,
            post_source: src_final,
            pre_target: tgt_opt,
            post_target: tgt_final_opt,
        })
    }

    // ---------- Pedagogical deed rules on hypothetical states ----------

    fn apply_colonize_rule(
        &self,
        src: &mut ExtendedTokenState,
        tgt_opt: Option<&mut ExtendedTokenState>,
        intensity: f64,
    ) {
        let life_seed = 1e-6 * intensity;
        let bioload_gain = 1e-3 * intensity;

        // Sacrifice cost at source
        let church_cost = 0.1 * intensity;
        let power_cost = 0.1 * intensity;

        if church_cost > 0.0 {
            src.church = (src.church - church_cost).max(0.0);
        }
        if power_cost > 0.0 {
            src.power = (src.power - power_cost).max(0.0);
        }

        // Increase LIFE / BIOLOAD at source (frontier) or target (new site).
        if let Some(tgt) = tgt_opt {
            tgt.add_life(life_seed, bioload_gain);
        } else {
            src.add_life(life_seed, bioload_gain);
        }

        // Colonization usually increases bioload at source too (logistics).
        src.bioload += 0.5 * bioload_gain;
    }

    fn apply_repair_rule(&self, src: &mut ExtendedTokenState, intensity: f64) {
        // Stronger intensity => more repair but more cost.
        let bioload_reduction = 0.01 * intensity;
        let pollution_reduction = 0.02 * intensity;
        let church_cost = 0.01 * intensity;
        let power_cost = 0.01 * intensity;
        let justice_gain = 0.01 * intensity;
        let trust_gain = 0.01 * intensity;
        let sacrifice_gain = 0.02 * intensity;

        src.apply_repair_deed(
            bioload_reduction,
            pollution_reduction,
            church_cost,
            power_cost,
            justice_gain,
            trust_gain,
            sacrifice_gain,
        );

        // LIFEFORCE recovery, DECAY stabilization.
        src.lifeforce += 0.01 * intensity;
        src.decay = (src.decay - 0.005 * intensity).max(0.0);
    }

    fn apply_emit_pollution_rule(
        &self,
        src: &mut ExtendedTokenState,
        tgt_opt: Option<&mut ExtendedTokenState>,
        intensity: f64,
    ) {
        let pollution_amount = 0.02 * intensity;
        let bioload_gain = 0.01 * intensity;
        let power_cost = 0.005 * intensity;

        if power_cost > 0.0 {
            src.power = (src.power - power_cost).max(0.0);
        }
        src.add_pollution(pollution_amount, bioload_gain);

        // Spillover to target if present.
        if let Some(tgt) = tgt_opt {
            tgt.add_pollution(0.5 * pollution_amount, 0.5 * bioload_gain);
            // FEAR rises from exposure to pollution.
            tgt.fear += 0.01 * intensity;
        }

        // Source FEAR increase as well.
        src.fear += 0.01 * intensity;
    }

    fn apply_conflict_rule(
        &self,
        src: &mut ExtendedTokenState,
        tgt_opt: Option<&mut ExtendedTokenState>,
        intensity: f64,
    ) {
        let church_loss = 0.02 * intensity;
        let trust_penalty = 0.05 * intensity;
        let bioload_increase = 0.02 * intensity;
        let pollution_increase = 0.01 * intensity;
        let decay_increase = 0.01 * intensity;
        let justice_loss = 0.02 * intensity;

        if let Some(tgt) = tgt_opt {
            src.apply_conflict_with(
                tgt,
                church_loss,
                trust_penalty,
                bioload_increase,
                pollution_increase,
                decay_increase,
                justice_loss,
            );
        } else {
            // Self-conflict: apply symmetric penalties to src only.
            src.church = (src.church - church_loss).max(0.0);
            src.trust -= trust_penalty;
            src.bioload += bioload_increase;
            src.pollution += pollution_increase;
            src.decay += decay_increase;
            src.justice = (src.justice - justice_loss).max(0.0);
        }

        // FEAR jump at both sites handled via context update outside or here:
        src.fear += 0.05 * intensity;
        if let Some(tgt) = tgt_opt {
            tgt.fear += 0.05 * intensity;
        }
    }

    fn apply_help_rule(
        &self,
        src: &mut ExtendedTokenState,
        tgt_opt: Option<&mut ExtendedTokenState>,
        intensity: f64,
    ) {
        if let Some(tgt) = tgt_opt {
            src.apply_help_deed_with(
                tgt,
                power_cost = 0.01 * intensity,
                church_gain_other = 0.02 * intensity,
                lifeforce_gain_other = 0.02 * intensity,
                trust_gain = 0.02 * intensity,
                bioload_gain_self = 0.005 * intensity,
                bioload_gain_other = 0.002 * intensity,
                sacrifice_gain_self = 0.01 * intensity,
            );
        } else {
            // Self-support: small self-repair / trust.
            src.power = (src.power - 0.005 * intensity).max(0.0);
            src.church += 0.01 * intensity;
            src.lifeforce += 0.01 * intensity;
            src.trust += 0.01 * intensity;
        }
    }

    fn apply_deploy_tech_rule(&self, src: &mut ExtendedTokenState, intensity: f64) {
        let tech_gain = 0.05 * intensity;
        let power_cost = 0.02 * intensity;
        let bioload_increase = 0.02 * intensity;

        if power_cost > 0.0 {
            src.power = (src.power - power_cost).max(0.0);
        }
        src.tech += tech_gain;
        src.bioload += bioload_increase;
        // FEAR may increase if tech is perceived as risky.
        src.fear += 0.01 * intensity;
    }

    // ---------- Invariant enforcement, scaling, and transformation ----------

    #[allow(clippy::too_many_arguments)]
    fn enforce_invariants(
        &self,
        deed_type: DeedType,
        src_pre: &ExtendedTokenState,
        src_new: &ExtendedTokenState,
        tgt_pre: Option<&ExtendedTokenState>,
        tgt_new: Option<&ExtendedTokenState>,
        intensity: f64,
    ) -> (
        DeedStatus,
        DeedType,
        String,
        ExtendedTokenState,
        Option<ExtendedTokenState>,
    ) {
        // Start by assuming we accept the hypothetical new states.
        let mut status = DeedStatus::Success;
        let mut final_type = deed_type;
        let mut reason = String::from("OK");

        let mut src_final = *src_new;
        let mut tgt_final = tgt_new.cloned();

        // 1. Biophysical safety: DECAY, LIFEFORCE band, BIOLOAD ceiling.
        if src_new.decay > self.invariants.decay_max
            || src_new.bioload > self.invariants.bioload_max_site
        {
            // Try to transform harmful deeds into Repair when meaningful.
            match deed_type {
                DeedType::Conflict | DeedType::EmitPollution | DeedType::DeployTech => {
                    status = DeedStatus::Transformed;
                    final_type = DeedType::Repair;
                    reason = "Transformed to Repair: proposed state would violate DECAY or BIOLOAD ceiling"
                        .to_string();

                    // Fall back to a conservative repair-style correction on pre-state.
                    src_final = *src_pre;
                    // Apply a small repair to counter the attempted harm.
                    let mut tmp = src_final;
                    self.apply_repair_rule(&mut tmp, intensity * 0.5);
                    tmp.clamp_invariants();
                    src_final = tmp;
                    // Target unchanged for repair in this basic version.
                    tgt_final = tgt_pre.cloned();
                }
                _ => {
                    status = DeedStatus::Blocked;
                    final_type = deed_type;
                    reason = "Blocked: proposed state would violate DECAY or BIOLOAD ceiling"
                        .to_string();
                    return (status, final_type, reason, *src_pre, tgt_pre.cloned());
                }
            }
        }

        // 2. Moral corridors: POWER <= k * CHURCH, FEAR band, SOVEREIGNTY for invasive deeds.
        // Enforce local POWER cap.
        let mut tmp_src = src_final;
        tmp_src.enforce_power_cap(self.invariants.power_church_k);

        if tmp_src.power < src_final.power {
            status = DeedStatus::Transformed;
            reason = format!(
                "{}; POWER capped by k * CHURCH",
                reason
            );
            src_final = tmp_src;
        }

        // FEAR band check (simple clamp; stronger policies can block).
        if src_final.fear < self.invariants.fear_min
            || src_final.fear > self.invariants.fear_max
        {
            // Clamp but do not block: FEAR is a signal; biophysical corridors are stricter.
            src_final.fear = clamp(
                src_final.fear,
                self.invariants.fear_min,
                self.invariants.fear_max,
            );
            status = DeedStatus::Transformed;
            reason = format!("{}; FEAR clamped into safe band", reason);
        }

        // Invasive deeds require minimum sovereignty.
        let invasive = matches!(
            deed_type,
            DeedType::Colonize | DeedType::DeployTech | DeedType::EmitPollution | DeedType::Conflict
        );
        if invasive && src_pre.sovereignty < self.invariants.sovereignty_min_invasive {
            status = DeedStatus::Blocked;
            final_type = deed_type;
            reason = "Blocked: inadequate SOVEREIGNTY for invasive/high-impact deed".to_string();
            return (status, final_type, reason, *src_pre, tgt_pre.cloned());
        }

        // 3. Justice corridors: prevent worsening UNFAIRDRAIN beyond threshold.
        if src_new.justice < self.invariants.justice_min {
            match deed_type {
                DeedType::EmitPollution | DeedType::Conflict => {
                    status = DeedStatus::Blocked;
                    final_type = deed_type;
                    reason = "Blocked: justice corridor tightened (UNFAIRDRAIN risk)".to_string();
                    return (status, final_type, reason, *src_pre, tgt_pre.cloned());
                }
                _ => {
                    // For neutral/positive deeds we allow but log the low-justice context.
                    reason = format!("{}; justice below preferred corridor", reason);
                }
            }
        }

        (status, final_type, reason, src_final, tgt_final)
    }
}

// Local helper re-exported because we used it above.
fn clamp(v: f64, min: f64, max: f64) -> f64 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}
