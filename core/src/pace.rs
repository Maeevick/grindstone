use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::gather_records::GatherRecords;
use crate::tendency::Change;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Judgement {
    Holding,
    VeryGood,
    Good,
    Stable,
    WrongWay,
    GainingTooFast,
    LosingTooFast,
    UnderHealthyMass,
    UnderHealthyFat,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pace {
    pub body_mass: Change,
    pub lean_mass: Option<Change>,
    pub fat_mass: Option<Change>,
    pub fat_ratio: Option<Change>,
    pub judgement: Judgement,
    pub safe_loss_kg: f64,
}

pub fn pace_on(_day: NaiveDate, _gathered: &GatherRecords) -> Option<Pace> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;

    use crate::gender::Gender;
    use crate::measurement::Measurement;
    use crate::profile::Profile;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn a_man_weighed(from_kg: f64, kg_a_day: f64) -> GatherRecords {
        let last = day("2026-10-10");
        GatherRecords {
            profiles: vec![(
                day("2026-03-01"),
                Profile {
                    birthday: day("1985-04-12"),
                    gender: Gender::Male,
                    height_cm: 180.0,
                    neck_cm: 38.5,
                    hip_cm: None,
                },
            )],
            measurements: (0..21)
                .map(|at| Measurement {
                    day: last - Days::new(20 - at),
                    body_mass_kg: from_kg + kg_a_day * at as f64,
                    waist_cm: Some(82.0),
                })
                .collect(),
            goals: vec![],
        }
    }

    #[test]
    fn a_weeks_pace_is_the_slope_of_the_seven_day_average_over_the_last_fourteen_days() {
        let pace = pace_on(day("2026-10-10"), &a_man_weighed(80.0, -0.05)).expect("a pace");

        assert!(
            (pace.body_mass.by - -0.35).abs() < 1e-9,
            "{}",
            pace.body_mass.by
        );
    }
}
