#[derive(Clone, Copy, Debug)]
pub struct SocialImpactVector {
    // 0.0–1.0 each, matching S_antistigma, S_nonexclusion, S_peacekeeping, S_eco
    pub antistigma: f32,
    pub nonexclusion: f32,
    pub peacekeeping: f32,
    pub eco: f32,
}

impl SocialImpactVector {
    pub fn average(self, other: SocialImpactVector, w_self: f32) -> SocialImpactVector {
        let w_other = 1.0 - w_self;
        SocialImpactVector {
            antistigma: self.antistigma * w_self + other.antistigma * w_other,
            nonexclusion: self.nonexclusion * w_self + other.nonexclusion * w_other,
            peacekeeping: self.peacekeeping * w_self + other.peacekeeping * w_other,
            eco: self.eco * w_self + other.eco * w_other,
        }
    }

    pub fn scalar(&self) -> f32 {
        (self.antistigma + self.nonexclusion + self.peacekeeping + self.eco) / 4.0
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TechDomain {
    Power,
    Nano,
    AI,
    Bio,
}

#[derive(Clone, Copy, Debug)]
pub struct GovernanceEnvelope {
    // 0–1 alignment with neurorights & non‑exclusion guards.
    pub neurorights_alignment: f32,
    pub nonexclusion_enforced: f32,
    pub auditability: f32,
    // Word‑Math style language profile for public comms about this tech.
    pub y_repetition: f32,
    pub z_drift: f32,
    pub t_toxicity: f32,
    pub k_kindness: f32,
    pub e_evidentiality: f32,
}

impl GovernanceEnvelope {
    pub fn quality_score(&self) -> f32 {
        // Matches the f(y,z,T,K,E) pattern: 1−y,1−z,1−T, K, E.[file:20]
        let q_lang =
            (1.0 - self.y_repetition)
            * (1.0 - self.z_drift)
            * (1.0 - self.t_toxicity)
            * self.k_kindness
            * self.e_evidentiality;

        // Governance quality prioritizes neurorights + non‑exclusion + auditability.
        let q_gov = (self.neurorights_alignment * 0.4)
            + (self.nonexclusion_enforced * 0.4)
            + (self.auditability * 0.2);

        // Soft combine, then clamp.
        let q = 0.5 * q_lang + 0.5 * q_gov;
        q.clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TechScenario {
    pub domain: TechDomain,
    // Fraction of population with access at this step (0–1).
    pub adoption_rate: f32,
    // How steep the next‑step adoption curve is (0–1 “acceleration”).
    pub rollout_speed: f32,
    // 0–1 “concentration of power”: 0 = fully distributed, 1 = captured by a few.
    pub centralization: f32,
    // Governance & comms envelope.
    pub governance: GovernanceEnvelope,
    // Intended social‑impact target from the spec / manifesto.
    pub target_impact: SocialImpactVector,
}

#[derive(Clone, Copy, Debug)]
pub struct SocietalState {
    // 0–1, higher = more stable and inclusive.
    pub stability: f32,
    // 0–1, higher = more unrest risk.
    pub unrest_risk: f32,
    // 0–1, higher = systemic fragility (cascade failure risk).
    pub fragility: f32,
    // Observed composite social‑impact from this step.
    pub realized_impact: SocialImpactVector,
}

impl SocietalState {
    pub fn hex_stamp(&self) -> String {
        // Simple, non‑cryptographic hex trace for audit trails.
        // Format: STABxxUNRSyyFRAGzz (each field 0–255 -> two hex chars).
        fn byte(v: f32) -> u8 {
            (v.clamp(0.0, 1.0) * 255.0).round() as u8
        }
        let stab = byte(self.stability);
        let unrest = byte(self.unrest_risk);
        let frag = byte(self.fragility);
        format!("STAB{:02X}UNRS{:02X}FRAG{:02X}", stab, unrest, frag)
    }
}
