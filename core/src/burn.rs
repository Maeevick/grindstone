use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn_records::BurnRecords;
use crate::burn_target::target_on;
use crate::gather_records::GatherRecords;
use crate::measurement::Measurement;
use crate::verdict::{Verdict, verdict_of};
use crate::version::version_in_force_on;
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
    /// The body mass in force on the day, the one its workouts are priced on.
    pub body_mass_kg: Option<f64>,
    pub target_kcal: Option<f64>,
    /// Whether the workouts reached the target, as both are shown; nothing without a weigh-in to price them.
    pub verdict: Option<Verdict>,
}

pub(crate) fn body_mass_in_force_on(day: NaiveDate, measurements: &[Measurement]) -> Option<f64> {
    let masses: Vec<(NaiveDate, f64)> = measurements
        .iter()
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    version_in_force_on(&masses, day).copied()
}

pub fn burn_on(day: NaiveDate, records: &BurnRecords, gather: &GatherRecords) -> Burn {
    Burns::of(records, gather).on(day)
}

/// A history's burn read day after day: its workouts grouped by day and its body masses in day order, gathered
/// once rather than for each day.
pub(crate) struct Burns<'records> {
    records: &'records BurnRecords,
    workouts: BTreeMap<NaiveDate, Vec<(i64, &'records Workout)>>,
    masses: Vec<(NaiveDate, f64)>,
}

impl<'records> Burns<'records> {
    pub(crate) fn of(records: &'records BurnRecords, gather: &'records GatherRecords) -> Self {
        let mut masses: Vec<(NaiveDate, f64)> = gather
            .measurements
            .iter()
            .map(|measured| (measured.day, measured.body_mass_kg))
            .collect();
        masses.sort_by_key(|(day, _)| *day);
        Burns {
            records,
            workouts: records
                .workouts
                .iter()
                .fold(BTreeMap::new(), |mut by_day, (id, done)| {
                    by_day
                        .entry(done.day)
                        .or_insert_with(Vec::new)
                        .push((*id, done));
                    by_day
                }),
            masses,
        }
    }

    /// The latest body mass up to the day, the earliest before any.
    fn body_mass_on(&self, day: NaiveDate) -> Option<f64> {
        let up_to = self.masses.partition_point(|(weighed, _)| *weighed <= day);
        up_to
            .checked_sub(1)
            .map_or(self.masses.first(), |latest| self.masses.get(latest))
            .map(|(_, kg)| *kg)
    }

    pub(crate) fn on(&self, day: NaiveDate) -> Burn {
        let body_mass_kg = self.body_mass_on(day);
        let workouts: Vec<BurntWorkout> = self
            .workouts
            .get(&day)
            .into_iter()
            .flatten()
            .map(|(id, done)| BurntWorkout {
                id: *id,
                workout: (*done).clone(),
                burnt_kcal: body_mass_kg.map(|kg| burnt_by(done, kg)),
            })
            .collect();
        let burnt_kcal = workouts
            .iter()
            .map(|done| done.burnt_kcal)
            .sum::<Option<f64>>();
        let target_kcal =
            target_on(day, &self.records.targets, body_mass_kg).map(|target| target.values);
        Burn {
            day,
            burnt_kcal,
            workouts,
            body_mass_kg,
            target_kcal,
            verdict: burnt_kcal
                .zip(target_kcal)
                .map(|(burnt, target)| verdict_of(burnt, target, f64::INFINITY)),
        }
    }
}

/// An entry of a day's burn: a workout on its own, or a circuit run, the workouts logged together under it.
#[derive(Clone, Debug, PartialEq)]
pub enum BurnEntry {
    Workout(BurntWorkout),
    Circuit {
        name: String,
        workouts: Vec<BurntWorkout>,
    },
}

/// A day's entries, newest first: each workout on its own, the steps of a circuit run together, in the order
/// they were logged.
pub fn entries_of(workouts: &[BurntWorkout]) -> Vec<BurnEntry> {
    let mut logged = workouts.to_vec();
    logged.sort_by_key(|done| done.id);
    let mut entries: Vec<BurnEntry> = Vec::new();
    for done in logged {
        match (entries.last_mut(), done.workout.circuit.clone()) {
            (Some(BurnEntry::Circuit { name, workouts }), Some(circuit)) if *name == circuit => {
                workouts.push(done);
            }
            (_, Some(circuit)) => entries.push(BurnEntry::Circuit {
                name: circuit,
                workouts: vec![done],
            }),
            (_, None) => entries.push(BurnEntry::Workout(done)),
        }
    }
    entries.reverse();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn done(id: i64, exercise: &str, circuit: Option<&str>) -> BurntWorkout {
        BurntWorkout {
            id,
            workout: Workout {
                day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a day"),
                exercise: exercise.into(),
                met: 4.0,
                minutes: 1.0,
                reps: None,
                sets: 1,
                load_kg: 0.0,
                circuit: circuit.map(String::from),
            },
            burnt_kcal: Some(10.0),
        }
    }

    #[test]
    fn two_circuits_run_back_to_back_stay_two_entries() {
        let logged = [
            done(1, "Push up", Some("Morning rust")),
            done(2, "Squat", Some("Leg day")),
        ];

        assert_eq!(
            entries_of(&logged),
            [
                BurnEntry::Circuit {
                    name: "Leg day".into(),
                    workouts: vec![logged[1].clone()]
                },
                BurnEntry::Circuit {
                    name: "Morning rust".into(),
                    workouts: vec![logged[0].clone()]
                }
            ]
        );
    }

    #[test]
    fn a_days_workouts_group_each_circuit_run_newest_first_its_steps_in_their_order() {
        let rust = Some("Morning rust");
        let logged = [
            done(1, "Walking", None),
            done(2, "Push up", rust),
            done(3, "Burpee", rust),
            done(4, "Squat", None),
            done(5, "Push up", rust),
            done(6, "Burpee", rust),
        ];
        let run = |ids: [usize; 2]| BurnEntry::Circuit {
            name: "Morning rust".into(),
            workouts: ids.map(|id| logged[id - 1].clone()).to_vec(),
        };

        assert_eq!(
            entries_of(&logged),
            [
                run([5, 6]),
                BurnEntry::Workout(logged[3].clone()),
                run([2, 3]),
                BurnEntry::Workout(logged[0].clone())
            ]
        );
    }
    use crate::kept::Kept;
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
    fn a_days_burn_tells_whether_its_workouts_reached_its_target_as_shown() {
        let walking_for = |minutes| BurnRecords {
            workouts: vec![(1, walked("2026-10-10", minutes))],
            targets: vec![(day("2026-03-01"), Kept::Own(80.0))],
            ..BurnRecords::default()
        };
        let weighed_in = gathered(&[weighed("2026-10-01", 80.0)]);
        let verdict_after =
            |minutes| burn_on(day("2026-10-10"), &walking_for(minutes), &weighed_in).verdict;

        assert_eq!(
            [
                verdict_after(15.0),
                verdict_after(19.9),
                verdict_after(30.0),
                burn_on(day("2026-10-10"), &walking_for(30.0), &gathered(&[])).verdict
            ],
            [
                Some(Verdict::Low),
                Some(Verdict::InBand),
                Some(Verdict::InBand),
                None
            ]
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
    fn a_day_without_workouts_burns_nothing() {
        let burn = burn_on(day("2026-10-10"), &BurnRecords::default(), &gathered(&[]));

        assert_eq!(burn.burnt_kcal, Some(0.0));
    }
}
