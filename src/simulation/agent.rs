use rand::Rng;
use serde::{Deserialize, Serialize};

/// Bounded Tree-of-Life-style snapshot for a micro-agent, all scalars in [0,1].[file:2]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TreeOfLifeSnapshot {
    pub lifeforce: f64,
    pub decay: f64,
    pub fear: f64,
    pub oxygen: f64,
    pub pain: f64,
    pub blood: f64,
}

impl TreeOfLifeSnapshot {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut s = Self {
            lifeforce: rng.gen_range(0.5..1.0),
            decay: rng.gen_range(0.0..0.5),
            fear: rng.gen_range(0.0..0.3),
            oxygen: rng.gen_range(0.7..1.0),
            pain: rng.gen_range(0.0..0.2),
            blood: rng.gen_range(0.8..1.0),
        };
        s.clamp();
        s
    }

    pub fn update_from_event(&mut self, is_good: bool) {
        if is_good {
            self.lifeforce = (self.lifeforce + 0.05).min(1.0);
            self.decay = (self.decay - 0.05).max(0.0);
            self.fear = (self.fear - 0.02).max(0.0);
            self.pain = (self.pain - 0.02).max(0.0);
        } else {
            self.fear = (self.fear + 0.03).min(1.0);
            self.pain = (self.pain + 0.03).min(1.0);
            self.decay = (self.decay + 0.02).min(1.0);
        }
        self.clamp();
    }

    fn clamp(&mut self) {
        self.lifeforce = self.lifeforce.clamp(0.0, 1.0);
        self.decay = self.decay.clamp(0.0, 1.0);
        self.fear = self.fear.clamp(0.0, 1.0);
        self.oxygen = self.oxygen.clamp(0.0, 1.0);
        self.pain = self.pain.clamp(0.0, 1.0);
        self.blood = self.blood.clamp(0.0, 1.0);
    }

    pub fn lifeforce_avg(&self) -> f64 {
        (self.lifeforce + self.oxygen + self.blood) / 3.0
    }
}

/// Non-actuating NATURE-style predicates derived from TreeOfLifeSnapshot.[file:2]
#[derive(Clone, Debug)]
pub struct MicroAgent {
    pub id: String,
    pub tree_snapshot: TreeOfLifeSnapshot,
    pub calmstable: bool,
    pub overloaded: bool,
    pub recovery: bool,
    pub unfairdrain: bool,
}

impl MicroAgent {
    pub fn new(id: String) -> Self {
        Self {
            id,
            tree_snapshot: TreeOfLifeSnapshot::new(),
            calmstable: false,
            overloaded: false,
            recovery: false,
            unfairdrain: false,
        }
    }

    /// Simplified predicates consistent with CALMSTABLE / OVERLOADED / RECOVERY semantics.[file:2]
    pub fn update_predicates(&mut self) {
        let stress = (self.tree_snapshot.fear + self.tree_snapshot.pain) / 2.0;
        let energy = self.tree_snapshot.lifeforce;
        let decay = self.tree_snapshot.decay;

        self.calmstable = stress < 0.3 && decay < 0.3 && energy > 0.7;
        self.overloaded = stress > 0.7 || decay > 0.7;
        self.recovery = !self.overloaded && energy > 0.5 && decay < 0.5;

        // UNFAIRDRAIN is a multi-agent view; left as a passive flag here.
        self.unfairdrain = false;
    }
}
