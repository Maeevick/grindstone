use serde::{Deserialize, Serialize};

use crate::burn::Burn;

/// An exercise over the week: how many times it was done, and its minutes, reps and burn in all.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExerciseWeek {
    pub exercise: String,
    pub times: usize,
    pub minutes: f64,
    pub reps: Option<f64>,
    pub burnt_kcal: Option<f64>,
}

impl ExerciseWeek {
    pub fn minutes_a_time(&self) -> f64 {
        0.0
    }

    pub fn reps_a_time(&self) -> Option<f64> {
        None
    }

    pub fn burnt_kcal_a_time(&self) -> Option<f64> {
        None
    }
}

/// What the week up to a day burnt, the day included: its days with a workout, its burn against the sum of
/// its days' targets, and each exercise summed up, the most burnt first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BurnWeek {
    pub active_days: usize,
    pub burnt_kcal: Option<f64>,
    pub target_kcal: Option<f64>,
    pub exercises: Vec<ExerciseWeek>,
}

impl BurnWeek {
    /// Whether the week's burn holds the sum of its days' targets, from 95% of it as the streak's tick does.
    pub fn on_target(&self) -> Option<bool> {
        None
    }
}

/// The week's burn, summed up from its days; nothing without a workout.
pub fn burn_week_of(days: &[Burn]) -> Option<BurnWeek> {
    let _ = days;
    Some(BurnWeek {
        active_days: 0,
        burnt_kcal: None,
        target_kcal: None,
        exercises: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::burn::BurntWorkout;
    use crate::workout::Workout;
    use chrono::NaiveDate;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn done(
        exercise: &str,
        (minutes, reps, sets): (f64, Option<u32>, u32),
        circuit: Option<&str>,
        burnt_kcal: f64,
    ) -> BurntWorkout {
        BurntWorkout {
            id: 1,
            workout: Workout {
                day: day("2026-10-10"),
                exercise: exercise.into(),
                met: 3.8,
                minutes,
                reps,
                sets,
                load_kg: 0.0,
                circuit: circuit.map(Into::into),
            },
            burnt_kcal: Some(burnt_kcal),
        }
    }

    fn a_day(date: &str, workouts: Vec<BurntWorkout>) -> Burn {
        Burn {
            day: day(date),
            burnt_kcal: workouts.iter().map(|done| done.burnt_kcal).sum(),
            workouts,
            off_the_clock: None,
            off_the_clock_is_usual: true,
            body_mass_kg: Some(70.0),
            target_kcal: Some(77.0),
        }
    }

    fn walking(minutes: f64, burnt_kcal: f64) -> BurntWorkout {
        done("Walking", (minutes, None, 1), None, burnt_kcal)
    }

    fn push_ups(circuit: Option<&str>) -> BurntWorkout {
        done("Push up", (0.5, Some(12), 3), circuit, 40.0)
    }

    fn a_week(burnt_kcal: Option<f64>, target_kcal: Option<f64>) -> BurnWeek {
        BurnWeek {
            active_days: 3,
            burnt_kcal,
            target_kcal,
            exercises: vec![],
        }
    }

    #[test]
    fn a_weeks_sum_up_totals_each_exercise_its_times_minutes_reps_and_burn_the_most_burnt_first() {
        let days = [
            a_day("2026-10-04", vec![walking(60.0, 252.0), push_ups(None)]),
            a_day("2026-10-06", vec![walking(30.0, 126.0)]),
            a_day("2026-10-08", vec![]),
            a_day("2026-10-10", vec![push_ups(None)]),
        ];

        assert_eq!(
            burn_week_of(&days),
            Some(BurnWeek {
                active_days: 3,
                burnt_kcal: Some(458.0),
                target_kcal: Some(308.0),
                exercises: vec![
                    ExerciseWeek {
                        exercise: "Walking".into(),
                        times: 2,
                        minutes: 90.0,
                        reps: None,
                        burnt_kcal: Some(378.0),
                    },
                    ExerciseWeek {
                        exercise: "Push up".into(),
                        times: 2,
                        minutes: 3.0,
                        reps: Some(72.0),
                        burnt_kcal: Some(80.0),
                    },
                ],
            })
        );
    }

    #[test]
    fn steps_run_in_a_circuit_count_with_their_exercise() {
        let days = [a_day(
            "2026-10-10",
            vec![push_ups(None), push_ups(Some("Morning rust"))],
        )];

        let week = burn_week_of(&days).expect("a week");

        assert_eq!(
            week.exercises
                .iter()
                .map(|summed| (summed.exercise.as_str(), summed.times))
                .collect::<Vec<_>>(),
            [("Push up", 2)]
        );
    }

    #[test]
    fn each_exercise_averages_its_minutes_reps_and_burn_a_time() {
        let push_ups = ExerciseWeek {
            exercise: "Push up".into(),
            times: 2,
            minutes: 3.0,
            reps: Some(72.0),
            burnt_kcal: Some(80.0),
        };
        let walking = ExerciseWeek {
            reps: None,
            ..push_ups.clone()
        };

        assert_eq!(
            (
                push_ups.minutes_a_time(),
                push_ups.reps_a_time(),
                push_ups.burnt_kcal_a_time(),
                walking.reps_a_time()
            ),
            (1.5, Some(36.0), Some(40.0), None)
        );
    }

    #[test]
    fn the_weeks_burn_holds_its_target_from_95_percent_of_the_sum_of_its_days_targets() {
        assert_eq!(
            [
                a_week(Some(95.0), Some(100.0)).on_target(),
                a_week(Some(94.0), Some(100.0)).on_target(),
                a_week(Some(95.0), None).on_target(),
                a_week(None, Some(100.0)).on_target()
            ],
            [Some(true), Some(false), None, None]
        );
    }

    #[test]
    fn a_week_without_a_workout_has_nothing_to_sum_up() {
        assert_eq!(
            [
                burn_week_of(&[a_day("2026-10-10", vec![])]),
                burn_week_of(&[])
            ],
            [None, None]
        );
    }
}
