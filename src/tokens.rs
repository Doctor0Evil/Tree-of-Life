// src/tokens.rs
// Tree-of-Life compatible token definitions and safe helpers for the Jetson-Line.
// This module has no side effects and no external IO; it is intended to be imported
// by the core 1-D MicroSociety dynamics and deed logic.

use serde::{Deserialize, Serialize};

/// Clamp a value into [min, max] to keep all tokens in safe numeric bounds.
fn clamp(v: f64, min: f64, max: f64) -> f64 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// Core Tree-of-Life tokens for one site on the Jetson-Line.
///
/// All fields are f64 for high precision; invariants are enforced by helper functions.
/// Units are normalized scalars (0–1 or capped ranges) as documented in TREETOKENS.md.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExtendedTokenState {
    // Moral–governance
    pub church: f64,     // ≥ 0
    pub fear: f64,       // bounded by [fear_min, fear_max] in practice
    pub power: f64,      // ≥ 0, with power <= k * church enforced by helpers
    pub tech: f64,       // ≥ 0, usually modest ceiling

    // Biophysical–ecological
    pub bioload: f64,    // ≥ 0, with territory-specific ceilings
    pub pollution: f64,  // ≥ 0
    pub exposure: f64,   // ≥ 0

    // Life–spectral
    pub life: f64,       // ≥ 0, can be very small (e.g. 1e-13)
    pub death: f64,      // ≥ 0, monotone non-decreasing
    pub lifeforce: f64,  // bounded vitality scalar, typically [0,1]
    pub decay: f64,      // ≥ 0, with hard ceiling 1.0

    // Social–relational
    pub trust: f64,      // typically [-1,1] or remapped [0,1]
    pub justice: f64,    // [0,1] diagnostic fairness proxy
    pub habit: f64,      // ≥ 0, entrenched behavior strength
    pub sovereignty: f64,// [0,1] consent / self-determination
    pub sacrifice: f64,  // ≥ 0, cumulative and monotone
}

impl ExtendedTokenState {
    /// Construct a new token state with all scalars set to zero.
    /// Callers should then set initial values explicitly or via scenario helpers.
    pub fn zero() -> Self {
        Self {
            church: 0.0,
            fear: 0.0,
            power: 0.0,
            tech: 0.0,
            bioload: 0.0,
            pollution: 0.0,
            exposure: 0.0,
            life: 0.0,
            death: 0.0,
            lifeforce: 0.0,
            decay: 0.0,
            trust: 0.0,
            justice: 0.0,
            habit: 0.0,
            sovereignty: 0.0,
            sacrifice: 0.0,
        }
    }

    /// Apply hard numeric safety clamps that should hold at every tick
    /// after all deed updates for a site.
    ///
    /// This enforces:
    /// - non-negativity for most tokens,
    /// - DECAY <= 1.0,
    /// - JUSTICE in [0,1],
    /// - SOVEREIGNTY in [0,1],
    /// - simple TRUST band [-1,1],
    /// - LIFE, DEATH, SACRIFICE monotone non-negative.
    pub fn clamp_invariants(&mut self) {
        // Moral–governance
        self.church = self.church.max(0.0);
        self.fear = self.fear.max(0.0); // global bands applied elsewhere
        self.power = self.power.max(0.0);
        self.tech = self.tech.max(0.0);

        // Biophysical–ecological
        self.bioload = self.bioload.max(0.0);
        self.pollution = self.pollution.max(0.0);
        self.exposure = self.exposure.max(0.0);

        // Life–spectral
        self.life = self.life.max(0.0);
        self.death = self.death.max(0.0);
        self.lifeforce = clamp(self.lifeforce, 0.0, 1.0);
        self.decay = clamp(self.decay, 0.0, 1.0);

        // Social–relational
        self.trust = clamp(self.trust, -1.0, 1.0);
        self.justice = clamp(self.justice, 0.0, 1.0);
        self.habit = self.habit.max(0.0);
        self.sovereignty = clamp(self.sovereignty, 0.0, 1.0);
        self.sacrifice = self.sacrifice.max(0.0);
    }

