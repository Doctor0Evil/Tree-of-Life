use crate::ledger::{DeedEvent, Ledger};
use crate::simulation::MicroAgent;
use crate::utils::crypto::compute_sha256_hash;
use crate::utils::rand_events::generate_ecological_event;
use crate::utils::time::current_timestamp;
use serde_json::json;
use uuid::Uuid;

/// Non-actuating 1D lattice of MicroAgents plus an internal observer Ledger.[file:1]
pub struct MicroSociety {
    pub agents: Vec<MicroAgent>,
    pub ledger: Ledger,
}

impl MicroSociety {
    pub fn new(agent_count: usize) -> Self {
        let mut agents = Vec::with_capacity(agent_count);
        for i in 0..agent_count {
            agents.push(MicroAgent::new(format!("agent_{}", i)));
        }
        Self {
            agents,
            ledger: Ledger::new(),
        }
    }

    /// One bounded simulation step: local random events, Tree updates, and DeedEvent logging.[file:1]
    pub fn simulate_cycle(&mut self) {
        let agent_len = self.agents.len();
        if agent_len == 0 {
            return;
        }

        let prev_hash = self.ledger.last_hash().to_string();

        for i in 0..agent_len {
            let event_type = generate_ecological_event();
            let is_good = matches!(
                event_type.as_str(),
                "ecological_sharing" | "resource_aid" | "math_science_education"
            );

            self.agents[i].tree_snapshot.update_from_event(is_good);
            self.agents[i].update_predicates();

            let actor_id = self.agents[i].id.clone();
            let target_id = format!("agent_{}", (i + 1) % agent_len);

            let mut deed = DeedEvent {
                event_id: Uuid::new_v4().to_string(),
                timestamp: current_timestamp(),
                prev_hash: prev_hash.clone(),
                self_hash: String::new(),
                actor_id,
                target_ids: vec![target_id],
                deed_type: event_type.clone(),
                tags: vec!["microlife".to_string()],
                context_json: json!({
                    "tree_snapshot": self.agents[i].tree_snapshot,
                    "calmstable": self.agents[i].calmstable,
                    "overloaded": self.agents[i].overloaded,
                    "recovery": self.agents[i].recovery,
                    "unfairdrain": self.agents[i].unfairdrain
                }),
                ethics_flags: if !is_good {
                    vec!["minor_harm".to_string()]
                } else {
                    Vec::new()
                },
                life_harm_flag: false,
            };

            let serialized = serde_json::to_string(&deed).expect("serialize deed");
            let hash = compute_sha256_hash(serialized.as_bytes());
            deed.self_hash = hash;

            self.ledger.append(deed);
        }
    }
}
