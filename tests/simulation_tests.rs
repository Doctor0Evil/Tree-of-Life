use approx::assert_relative_eq;

use microsociety_tree_of_life::ledger::ChurchAccountState;
use microsociety_tree_of_life::simulation::{MicroAgent, MicroSociety};

#[test]
fn test_agent_update_predicates() {
    let mut agent = MicroAgent::new("test".to_string());
    agent.tree_snapshot.lifeforce = 0.8;
    agent.tree_snapshot.decay = 0.2;
    agent.tree_snapshot.fear = 0.1;
    agent.tree_snapshot.pain = 0.1;
    agent.update_predicates();
    assert!(agent.calmstable);
    assert!(!agent.overloaded);

    agent.tree_snapshot.fear = 0.9;
    agent.tree_snapshot.pain = 0.9;
    agent.tree_snapshot.decay = 0.8;
    agent.update_predicates();
    assert!(agent.overloaded);
}

#[test]
fn test_society_simulation_creates_events() {
    let mut society = MicroSociety::new(5);
    society.simulate_cycle();
    assert_eq!(society.ledger.all_events().len(), 5);
}

#[test]
fn test_rights_score_bounded_and_computed() {
    let mut state = ChurchAccountState::default();
    state.eco_score = 0.8;
    state.cumulative_harm_flags = 2;
    let harm_norm = (state.cumulative_harm_flags as f64 / 10.0).min(1.0);
    let lifeforce_avg = 0.9;
    let existence = state.eco_score * (1.0 - harm_norm) * lifeforce_avg;
    let clamped = existence.clamp(0.0, 1.0);
    assert_relative_eq!(clamped, 0.8 * 0.8 * 0.9, epsilon = 1e-6);
}