    /// Enforce Neuromorph-GOD's POWER ≤ k * CHURCH constraint locally for this site.
    ///
    /// This should be called after any operation that might change POWER or CHURCH.
    pub fn enforce_power_cap(&mut self, k: f64) {
        if k <= 0.0 {
            // Degenerate case: no legitimate power allowed.
            self.power = 0.0;
            return;
        }
        let max_power = k * self.church.max(0.0);
        if self.power > max_power {
            self.power = max_power;
        }
    }

    /// Update FEAR as a simple function of BIOLOAD, EXPOSURE, and TRUST.
    ///
    /// This is a local, interpretable homeostatic rule; global safe bands
    /// are checked elsewhere.
    pub fn update_fear(&mut self, alpha_load: f64, alpha_exposure: f64, beta_trust: f64) {
        // Normalize load and exposure contributions using a soft factor
        // to keep numbers in a reasonable range without guessing real physiology.
        let load_term = alpha_load * self.bioload;
        let exposure_term = alpha_exposure * self.exposure;
        let trust_term = beta_trust * self.trust; // trust can buffer fear if beta_trust < 0

        let delta = load_term + exposure_term + trust_term;
        self.fear += delta;
        if self.fear < 0.0 {
            self.fear = 0.0;
        }
    }

    /// Simple TECH growth rule: TECH increases when POWER is available
    /// and bioload is under a given threshold.
    pub fn update_tech(&mut self, bioload_threshold: f64, tech_rate: f64) {
        if self.bioload <= bioload_threshold && self.power > 0.0 {
            let inc = tech_rate * self.power;
            self.tech += inc;
        }
    }

    /// Increment LIFE and BIOLOAD when new life emerges or colonization seeds a site.
    ///
    /// `delta_life` may be very small (e.g., 1e-13) to represent first microscopic life.
    pub fn add_life(&mut self, delta_life: f64, extra_bioload: f64) {
        if delta_life > 0.0 {
            self.life += delta_life;
        }
        if extra_bioload > 0.0 {
            self.bioload += extra_bioload;
        }
    }

    /// Record a death / extinction event:
    /// - LIFE decreases (not below zero),
    /// - DEATH increases by the same or a scaled amount,
    /// - DECAY and BIOLOAD may increase to reflect residue or damage.
    pub fn register_death(
        &mut self,
        life_lost: f64,
        death_scale: f64,
        decay_increase: f64,
        bioload_increase: f64,
    ) {
        if life_lost > 0.0 {
            let actual_loss = life_lost.min(self.life);
            self.life -= actual_loss;
            self.death += death_scale.max(0.0) * actual_loss;
        }
        if decay_increase > 0.0 {
            self.decay += decay_increase;
        }
        if bioload_increase > 0.0 {
            self.bioload += bioload_increase;
        }
    }

    /// Add pollution and associated bioload.
    pub fn add_pollution(&mut self, delta_pollution: f64, extra_bioload: f64) {
        if delta_pollution > 0.0 {
            self.pollution += delta_pollution;
        }
        if extra_bioload > 0.0 {
            self.bioload += extra_bioload;
        }
    }

    /// Perform a repair-style deed:
    /// - reduces BIOLOAD and POLLUTION,
    /// - costs CHURCH and possibly POWER,
    /// - can slightly increase JUSTICE and TRUST.
    pub fn apply_repair_deed(
        &mut self,
        bioload_reduction: f64,
        pollution_reduction: f64,
        church_cost: f64,
        power_cost: f64,
        justice_gain: f64,
        trust_gain: f64,
        sacrifice_gain: f64,
    ) {
        // Costs
        if church_cost > 0.0 {
            self.church = (self.church - church_cost).max(0.0);
        }
        if power_cost > 0.0 {
            self.power = (self.power - power_cost).max(0.0);
        }

        // Load and pollution reduction
        if bioload_reduction > 0.0 {
            self.bioload = (self.bioload - bioload_reduction).max(0.0);
        }
        if pollution_reduction > 0.0 {
            self.pollution = (self.pollution - pollution_reduction).max(0.0);
        }

        // Social–moral improvements
        if justice_gain > 0.0 {
            self.justice += justice_gain;
        }
        if trust_gain > 0.0 {
            self.trust += trust_gain;
        }
        if sacrifice_gain > 0.0 {
            self.sacrifice += sacrifice_gain;
        }
    }

