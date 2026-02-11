use crate::ledger::Ledger;
use crate::simulation::TreeOfLifeSnapshot;
use crate::utils::time::time_discount_factor;
use chrono::Utc;

/// Advisory, non-actuating Church-style account view derived from the ledger.[file:1]
#[derive(Debug, Default, Clone)]
pub struct ChurchAccountState {
    pub cumulative_good_deeds: f64,
    pub cumulative_harm_flags: u32,
    pub eco_score: f64,
    pub debt_ceiling: f64,
    pub church_balance: f64,
    pub existence_rights_score: f64,
}

impl ChurchAccountState {
    /// Compute an advisory rights view from the DeedEvent ledger, never mutating external state.[file:1]
    pub fn compute_from_ledger(ledger: &Ledger, actor_id: &str) -> Option<Self> {
        let events = ledger.events_for_actor(actor_id);
        if events.is_empty() {
            return None;
        }

        let now = Utc::now().timestamp() as u64;

        let mut good_deeds = 0.0;
        let mut harm_flags = 0u32;
        let mut lifeforce_sum = 0.0;
        let mut lifeforce_count = 0.0;

        for event in events {
            let age = now.saturating_sub(event.timestamp);
            let discount = time_discount_factor(age);

            if event.is_good_deed() {
                good_deeds += discount;
            }

            if event.life_harm_flag {
                harm_flags = harm_flags.saturating_add(1);
            }

            if let Ok(wrapper) = serde_json::from_value::<serde_json::Value>(event.context_json.clone()) {
                if let Some(snapshot_val) = wrapper.get("tree_snapshot") {
                    if let Ok(snapshot) =
                        serde_json::from_value::<TreeOfLifeSnapshot>(snapshot_val.clone())
                    {
                        lifeforce_sum += snapshot.lifeforce_avg();
                        lifeforce_count += 1.0;
                    }
                }
            }
        }

        let good_deeds_norm = good_deeds.min(1.0);
        let harm_norm = (harm_flags as f64 / 10.0).min(1.0);
        let eco_score = 0.7 * good_deeds_norm + 0.3 * (1.0 - harm_norm);
        let debt_ceiling = 1.0 - harm_norm;
        let church_balance = good_deeds * 0.1;
        let lifeforce_avg = if lifeforce_count > 0.0 {
            lifeforce_sum / lifeforce_count
        } else {
            0.5
        };

        let existence_raw = eco_score * (1.0 - harm_norm) * lifeforce_avg;
        let existence_rights_score = existence_raw.clamp(0.0, 1.0);

        Some(Self {
            cumulative_good_deeds: good_deeds,
            cumulative_harm_flags: harm_flags,
            eco_score,
            debt_ceiling,
            church_balance,
            existence_rights_score,
        })
    }

    /// Symbolic eco-grant suggestion, advisory only.[file:1]
    pub fn compute_grant_amount(&self) -> f64 {
        self.existence_rights_score * 5.0
    }
}
