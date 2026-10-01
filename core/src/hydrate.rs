use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn::burn_on;
use crate::burn_records::BurnRecords;
use crate::drinking::Drinking;
use crate::gather_records::GatherRecords;
use crate::hydrate_base::base_on;
use crate::hydrate_records::HydrateRecords;
use crate::verdict::{Verdict, verdict_of};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hydrate {
    pub day: NaiveDate,
    pub drinkings: Vec<(i64, Drinking)>,
    pub drunk_ml: f64,
    pub target_ml: f64,
    pub floor_ml: f64,
    pub cap_ml: f64,
    pub verdict: Verdict,
}

const ML_PER_KCAL_BURNT: f64 = 1.0;
const BAND_SHARE_EITHER_SIDE: f64 = 0.10;

pub fn hydrate_on(
    day: NaiveDate,
    records: &HydrateRecords,
    gather: &GatherRecords,
    burn: &BurnRecords,
) -> Hydrate {
    let drinkings: Vec<(i64, Drinking)> = records
        .drinkings
        .iter()
        .filter(|(_, drunk)| drunk.day == day)
        .cloned()
        .collect();
    let drunk_ml: f64 = drinkings.iter().map(|(_, drunk)| drunk.volume_ml).sum();
    let burnt_kcal = burn_on(day, burn, &gather.measurements)
        .burnt_kcal
        .unwrap_or(0.0);
    let target_ml =
        base_on(day, &records.bases, &gather.profiles).base_ml + burnt_kcal * ML_PER_KCAL_BURNT;
    let floor_ml = target_ml * (1.0 - BAND_SHARE_EITHER_SIDE);
    let cap_ml = target_ml * (1.0 + BAND_SHARE_EITHER_SIDE);
    Hydrate {
        day,
        drinkings,
        drunk_ml,
        target_ml,
        floor_ml,
        cap_ml,
        verdict: verdict_of(drunk_ml, floor_ml, cap_ml),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measurement::Measurement;
    use crate::workout::Workout;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    fn drunk(date: &str, drink: &str, volume_ml: f64) -> Drinking {
        Drinking {
            day: day(date),
            drink: drink.into(),
            volume_ml,
            kind: crate::drink::DrinkKind::Other,
            servings: 1.0,
        }
    }

    fn based_on(base_ml: f64, drinkings: Vec<(i64, Drinking)>) -> HydrateRecords {
        HydrateRecords {
            bases: vec![(day("2026-03-01"), base_ml)],
            drinkings,
            ..HydrateRecords::default()
        }
    }

    fn walked_an_hour(date: &str) -> BurnRecords {
        BurnRecords {
            workouts: vec![(
                1,
                Workout {
                    day: day(date),
                    exercise: "Walking".into(),
                    met: 4.0,
                    minutes: 60.0,
                    reps: None,
                    sets: 1,
                    load_kg: 0.0,
                    circuit: None,
                },
            )],
            ..BurnRecords::default()
        }
    }

    fn weighed(date: &str, body_mass_kg: f64) -> GatherRecords {
        GatherRecords {
            measurements: vec![Measurement {
                day: day(date),
                body_mass_kg,
                waist_cm: None,
            }],
            ..GatherRecords::default()
        }
    }

    #[test]
    fn a_day_drinks_the_sum_of_its_own_drinkings() {
        let records = based_on(
            2000.0,
            vec![
                (1, drunk("2026-10-10", "Expresso", 80.0)),
                (2, drunk("2026-10-10", "Tea", 200.0)),
                (3, drunk("2026-10-11", "Tea", 400.0)),
            ],
        );

        let hydrate = hydrate_on(
            day("2026-10-10"),
            &records,
            &GatherRecords::default(),
            &BurnRecords::default(),
        );

        assert_eq!(
            hydrate.drinkings,
            vec![
                (1, drunk("2026-10-10", "Expresso", 80.0)),
                (2, drunk("2026-10-10", "Tea", 200.0)),
            ]
        );
        assert!(about(hydrate.drunk_ml, 280.0));
    }

    #[test]
    fn a_day_aims_for_its_base_and_a_millilitre_more_per_kcal_its_workouts_burnt() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &based_on(2000.0, vec![]),
            &weighed("2026-10-01", 80.0),
            &walked_an_hour("2026-10-10"),
        );

        assert!(about(hydrate.target_ml, 2000.0 + 252.0));
    }

    #[test]
    fn a_burn_of_unknown_size_without_any_weigh_in_adds_nothing_to_the_target() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &based_on(2000.0, vec![]),
            &GatherRecords::default(),
            &walked_an_hour("2026-10-10"),
        );

        assert!(about(hydrate.target_ml, 2000.0));
    }

    #[test]
    fn with_no_base_set_a_day_aims_for_the_default_base() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &GatherRecords::default(),
            &BurnRecords::default(),
        );

        assert!(about(hydrate.target_ml, 2000.0));
    }

    #[test]
    fn a_day_reads_its_band_from_twenty_percent_under_to_twenty_percent_over_its_target() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &based_on(2000.0, vec![]),
            &GatherRecords::default(),
            &BurnRecords::default(),
        );

        assert!(about(hydrate.floor_ml, 1600.0));
        assert!(about(hydrate.cap_ml, 2400.0));
    }

    #[test]
    fn a_day_drunk_under_its_band_reads_low_inside_it_in_band_and_over_it_high() {
        let records = based_on(
            500.0,
            vec![
                (1, drunk("2026-10-09", "Tea", 350.0)),
                (2, drunk("2026-10-10", "Tea", 300.0)),
                (3, drunk("2026-10-10", "Tea", 200.0)),
                (4, drunk("2026-10-11", "Tea", 650.0)),
            ],
        );
        let verdict_on = |date| {
            hydrate_on(
                day(date),
                &records,
                &GatherRecords::default(),
                &BurnRecords::default(),
            )
            .verdict
        };

        assert_eq!(
            [
                verdict_on("2026-10-09"),
                verdict_on("2026-10-10"),
                verdict_on("2026-10-11")
            ],
            [Verdict::Low, Verdict::InBand, Verdict::High]
        );
    }
}