    /// Apply a cooperative help deed between two sites (self = actor, other = recipient).
    ///
    /// This models:
    /// - POWER and possibly CHURCH cost for the actor,
    /// - CHURCH and LIFE / LIFEFORCE benefit for the recipient,
    /// - TRUST increase for both,
    /// - modest BIOLOAD increase (sacrifice).
    pub fn apply_help_deed_with(
        &mut self,
        other: &mut ExtendedTokenState,
        power_cost: f64,
        church_gain_other: f64,
        lifeforce_gain_other: f64,
        trust_gain: f64,
        bioload_gain_self: f64,
        bioload_gain_other: f64,
        sacrifice_gain_self: f64,
    ) {
        // Actor costs
        if power_cost > 0.0 {
            self.power = (self.power - power_cost).max(0.0);
        }
        if bioload_gain_self > 0.0 {
            self.bioload += bioload_gain_self;
        }
        if sacrifice_gain_self > 0.0 {
            self.sacrifice += sacrifice_gain_self;
        }

        // Recipient benefits
        if church_gain_other > 0.0 {
            other.church += church_gain_other;
        }
        if lifeforce_gain_other > 0.0 {
            other.lifeforce += lifeforce_gain_other;
        }
        if bioload_gain_other > 0.0 {
            other.bioload += bioload_gain_other;
        }

        // Trust and justice
        if trust_gain > 0.0 {
            self.trust += trust_gain;
            other.trust += trust_gain;
        }
    }

    /// Apply a conflict deed between two sites (self = one side, other = the other side).
    ///
    /// This models:
    /// - Possible POWER gain for the "winner" (controlled externally),
    /// - CHURCH loss for both,
    /// - TRUST loss for both,
    /// - BIOLOAD, POLLUTION, DECAY increase for both,
    /// - JUSTICE decrease.
    pub fn apply_conflict_with(
        &mut self,
        other: &mut ExtendedTokenState,
        church_loss: f64,
        trust_penalty: f64,
        bioload_increase: f64,
        pollution_increase: f64,
        decay_increase: f64,
        justice_loss: f64,
    ) {
        if church_loss > 0.0 {
            self.church = (self.church - church_loss).max(0.0);
            other.church = (other.church - church_loss).max(0.0);
        }
        if trust_penalty > 0.0 {
            self.trust -= trust_penalty;
            other.trust -= trust_penalty;
        }
        if bioload_increase > 0.0 {
            self.bioload += bioload_increase;
            other.bioload += bioload_increase;
        }
        if pollution_increase > 0.0 {
            self.pollution += pollution_increase;
            other.pollution += pollution_increase;
        }
        if decay_increase > 0.0 {
            self.decay += decay_increase;
            other.decay += decay_increase;
        }
        if justice_loss > 0.0 {
            self.justice = (self.justice - justice_loss).max(0.0);
            other.justice = (other.justice - justice_loss).max(0.0);
        }
    }

    /// Simple justice update based on local pollution, bioload, and habit.
    ///
    /// This is intentionally transparent: higher pollution/bioload/habit of harm
    /// reduces justice slightly; repair deeds should then raise it.
    pub fn update_justice_from_context(
        &mut self,
        weight_bioload: f64,
        weight_pollution: f64,
        weight_habit: f64,
    ) {
        let penalty = weight_bioload * self.bioload
            + weight_pollution * self.pollution
            + weight_habit * self.habit;

        if penalty > 0.0 {
            self.justice = (self.justice - penalty).max(0.0);
        }
    }

    /// Increment habit toward harmful or restorative patterns.
    ///
    /// Positive `delta` strengthens the current direction of behavior;
    /// callers should separate "harmful habit" vs "restorative habit" if needed.
    pub fn update_habit(&mut self, delta: f64) {
        if delta > 0.0 {
            self.habit += delta;
        }
    }
}
