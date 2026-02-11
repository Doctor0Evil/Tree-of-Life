use chrono::Utc;

/// Simple exponential time discount with 1 day time constant, age in seconds.[file:2]
pub fn time_discount_factor(age_seconds: u64) -> f64 {
    let tau = 86_400.0;
    (-(age_seconds as f64) / tau).exp()
}

pub fn current_timestamp() -> u64 {
    Utc::now().timestamp() as u64
}
