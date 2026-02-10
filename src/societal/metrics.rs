use crate::societal::model::{GovernanceEnvelope, SocietalState};
use crate::societal::word_math::WordMathScores;

#[derive(Clone, Copy, Debug)]
pub struct SocialImpactScores {
    pub antistigma: f32,
    pub nonexclusion: f32,
    pub peacekeeping: f32,
    pub eco: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SystemOutputFactors {
    // F = sovereignty-biased metascore; O = output factor, same pattern as your ALN spec.[file:16]
    pub f_score: f32,
    pub o_factor: f32,
}

pub fn compute_social_impact_scores(state: &SocietalState) -> SocialImpactScores {
    SocialImpactScores {
        antistigma: state.realized_impact.antistigma,
        nonexclusion: state.realized_impact.nonexclusion,
        peacekeeping: state.realized_impact.peacekeeping,
        eco: state.realized_impact.eco,
    }
}

pub fn compute_system_output(
    wm: &WordMathScores,
    sis: &SocialImpactScores,
) -> SystemOutputFactors {
    let f = wm.quality(); // reuse f(y,z,T,K,E).[file:20]
    let s_bar = (sis.antistigma + sis.nonexclusion + sis.peacekeeping + sis.eco) / 4.0;
    let o = (f * s_bar).clamp(0.0, 1.0);
    SystemOutputFactors { f_score: f, o_factor: o }
}
