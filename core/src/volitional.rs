use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Volitional {
    pub sitting_min: f64,
    pub standing_min: f64,
    pub moving_km: f64,
}
