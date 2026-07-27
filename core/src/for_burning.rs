use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn::{Burn, burn_on};
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::refusal::Refusal;
use crate::volitional::volitional;
use crate::workout::workout;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkoutForm {
    pub date: String,
    pub exercise: String,
    pub minutes: f64,
    pub reps: Option<u32>,
    pub sets: u32,
    pub load_kg: f64,
}

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
        form.minutes,
        form.reps,
        form.sets,
        form.load_kg,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exercise::Exercise;
    use crate::for_tracking::log_measurement;
    use crate::testing::{BrokenBurnRecords, BurnRecordsInMemory, GatherRecordsInMemory};
    use crate::volitional::Volitional;

    fn a_catalog_with_walking() -> BurnRecordsInMemory {
        let burn = BurnRecordsInMemory::default();
        burn.save_exercise(&Exercise {
            name: "Walking".into(),
            met: 4.0,
        })
        .expect("an exercise saves");
        burn
    }

    fn walking(date: &str, minutes: f64) -> WorkoutForm {
        WorkoutForm {
            date: date.into(),
            exercise: "Walking".into(),
            minutes,
            reps: None,
            sets: 1,
            load_kg: 0.0,
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
}
