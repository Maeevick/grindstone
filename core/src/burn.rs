use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn_records::BurnRecords;
use crate::burn_target::target_on;
use crate::gather_records::GatherRecords;
use crate::kept::{InForce, Kept, in_force};
use crate::measurement::Measurement;
use crate::profile::Profile;
use crate::recommendations::recommended;
use crate::version::{dated_version_in_force_on, version_in_force_on};
use crate::volitional::Volitional;
use crate::workout::{Workout, burnt_by};

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
    /// The body mass in force on the day, the one its workouts are priced on.
    pub body_mass_kg: Option<f64>,
    pub target_kcal: Option<f64>,
}

pub(crate) fn body_mass_in_force_on(day: NaiveDate, measurements: &[Measurement]) -> Option<f64> {
    let masses: Vec<(NaiveDate, f64)> = measurements
        .iter()
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    version_in_force_on(&masses, day).copied()
}

/// The usual day off the clock in force on a day: the one kept from the latest day up to it (the earliest
/// before any), its own or the recommendation for the age in force; never kept, the recommendation; nothing
/// without a profile to recommend for.
pub(crate) fn usual_day_in_force(
    day: NaiveDate,
    usual_days: &[(NaiveDate, Kept<Volitional>)],
    profiles: &[(NaiveDate, Profile)],
) -> Option<InForce<Volitional>> {
    in_force(dated_version_in_force_on(usual_days, day), || {
        version_in_force_on(profiles, day).map(|profile| recommended(profile, day, None).usual_day)
    })
}

pub fn burn_on(day: NaiveDate, records: &BurnRecords, gather: &GatherRecords) -> Burn {
    let body_mass_kg = body_mass_in_force_on(day, &gather.measurements);
    let workouts: Vec<BurntWorkout> = records
        .workouts
        .iter()
        .filter(|(_, done)| done.day == day)
        .map(|(id, done)| BurntWorkout {
            id: *id,
            workout: done.clone(),
            burnt_kcal: body_mass_kg.map(|kg| burnt_by(done, kg)),
        })
        .collect();
    let own_day = records
        .days_off_the_clock
        .iter()
        .find(|(kept_on, _)| *kept_on == day)
        .map(|(_, volitional)| *volitional);
    Burn {
        day,
        burnt_kcal: workouts
            .iter()
            .map(|done| done.burnt_kcal)
            .sum::<Option<f64>>(),
        workouts,
        off_the_clock: own_day.or_else(|| {
            usual_day_in_force(day, &records.usual_days, &gather.profiles).map(|usual| usual.values)
        }),
        off_the_clock_is_usual: own_day.is_none(),
        body_mass_kg,
        target_kcal: target_on(day, &records.targets, body_mass_kg).map(|target| target.values),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workout::burnt_by;

    fn gathered(measurements: &[Measurement]) -> GatherRecords {
        GatherRecords {
            measurements: measurements.to_vec(),
            ..GatherRecords::default()
        }
    }

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
            circuit: None,
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
    fn a_days_burn_carries_the_target_in_force_on_its_body_mass() {
        let records = BurnRecords {
            targets: vec![(day("2026-10-15"), Kept::Own(150.0))],
            ..BurnRecords::default()
        };
        let measurements = [weighed("2026-10-01", 72.0)];

        assert_eq!(
            (
                burn_on(day("2026-10-10"), &records, &gathered(&measurements)).target_kcal,
                burn_on(day("2026-10-20"), &records, &gathered(&measurements)).target_kcal
            ),
            (
                crate::burn_target::target_on(day("2026-10-10"), &[], Some(72.0))
                    .map(|target| target.values),
                Some(150.0)
            )
        );
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
            &gathered(&[weighed("2026-10-01", 80.0), weighed("2026-10-11", 79.0)]),
        );

        let expected = burnt_by(&walked("2026-10-10", 30.0), 80.0)
            + burnt_by(&walked("2026-10-10", 15.0), 80.0);
        assert_eq!(burn.workouts.len(), 2);
        assert!(
            burn.burnt_kcal
                .is_some_and(|kcal| (kcal - expected).abs() < 1e-9)
        );
    }

    #[test]
    fn workouts_before_the_first_weigh_in_burn_at_the_earliest_body_mass() {
        let records = BurnRecords {
            workouts: vec![(1, walked("2026-09-01", 30.0))],
            ..BurnRecords::default()
        };

        let burn = burn_on(
            day("2026-09-01"),
            &records,
            &gathered(&[weighed("2026-10-01", 80.0)]),
        );

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

        let burn = burn_on(day("2026-10-10"), &records, &gathered(&[]));

        assert_eq!(burn.burnt_kcal, None);
        assert_eq!(
            burn.workouts.first().map(|done| done.burnt_kcal),
            Some(None)
        );
    }

    #[test]
    fn a_day_carries_the_body_mass_its_workouts_are_priced_on_and_none_without_a_weigh_in() {
        let weighed_in = [weighed("2026-10-01", 80.0), weighed("2026-10-11", 79.0)];

        let priced = burn_on(
            day("2026-10-10"),
            &BurnRecords::default(),
            &gathered(&weighed_in),
        );
        let unweighed = burn_on(day("2026-10-10"), &BurnRecords::default(), &gathered(&[]));

        assert_eq!(
            (priced.body_mass_kg, unweighed.body_mass_kg),
            (Some(80.0), None)
        );
    }

    #[test]
    fn a_day_without_workouts_burns_nothing_beyond_its_day_off_the_clock() {
        let burn = burn_on(day("2026-10-10"), &BurnRecords::default(), &gathered(&[]));

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
            usual_days: vec![(day("2026-03-01"), Kept::Own(a_desk_day()))],
            ..BurnRecords::default()
        };

        let saturday = burn_on(day("2026-10-10"), &records, &gathered(&[]));
        let sunday = burn_on(day("2026-10-11"), &records, &gathered(&[]));

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
