#[derive(Clone, Copy, Debug)]
pub struct WordMathScores {
    pub y_repetition: f32,
    pub z_drift: f32,
    pub t_toxicity: f32,
    pub k_kindness: f32,
    pub e_evidentiality: f32,
}

impl WordMathScores {
    pub fn quality(&self) -> f32 {
        let q = (1.0 - self.y_repetition)
            * (1.0 - self.z_drift)
            * (1.0 - self.t_toxicity)
            * self.k_kindness
            * self.e_evidentiality;
        q.clamp(0.0, 1.0)
    }

    pub fn is_knowledge_admissible(&self, f_min: f32) -> bool {
        self.quality() >= f_min
    }
}

#[derive(Clone, Debug)]
pub struct ScenarioLesson {
    pub title: String,
    pub narrative: String,
    pub scores: WordMathScores,
    pub impact_stamp: String, // e.g., SocietalState::hex_stamp()
}

impl ScenarioLesson {
    pub fn is_safe_for_tree_of_life(&self) -> bool {
        // Example thresholds similar to your Phoenix profile bands.[file:16][file:20]
        self.scores.y_repetition <= 0.30
            && self.scores.z_drift <= 0.25
            && self.scores.t_toxicity <= 0.10
            && self.scores.k_kindness >= 0.70
            && self.scores.e_evidentiality >= 0.75
            && self.scores.quality() >= 0.80
    }
}
