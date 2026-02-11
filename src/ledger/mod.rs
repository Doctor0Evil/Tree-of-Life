mod deed_event;
mod account;

pub use account::ChurchAccountState;
pub use deed_event::DeedEvent;

#[derive(Default, Debug)]
pub struct Ledger {
    events: Vec<DeedEvent>,
    last_hash: String,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            last_hash: String::new(),
        }
    }

    pub fn append(&mut self, event: DeedEvent) {
        if !self.events.is_empty() && event.prev_hash != self.last_hash {
            panic!("Invalid prev_hash for DeedEvent");
        }
        self.last_hash = event.self_hash.clone();
        self.events.push(event);
    }

    pub fn last_hash(&self) -> &str {
        &self.last_hash
    }

    pub fn events_for_actor(&self, actor_id: &str) -> Vec<&DeedEvent> {
        self.events
            .iter()
            .filter(|e| e.actor_id == actor_id)
            .collect()
    }

    pub fn all_events(&self) -> &[DeedEvent] {
        &self.events
    }
}
