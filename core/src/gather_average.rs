use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::gather_records::GatherRecords;
use crate::ledger::{Day, ledger};

/// Each measure averaged over the weigh-ins of a span, the composition as the ledger carries it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GatherAverage {
    pub weigh_ins: usize,
    pub body_mass_kg: f64,
    pub lean_mass_kg: Option<f64>,
    pub fat_mass_kg: Option<f64>,
    pub fat_ratio: Option<f64>,
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let (sum, count) = values.fold((0.0, 0), |(sum, count), value| (sum + value, count + 1));
    (count > 0).then(|| sum / f64::from(count))
}

/// The average from a first day to the last, both included; none without a weigh-in.
pub fn averaged_between(
    first: NaiveDate,
    last: NaiveDate,
    gathered: &GatherRecords,
) -> Option<GatherAverage> {
    let days: Vec<Day> = ledger(&gathered.measurements, &gathered.profiles)
        .into_iter()
        .filter(|measured| (first..=last).contains(&measured.day))
        .collect();
    Some(GatherAverage {
        weigh_ins: days.len(),
        body_mass_kg: mean(days.iter().map(|measured| measured.body_mass_kg))?,
        lean_mass_kg: None,
        fat_mass_kg: None,
        fat_ratio: None,
    })
}
