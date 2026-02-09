use serde::{Serialize, Deserialize};
use crate::policy::CapabilityState;              // from aln_core.rs lattice
use crate::envelope::BiophysicalEnvelopeSnapshot; // from BiophysicalEnvelopeSpec
use crate::roh::RoHProjection;                   // roh_before, roh_after, ceiling

/// View-only input for a single neuromorphic snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroPrintInput {
    pub capability_state: CapabilityState,
    pub roh: RoHProjection,
    pub envelope: BiophysicalEnvelopeSnapshot,
    pub evolve_index: Option<u64>,
    pub epoch_index: Option<u64>,
}

/// Human-binary compatible “neuroprint” view.
/// All fields are normalized to [0.0, 1.0] or are symbolic labels only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuroPrintView {
    // TREE-style assets, 0.0–1.0, pure projections.
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

    // Optional advisory labels only; no policy effect.
    pub labels: Vec<String>,
}

/// Pure, non-actuating projection from governed state to neuroprint view.
pub fn neuroprint_from_snapshot(input: &NeuroPrintInput) -> NeuroPrintView {
    // Helpers (clamp, mappers) use only envelope + RoH; no capability mutation.
    fn clamp01(x: f32) -> f32 {
        if x < 0.0 { 0.0 } else if x > 1.0 { 1.0 } else { x }
    }

    // Examples of grounded mappings (you’d plug in the exact formulas from
    // BiophysicalEnvelopeSpec / Tree-of-Life mapping tables).
    let blood  = clamp01(map_blood(&input.envelope));
    let oxygen = clamp01(map_oxygen(&input.envelope));
    let wave   = clamp01(map_wave(&input.envelope));

    // RoH-based assets (DECAY, LIFEFORCE) as in the provable-spine spec.
    let roh_norm = clamp01(input.roh.after / input.roh.ceiling); // 0..1 within ceiling
    let decay     = roh_norm;           // higher RoH → higher DECAY
    let lifeforce = 1.0 - roh_norm;     // complement in [0,1]

    // Capability/evolution-derived axes (EVOLVE, BRAIN, SMART, NANO).
    let evolve   = map_evolve(input.capability_state, input.evolve_index);
    let brain    = map_brain(input.capability_state);
    let smart    = map_smart(input.capability_state);
    let nano     = map_nano(input.evolve_index);

    // Remaining fields follow the Tree-of-Life blueprint; all formulas documented
    // and sourced from existing envelope shards (no speculative signals).
    let h2o   = map_h2o_placeholder();     // currently neutral until hydration axis exists
    let time  = map_time(&input.envelope);
    let power = map_power(&input.envelope);
    let tech  = map_tech(&input.envelope);
    let fear  = map_fear(&input.envelope);
    let pain  = map_pain(&input.envelope);

    let labels = derive_advisory_labels(decay, lifeforce, fear, pain);

    NeuroPrintView {
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
        labels,
    }
}
