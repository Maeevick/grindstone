use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::gather_records::GatherRecords;

/// Each measure averaged over the weigh-ins of a span, the composition over those with a waist.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GatherAverage {
    pub weigh_ins: usize,
    pub body_mass_kg: f64,
    pub lean_mass_kg: Option<f64>,
    pub fat_mass_kg: Option<f64>,
    pub fat_ratio: Option<f64>,
}

/// The average from a first day to the last, both included; none without a weigh-in.
pub fn averaged_between(
    _first: NaiveDate,
    _last: NaiveDate,
    _gathered: &GatherRecords,
) -> Option<GatherAverage> {
    None
}
