use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreeOfLifeNeuroprint {
    // Biophysical scalars 0.0–1.0 (normalized from bioscale models/logs)
    pub blood: f32,
    pub oxygen: f32,
    pub wave: f32,
    pub h2o: f32,
    pub time: f32,
    pub decay: f32,
    pub lifeforce: f32,
    pub brain: f32,

    // Governance / evolution context (still simulation‑only here)
    pub smart: f32,
    pub evolve: f32,
    pub power: f32,
    pub tech: f32,
    pub fear: f32,
    pub pain: f32,
    pub nano: f32,
}

impl TreeOfLifeNeuroprint {
    /// Pure check: all components in [0,1].
    pub fn validate_bounds(&self) -> bool {
        self.iter_components().all(|v| (0.0..=1.0).contains(&v))
    }

    /// Access all components as an iterator (for metrics, distance, etc.).
    pub fn iter_components(&self) -> impl Iterator<Item = f32> + '_ {
        [
            self.blood,
            self.oxygen,
            self.wave,
            self.h2o,
            self.time,
            self.decay,
            self.lifeforce,
            self.brain,
            self.smart,
            self.evolve,
            self.power,
            self.tech,
            self.fear,
            self.pain,
            self.nano,
        ]
        .into_iter()
    }

    /// Example: L2 distance to another neuroprint – used for
    /// *similar pattern* matching, never for control.
    pub fn l2_distance(&self, other: &TreeOfLifeNeuroprint) -> f32 {
        self.iter_components()
            .zip(other.iter_components())
            .map(|(a, b)| {
                let d = a - b;
                d * d
            })
            .sum::<f32>()
            .sqrt()
    }
}
