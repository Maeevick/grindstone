use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::exercise::Exercise;
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Workout {
    pub day: NaiveDate,
    pub exercise: String,
    pub met: f64,
    pub minutes: f64,
    pub reps: Option<u32>,
    pub sets: u32,
    pub load_kg: f64,
    pub circuit: Option<String>,
}

pub fn workout(
    date: &str,
    exercise: &Exercise,
    minutes: f64,
    reps: Option<u32>,
    sets: u32,
    load_kg: f64,
) -> Result<Workout, Refusal> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Refusal::NotADate)?;
    if !(minutes.is_finite() && minutes > 0.0) {
        return Err(Refusal::NotADuration);
    }
    if sets == 0 {
        return Err(Refusal::NotACount);
    }
    if !load_kg.is_finite() {
        return Err(Refusal::NotALoad);
    }
    Ok(Workout {
        day,
        exercise: exercise.name.clone(),
        met: exercise.met,
        minutes,
        reps: reps.filter(|reps| *reps > 0),
        sets,
        load_kg,
        circuit: None,
    })
}

pub fn burnt_by(workout: &Workout, body_mass_kg: f64) -> f64 {
    f64::from(workout.sets)
        * activity_induced_energy_expenditure_from_compendium(
            body_mass_kg,
            workout.met,
            workout.minutes,
            workout.load_kg,
        )
}

/// What a workout burns above rest: its MET less one, at the body mass plus the load, for its minutes.
///
/// METs from Herrmann S. D. et al. (2024), *2024 Adult Compendium of Physical Activities: a third update
/// of the energy costs of human activities*, J Sport Health Sci 13(1):6-12.
pub fn activity_induced_energy_expenditure_from_compendium(
    bodymass: f64,
    met: f64,
    duration: f64,
    weight: f64,
) -> f64 {
    (met - 1.0).max(0.0) * 3.5 * (bodymass + weight).max(0.0) / 200.0 * duration.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn an_hour_at_two_mets_burns_one_met_above_rest_about_a_kcal_per_kilo() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(70.0, 2.0, 60.0, 0.0),
            73.5
        ));
    }

    #[test]
    fn an_hour_at_rest_burns_nothing_since_the_day_already_pays_for_it() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(70.0, 1.0, 60.0, 0.0),
            0.0
        ));
    }

    #[test]
    fn a_load_adds_its_kilos_to_the_working_share_and_a_band_takes_them_off() {
        let unloaded = activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, 0.0);
        let loaded = activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, 10.0);
        let helped = activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, -10.0);
        let ten_kilos_of_working_share = 6.5 * 3.5 * 10.0 / 200.0 * 10.0;

        assert!(about(loaded - unloaded, ten_kilos_of_working_share));
        assert!(about(unloaded - helped, ten_kilos_of_working_share));
    }

    #[test]
    fn a_band_helping_more_than_the_body_weighs_burns_nothing_rather_than_below() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, -200.0),
            0.0
        ));
    }

    #[test]
    fn an_activity_below_rest_burns_nothing_rather_than_below() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(75.0, 0.5, 30.0, 0.0),
            0.0
        ));
    }

    #[test]
    fn a_negative_duration_burns_nothing_rather_than_below() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(75.0, 7.5, -30.0, 0.0),
            0.0
        ));
    }

    fn push_up() -> Exercise {
        Exercise {
            name: "Push up".into(),
            met: 3.8,
            description: String::new(),
        }
    }

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    #[test]
    fn a_workout_keeps_its_day_its_exercise_and_how_it_went() {
        assert_eq!(
            workout("2026-10-10", &push_up(), 0.5, Some(12), 3, 0.0),
            Ok(Workout {
                day: day("2026-10-10"),
                exercise: "Push up".into(),
                met: 3.8,
                minutes: 0.5,
                reps: Some(12),
                sets: 3,
                load_kg: 0.0,
                circuit: None,
            })
        );
    }

    #[test]
    fn no_reps_at_all_reads_as_a_workout_counted_in_minutes() {
        assert_eq!(
            workout("2026-10-10", &push_up(), 20.0, Some(0), 1, 0.0).map(|done| done.reps),
            Ok(None)
        );
    }

    #[test]
    fn a_workout_that_could_not_have_happened_is_refused() {
        assert_eq!(
            workout("2026-02-30", &push_up(), 1.0, None, 1, 0.0),
            Err(Refusal::NotADate)
        );
        for minutes in [0.0, -1.0, f64::NAN] {
            assert_eq!(
                workout("2026-10-10", &push_up(), minutes, None, 1, 0.0),
                Err(Refusal::NotADuration),
                "{minutes}"
            );
        }
        assert_eq!(
            workout("2026-10-10", &push_up(), 1.0, None, 0, 0.0),
            Err(Refusal::NotACount)
        );
        assert_eq!(
            workout("2026-10-10", &push_up(), 1.0, None, 1, f64::NAN),
            Err(Refusal::NotALoad)
        );
    }

    #[test]
    fn each_set_burns_its_minutes_above_rest_at_the_body_mass_and_load_of_the_day() {
        let done = workout("2026-10-10", &push_up(), 0.5, Some(12), 3, 10.0).expect("a workout");

        assert!(about(
            burnt_by(&done, 70.0),
            3.0 * activity_induced_energy_expenditure_from_compendium(70.0, 3.8, 0.5, 10.0)
        ));
    }
}
