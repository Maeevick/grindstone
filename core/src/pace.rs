use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::gather_records::GatherRecords;
use crate::progress::progress_towards;
use crate::tendency::{Change, Heading};
use crate::trend::seven_day_trend;

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

const DAYS_OF_THE_SLOPE: u64 = 13;
const DAYS_OF_THE_AVERAGE: u64 = 6;

/// The week's pace on a day: the slope, per week, of the seven-day average of the body mass over the
/// fourteen days ending on it.
pub fn pace_on(day: NaiveDate, gathered: &GatherRecords) -> Option<Pace> {
    let readings: Vec<(NaiveDate, f64)> = gathered
        .measurements
        .iter()
        .filter(|measured| {
            measured.day <= day
                && measured.day + Days::new(DAYS_OF_THE_SLOPE + DAYS_OF_THE_AVERAGE) >= day
        })
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    let trend: Vec<(NaiveDate, f64)> = seven_day_trend(&readings)
        .into_iter()
        .filter(|(averaged, _)| *averaged + Days::new(DAYS_OF_THE_SLOPE) >= day)
        .collect();
    let progress = progress_towards(None, &trend)?;
    Some(Pace {
        body_mass: Change {
            by: progress.per_week,
            heading: Heading::Unaimed,
        },
        lean_mass: None,
        fat_mass: None,
        fat_ratio: None,
        judgement: Judgement::Stable,
        safe_loss_kg: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
