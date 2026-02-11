use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Hash-linked, append-only DeedEvent for MicroSociety sandbox.[file:1]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeedEvent {
    pub event_id: String,
    pub timestamp: u64,
    pub prev_hash: String,
    #[serde(skip_serializing)]
    pub self_hash: String,
    pub actor_id: String,
    pub target_ids: Vec<String>,
    pub deed_type: String,
    pub tags: Vec<String>,
    pub context_json: Value,
    pub ethics_flags: Vec<String>,
    pub life_harm_flag: bool,
}

impl DeedEvent {
    pub fn is_good_deed(&self) -> bool {
        !self.life_harm_flag
            && self.ethics_flags.is_empty()
            && self.tags.contains(&"microlife".to_string())
            && matches!(
                self.deed_type.as_str(),
                "ecological_sharing" | "resource_aid" | "math_science_education"
            )
    }
}
