use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn::{Burn, burn_on};
use crate::circuit::{Circuit, StepForm};
use crate::exercise::Exercise;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::refusal::Refusal;
use crate::volitional::volitional;
use crate::workout::{Workout, workout};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkoutForm {
    pub date: String,
    pub exercise: String,
    pub minutes: Option<f64>,
    pub reps: Option<u32>,
    pub sets: Option<u32>,
    pub load_kg: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CircuitSessionForm {
    pub date: String,
    pub circuit: String,
    pub sets: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionForm {
    pub date: String,
    pub steps: Vec<StepForm>,
    pub sets: Option<u32>,
}

pub const DEFAULT_MINUTES: f64 = 1.0;
pub const DEFAULT_REPS: u32 = 1;
pub const DEFAULT_SETS: u32 = 1;
pub const DEFAULT_LOAD_KG: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct OffTheClockForm {
    pub sitting_min: f64,
    pub standing_min: f64,
    pub moving_km: f64,
}

fn day(date: &str) -> Result<NaiveDate, Failure> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Failure::Refused(Refusal::NotADate))
}

fn burn_of_the_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    day: NaiveDate,
) -> Result<Burn, Failure> {
    let measurements = gather.gather_records()?.measurements;
    Ok(burn_on(day, &burn.burn_records()?, &measurements))
}

pub fn burn_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    date: &str,
) -> Result<Burn, Failure> {
    burn_of_the_day(gather, burn, day(date)?)
}

pub fn log_workout(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    form: &WorkoutForm,
) -> Result<Burn, Failure> {
    let exercise = burn
        .burn_records()?
        .exercises
        .into_iter()
        .find(|known| known.name == form.exercise)
        .ok_or(Failure::Refused(Refusal::NotAnExercise))?;
    let done = workout(
        &form.date,
        &exercise,
        form.minutes.unwrap_or(DEFAULT_MINUTES),
        form.reps.or(Some(DEFAULT_REPS)),
        form.sets.unwrap_or(DEFAULT_SETS),
        form.load_kg.unwrap_or(DEFAULT_LOAD_KG),
    )
    .map_err(Failure::Refused)?;
    burn.save_workout(&done)?;
    burn_of_the_day(gather, burn, done.day)
}

pub fn remove_workout(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    date: &str,
    id: i64,
) -> Result<Burn, Failure> {
    let day = day(date)?;
    burn.remove_workout(id)?;
    burn_of_the_day(gather, burn, day)
}

pub fn set_day_off_the_clock(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    date: &str,
    form: &OffTheClockForm,
) -> Result<Burn, Failure> {
    let day = day(date)?;
    let off_the_clock = volitional(form.sitting_min, form.standing_min, form.moving_km)
        .map_err(Failure::Refused)?;
    burn.save_day_off_the_clock(day, &off_the_clock)?;
    burn_of_the_day(gather, burn, day)
}

fn workouts_of_the_session(
    crafted: &Circuit,
    form: &CircuitSessionForm,
) -> Result<Vec<Workout>, Refusal> {
    crafted
        .steps
        .iter()
        .map(|step| {
            let exercise = Exercise {
                name: step.exercise.clone(),
                met: step.met,
                description: String::new(),
            };
            workout(
                &form.date,
                &exercise,
                step.minutes,
                step.reps,
                form.sets.unwrap_or(DEFAULT_SETS),
                step.load_kg,
            )
            .map(|done| Workout {
                circuit: Some(crafted.name.clone()),
                ..done
            })
        })
        .collect()
}

pub fn log_circuit(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    form: &CircuitSessionForm,
) -> Result<Burn, Failure> {
    let day = day(&form.date)?;
    let crafted = burn
        .burn_records()?
        .circuits
        .into_iter()
        .find(|known| known.name == form.circuit)
        .ok_or(Failure::Refused(Refusal::NotACircuit))?;
    for done in workouts_of_the_session(&crafted, form).map_err(Failure::Refused)? {
        burn.save_workout(&done)?;
    }
    burn_of_the_day(gather, burn, day)
}

