use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn_records::BurnRecords;
use crate::measurement::Measurement;
use crate::volitional::Volitional;
use crate::workout::Workout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BurntWorkout {
    pub id: i64,
    pub workout: Workout,
    pub burnt_kcal: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Burn {
    pub day: NaiveDate,
    pub workouts: Vec<BurntWorkout>,
    pub burnt_kcal: Option<f64>,
    pub off_the_clock: Option<Volitional>,
    pub off_the_clock_is_usual: bool,
}

pub fn burn_on(day: NaiveDate, records: &BurnRecords, measurements: &[Measurement]) -> Burn {
    let _ = (records, measurements);
    Burn {
        day,
        workouts: vec![],
        burnt_kcal: None,
        off_the_clock: None,
        off_the_clock_is_usual: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workout::burnt_by;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn weighed(date: &str, body_mass_kg: f64) -> Measurement {
        Measurement {
            day: day(date),
            body_mass_kg,
            waist_cm: None,
        }
    }

    fn walked(date: &str, minutes: f64) -> Workout {
        Workout {
            day: day(date),
            exercise: "Walking".into(),
            met: 4.0,
            minutes,
            reps: None,
            sets: 1,
            load_kg: 0.0,
        }
    }

    fn a_desk_day() -> Volitional {
        Volitional {
            sitting_min: 480.0,
            standing_min: 120.0,
            moving_km: 3.0,
        }
    }

    #[test]
    fn a_day_burns_the_sum_of_its_workouts_at_the_body_mass_in_force() {
        let records = BurnRecords {
            workouts: vec![
                (1, walked("2026-10-10", 30.0)),
                (2, walked("2026-10-10", 15.0)),
                (3, walked("2026-10-11", 60.0)),
            ],
            ..BurnRecords::default()
        };

        let burn = burn_on(
            day("2026-10-10"),
            &records,
            &[weighed("2026-10-01", 80.0), weighed("2026-10-11", 79.0)],
        );

        let expected = burnt_by(&walked("2026-10-10", 30.0), 80.0)
            + burnt_by(&walked("2026-10-10", 15.0), 80.0);
        assert_eq!(burn.workouts.len(), 2);
        assert!(burn.burnt_kcal.is_some_and(|kcal| (kcal - expected).abs() < 1e-9));
    }

    #[test]
    fn workouts_before_the_first_weigh_in_burn_at_the_earliest_body_mass() {
        let records = BurnRecords {
            workouts: vec![(1, walked("2026-09-01", 30.0))],
            ..BurnRecords::default()
        };

        let burn = burn_on(day("2026-09-01"), &records, &[weighed("2026-10-01", 80.0)]);

        assert_eq!(
            burn.burnt_kcal,
            Some(burnt_by(&walked("2026-09-01", 30.0), 80.0))
        );
    }

    #[test]
    fn without_any_weigh_in_a_workout_burns_an_unknown_amount_rather_than_a_stand_in_body() {
        let records = BurnRecords {
            workouts: vec![(1, walked("2026-10-10", 30.0))],
            ..BurnRecords::default()
        };

        let burn = burn_on(day("2026-10-10"), &records, &[]);

        assert_eq!(burn.burnt_kcal, None);
        assert_eq!(burn.workouts.first().map(|done| done.burnt_kcal), Some(None));
    }

    #[test]
    fn a_day_without_workouts_burns_nothing_beyond_its_day_off_the_clock() {
        let burn = burn_on(day("2026-10-10"), &BurnRecords::default(), &[]);

        assert_eq!(burn.burnt_kcal, Some(0.0));
    }

    #[test]
    fn a_day_off_the_clock_of_its_own_comes_before_the_usual_one() {
        let lazy_sunday = Volitional {
            sitting_min: 600.0,
            standing_min: 60.0,
            moving_km: 1.0,
        };
        let records = BurnRecords {
            days_off_the_clock: vec![(day("2026-10-11"), lazy_sunday)],
            usual_days: vec![(day("2026-03-01"), a_desk_day())],
            ..BurnRecords::default()
        };

        let saturday = burn_on(day("2026-10-10"), &records, &[]);
        let sunday = burn_on(day("2026-10-11"), &records, &[]);

        assert_eq!(
            (saturday.off_the_clock, saturday.off_the_clock_is_usual),
            (Some(a_desk_day()), true)
        );
        assert_eq!(
            (sunday.off_the_clock, sunday.off_the_clock_is_usual),
            (Some(lazy_sunday), false)
        );
    }
}
