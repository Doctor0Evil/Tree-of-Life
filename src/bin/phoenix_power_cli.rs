use std::fmt;

use societal_sim::societal::model::SocietalState;
use societal_sim::examples::phoenix_power_nano::{
    phoenix_power_grid_nano_scenario_fast,
    phoenix_power_grid_nano_scenario_slow,
    run_steps,
};

struct Row {
    step: usize,
    adoption_fast: f32,
    stab_fast: f32,
    unrest_fast: f32,
    frag_fast: f32,
    adoption_slow: f32,
    stab_slow: f32,
    unrest_slow: f32,
    frag_slow: f32,
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
" {step:>3} | {af:>5.2} | {sf:>5.2} | {uf:>5.2} | {ff:>5.2} || {as_:>5.2} | {ss:>5.2} | {us:>5.2} | {fs:>5.2}",
            step = self.step,
            af = self.adoption_fast,
            sf = self.stab_fast,
            uf = self.unrest_fast,
            ff = self.frag_fast,
            as_ = self.adoption_slow,
            ss = self.stab_slow,
            us = self.unrest_slow,
            fs = self.frag_slow,
        )
    }
}

fn main() {
    let steps = 16;

    let (state_fast, scenario_fast) = phoenix_power_grid_nano_scenario_fast();
    let (state_slow, scenario_slow) = phoenix_power_grid_nano_scenario_slow();

    let seq_fast = run_steps(state_fast, scenario_fast, steps);
    let seq_slow = run_steps(state_slow, scenario_slow, steps);

    println!(
"Step |  A_f |  S_f |  U_f |  F_f ||  A_s |  S_s |  U_s |  F_s
-----+------+------+------+------+------+------+------+------+"
    );

    let mut adoption_fast = scenario_fast.adoption_rate;
    let mut adoption_slow = scenario_slow.adoption_rate;

    for i in 0..steps {
        let sf: &SocietalState = &seq_fast[i];
        let ss: &SocietalState = &seq_slow[i];

        let row = Row {
            step: i,
            adoption_fast,
            stab_fast: sf.stability,
            unrest_fast: sf.unrest_risk,
            frag_fast: sf.fragility,
            adoption_slow,
            stab_slow: ss.stability,
            unrest_slow: ss.unrest_risk,
            frag_slow: ss.fragility,
        };

        println!("{row}");

        // Naive approximation: keep adoption in sync with the engine.
        // For teaching, it's enough that students see fast vs slow.
        adoption_fast = (adoption_fast + scenario_fast.rollout_speed * (1.0 - adoption_fast))
            .clamp(0.0, 1.0);
        adoption_slow = (adoption_slow + scenario_slow.rollout_speed * (1.0 - adoption_slow))
            .clamp(0.0, 1.0);
    }
}