/// Each step of the session as a workout of its day, done as many sets as the session; none when
/// one of them cannot be.
pub fn log_session(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    form: &SessionForm,
) -> Result<Burn, Failure> {
    let exercises = burn.burn_records()?.exercises;
    let sets = form.sets.unwrap_or(DEFAULT_SETS);
    let done = form
        .steps
        .iter()
        .map(|step| {
            let exercise = exercises
                .iter()
                .find(|known| known.name == step.exercise)
                .ok_or(Failure::Refused(Refusal::NotAnExercise))?;
            workout(
                &form.date,
                exercise,
                step.minutes,
                step.reps,
                sets,
                step.load_kg,
            )
            .map_err(Failure::Refused)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for each in &done {
        burn.save_workout(each)?;
    }
    burn_of_the_day(gather, burn, day(&form.date)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::{StepForm, burnt_per_set};
    use crate::exercise::Exercise;
    use crate::for_cataloguing::{
        CircuitForm, add_exercise, circuits, craft_circuit, remove_circuit,
    };
    use crate::for_tracking::log_measurement;
    use crate::testing::{BrokenBurnRecords, BurnRecordsInMemory, GatherRecordsInMemory};
    use crate::volitional::Volitional;
    use crate::workout::Workout;

    fn a_catalog_with_walking() -> BurnRecordsInMemory {
        let burn = BurnRecordsInMemory::default();
        burn.save_exercise(&Exercise {
            name: "Walking".into(),
            met: 4.0,
            description: String::new(),
        })
        .expect("an exercise saves");
        burn
    }

    fn walking(date: &str, minutes: f64) -> WorkoutForm {
        WorkoutForm {
            date: date.into(),
            exercise: "Walking".into(),
            minutes: Some(minutes),
            reps: None,
            sets: Some(1),
            load_kg: Some(0.0),
        }
    }

    #[test]
    fn a_logged_workout_comes_back_in_its_days_burn_with_what_it_burnt() {
        let gather = GatherRecordsInMemory::default();
        log_measurement(&gather, "2026-10-10", 80.0, None).expect("a measurement logs");
        let burn = a_catalog_with_walking();

        let burnt = log_workout(&gather, &burn, &walking("2026-10-10", 60.0)).expect("a burn");

        assert_eq!(burnt.workouts.len(), 1);
        assert!(
            burnt
                .burnt_kcal
                .is_some_and(|kcal| (kcal - 252.0).abs() < 1e-9)
        );
    }

    #[test]
    fn a_workout_of_an_exercise_missing_from_the_catalog_is_refused() {
        let refused = log_workout(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &walking("2026-10-10", 60.0),
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotAnExercise)));
    }

    #[test]
    fn removing_a_workout_leaves_the_others_of_its_day() {
        let gather = GatherRecordsInMemory::default();
        let burn = a_catalog_with_walking();
        log_workout(&gather, &burn, &walking("2026-10-10", 30.0)).expect("a workout logs");
        let logged = log_workout(&gather, &burn, &walking("2026-10-10", 15.0)).expect("a burn");
        let first = logged
            .workouts
            .first()
            .map(|done| done.id)
            .expect("a first workout");

        let burnt = remove_workout(&gather, &burn, "2026-10-10", first).expect("a burn");

        assert_eq!(
            burnt
                .workouts
                .iter()
                .map(|done| done.workout.minutes)
                .collect::<Vec<_>>(),
            vec![15.0]
        );
    }

    #[test]
    fn a_day_off_the_clock_set_for_a_day_is_that_days_own() {
        let burnt = set_day_off_the_clock(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            "2026-10-10",
            &OffTheClockForm {
                sitting_min: 600.0,
                standing_min: 60.0,
                moving_km: 1.0,
            },
        )
        .expect("a burn");

        assert_eq!(
            (burnt.off_the_clock, burnt.off_the_clock_is_usual),
            (
                Some(Volitional {
                    sitting_min: 600.0,
                    standing_min: 60.0,
                    moving_km: 1.0,
                }),
                false
            )
        );
    }

    #[test]
    fn a_days_burn_reads_back_what_was_logged_on_it() {
        let gather = GatherRecordsInMemory::default();
        let burn = a_catalog_with_walking();
        log_workout(&gather, &burn, &walking("2026-10-10", 30.0)).expect("a workout logs");

        let burnt = burn_day(&gather, &burn, "2026-10-10").expect("a burn");

        assert_eq!(burnt.workouts.len(), 1);
    }

    #[test]
    fn a_day_the_calendar_lacks_or_broken_records_give_no_burn() {
        let gather = GatherRecordsInMemory::default();

        assert_eq!(
            burn_day(&gather, &BurnRecordsInMemory::default(), "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            burn_day(&gather, &BrokenBurnRecords, "2026-10-10"),
            Err(Failure::Broken("the disk is full".into()))
        );
        assert_eq!(
            remove_workout(&gather, &BrokenBurnRecords, "2026-10-10", 1),
            Err(Failure::Broken("the disk is full".into()))
        );
        assert_eq!(
            set_day_off_the_clock(
                &gather,
                &BurnRecordsInMemory::default(),
                "2026-10-10",
                &OffTheClockForm {
                    sitting_min: -1.0,
                    standing_min: 0.0,
                    moving_km: 0.0,
                },
            ),
            Err(Failure::Refused(Refusal::NotADistance))
        );
    }

    fn a_lab_with_morning_rust() -> BurnRecordsInMemory {
        let burn = BurnRecordsInMemory::default();
        add_exercise(&burn, "Push up", 3.8, "").expect("an exercise adds");
        add_exercise(&burn, "Burpee", 8.0, "").expect("an exercise adds");
        craft_circuit(
            &burn,
            &CircuitForm {
                name: "Morning rust".into(),
                description: String::new(),
                steps: vec![
                    StepForm {
                        exercise: "Push up".into(),
                        reps: Some(12),
                        minutes: 0.5,
                        load_kg: 0.0,
                    },
                    StepForm {
                        exercise: "Burpee".into(),
                        reps: Some(10),
                        minutes: 1.0,
                        load_kg: 5.0,
                    },
                ],
            },
        )
        .expect("a circuit crafts");
        burn
    }

    fn running(circuit: &str, sets: u32) -> CircuitSessionForm {
        CircuitSessionForm {
            date: "2026-10-10".into(),
            circuit: circuit.into(),
            sets: Some(sets),
        }
    }

    #[test]
    fn a_circuit_logged_files_one_workout_per_step_each_done_as_many_sets_as_the_circuit() {
        let the_day = NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day");

        let burnt = log_circuit(
            &GatherRecordsInMemory::default(),
            &a_lab_with_morning_rust(),
            &running("Morning rust", 3),
        )
        .expect("a burn");

        assert_eq!(
            burnt
                .workouts
                .into_iter()
                .map(|done| done.workout)
                .collect::<Vec<_>>(),
            vec![
                Workout {
                    day: the_day,
                    exercise: "Push up".into(),
                    met: 3.8,
                    minutes: 0.5,
                    reps: Some(12),
                    sets: 3,
                    load_kg: 0.0,
                    circuit: Some("Morning rust".into()),
                },
                Workout {
                    day: the_day,
                    exercise: "Burpee".into(),
                    met: 8.0,
                    minutes: 1.0,
                    reps: Some(10),
                    sets: 3,
                    load_kg: 5.0,
                    circuit: Some("Morning rust".into()),
                },
            ]
        );
    }

    fn a_session(steps: &[(&str, Option<u32>, f64, f64)], sets: Option<u32>) -> SessionForm {
        SessionForm {
            date: "2026-10-10".into(),
            steps: steps
                .iter()
                .map(|(exercise, reps, minutes, load_kg)| StepForm {
                    exercise: (*exercise).into(),
                    reps: *reps,
                    minutes: *minutes,
                    load_kg: *load_kg,
                })
                .collect(),
            sets,
        }
    }

    #[test]
    fn a_session_logs_one_workout_per_step_each_done_as_many_sets_as_the_session() {
        let the_day = NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day");

        let burnt = log_session(
            &GatherRecordsInMemory::default(),
            &a_lab_with_morning_rust(),
            &a_session(
                &[("Push up", Some(12), 0.5, 0.0), ("Burpee", None, 1.0, 5.0)],
                Some(3),
            ),
        )
        .expect("a burn");

        assert_eq!(
            burnt
                .workouts
                .into_iter()
                .map(|done| done.workout)
                .collect::<Vec<_>>(),
            vec![
                Workout {
                    day: the_day,
                    exercise: "Push up".into(),
                    met: 3.8,
                    minutes: 0.5,
                    reps: Some(12),
                    sets: 3,
                    load_kg: 0.0,
                    circuit: None,
                },
                Workout {
                    day: the_day,
                    exercise: "Burpee".into(),
                    met: 8.0,
                    minutes: 1.0,
                    reps: None,
                    sets: 3,
                    load_kg: 5.0,
                    circuit: None,
                },
            ]
        );
    }

    #[test]
    fn a_circuit_logged_burns_its_sets_times_what_a_set_burns_at_the_body_mass_in_force() {
        let gather = GatherRecordsInMemory::default();
        log_measurement(&gather, "2026-10-10", 80.0, None).expect("a measurement logs");
        let burn = a_lab_with_morning_rust();
        let morning_rust = circuits(&burn)
            .ok()
            .and_then(|crafted| crafted.into_iter().next())
            .expect("a circuit");

        let burnt = log_circuit(&gather, &burn, &running("Morning rust", 3)).expect("a burn");

        assert!(
            burnt
                .burnt_kcal
                .is_some_and(|kcal| (kcal - 3.0 * burnt_per_set(&morning_rust, 80.0)).abs() < 1e-9)
        );
    }

    #[test]
    fn a_circuit_missing_from_the_catalog_or_run_without_a_set_or_on_no_day_is_refused() {
        let gather = GatherRecordsInMemory::default();
        let burn = a_lab_with_morning_rust();

        assert_eq!(
            log_circuit(&gather, &burn, &running("Evening grind", 1)),
            Err(Failure::Refused(Refusal::NotACircuit))
        );
        assert_eq!(
            log_circuit(&gather, &burn, &running("Morning rust", 0)),
            Err(Failure::Refused(Refusal::NotACount))
        );
        assert_eq!(
            log_circuit(
                &gather,
                &burn,
                &CircuitSessionForm {
                    date: "2026-02-30".into(),
                    ..running("Morning rust", 1)
                }
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            log_circuit(&gather, &BrokenBurnRecords, &running("Morning rust", 1)),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn removing_a_circuit_leaves_the_workouts_already_logged_with_it() {
        let gather = GatherRecordsInMemory::default();
        let burn = a_lab_with_morning_rust();
        log_circuit(&gather, &burn, &running("Morning rust", 2)).expect("a circuit logs");

        remove_circuit(&burn, "Morning rust").expect("a circuit is removed");

        assert_eq!(
            burn_day(&gather, &burn, "2026-10-10").map(|burnt| burnt.workouts.len()),
            Ok(2)
        );
    }

    #[test]
    fn a_workout_logged_with_nothing_but_its_exercise_is_one_set_of_one_rep_for_a_minute_without_load()
     {
        let burnt = log_workout(
            &GatherRecordsInMemory::default(),
            &a_catalog_with_walking(),
            &WorkoutForm {
                date: "2026-10-10".into(),
                exercise: "Walking".into(),
                minutes: None,
                reps: None,
                sets: None,
                load_kg: None,
            },
        )
        .expect("a burn");

        assert_eq!(
            burnt
                .workouts
                .iter()
                .map(|done| (
                    done.workout.minutes,
                    done.workout.reps,
                    done.workout.sets,
                    done.workout.load_kg
                ))
                .collect::<Vec<_>>(),
            vec![(
                DEFAULT_MINUTES,
                Some(DEFAULT_REPS),
                DEFAULT_SETS,
                DEFAULT_LOAD_KG
            )]
        );
        assert_eq!(
            (DEFAULT_MINUTES, DEFAULT_REPS, DEFAULT_SETS, DEFAULT_LOAD_KG),
            (1.0, 1, 1, 0.0)
        );
    }

    #[test]
    fn a_circuit_logged_without_sets_runs_once() {
        let burnt = log_circuit(
            &GatherRecordsInMemory::default(),
            &a_lab_with_morning_rust(),
            &CircuitSessionForm {
                sets: None,
                ..running("Morning rust", 0)
            },
        )
        .expect("a burn");

        assert_eq!(
            burnt
                .workouts
                .iter()
                .map(|done| done.workout.sets)
                .collect::<Vec<_>>(),
            vec![1, 1]
        );
    }
}
