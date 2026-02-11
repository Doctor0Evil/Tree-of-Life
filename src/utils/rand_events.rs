use rand::Rng;

/// Bounded ecological event types for advisory sandbox use only.[file:1]
pub fn generate_ecological_event() -> String {
    let events = [
        "ecological_sharing",
        "resource_aid",
        "minor_disturbance",
        "math_science_education",
    ];
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..events.len());
    events[idx].to_string()
}
