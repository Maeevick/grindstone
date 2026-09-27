use serde::{Deserialize, Serialize};

use crate::exercise::Exercise;
use crate::refusal::Refusal;
use crate::workout::activity_induced_energy_expenditure_from_compendium;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CircuitStep {
    pub exercise: String,
    pub met: f64,
    pub reps: Option<u32>,
    pub minutes: f64,
    pub load_kg: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    pub name: String,
    pub description: String,
    pub steps: Vec<CircuitStep>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepForm {
    pub exercise: String,
    pub reps: Option<u32>,
    pub minutes: f64,
    pub load_kg: f64,
}

fn step_frozen_on_its_exercise(
    asked: &StepForm,
    catalog: &[Exercise],
) -> Result<CircuitStep, Refusal> {
    let exercise = catalog
        .iter()
        .find(|known| known.name == asked.exercise)
        .ok_or(Refusal::NotAnExercise)?;
    if !(asked.minutes.is_finite() && asked.minutes > 0.0) {
        return Err(Refusal::NotADuration);
    }
    if !asked.load_kg.is_finite() {
        return Err(Refusal::NotALoad);
    }
    Ok(CircuitStep {
        exercise: exercise.name.clone(),
        met: exercise.met,
        reps: asked.reps.filter(|reps| *reps > 0),
        minutes: asked.minutes,
        load_kg: asked.load_kg,
    })
}

pub fn circuit(
    name: &str,
    description: &str,
    steps: &[StepForm],
    catalog: &[Exercise],
) -> Result<Circuit, Refusal> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Refusal::NotAName);
    }
    if steps.is_empty() {
        return Err(Refusal::NotASingleStep);
    }
    Ok(Circuit {
        name: name.into(),
        description: description.trim().into(),
        steps: steps
            .iter()
            .map(|asked| step_frozen_on_its_exercise(asked, catalog))
            .collect::<Result<_, _>>()?,
    })
}

pub fn burnt_per_set(circuit: &Circuit, body_mass_kg: f64) -> f64 {
    circuit
        .steps
        .iter()
        .map(|step| {
            activity_induced_energy_expenditure_from_compendium(
                body_mass_kg,
                step.met,
                step.minutes,
                step.load_kg,
            )
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    fn the_catalog() -> Vec<Exercise> {
        vec![
            Exercise {
                name: "Burpee".into(),
                met: 8.0,
            },
            Exercise {
                name: "Push up".into(),
                met: 3.8,
            },
        ]
    }

    fn asked(exercise: &str, reps: Option<u32>, minutes: f64, load_kg: f64) -> StepForm {
        StepForm {
            exercise: exercise.into(),
            reps,
            minutes,
            load_kg,
        }
    }

    fn push_ups_then_loaded_burpees() -> Vec<StepForm> {
        vec![
            asked("Push up", Some(12), 0.5, 0.0),
            asked("Burpee", Some(10), 1.0, 5.0),
        ]
    }

    #[test]
    fn a_circuit_keeps_its_trimmed_name_its_description_and_its_steps_in_order_each_frozen_on_its_exercise_met()
     {
        let crafted = circuit(
            "  Morning rust ",
            "  Found on YouTube, the rusty goblin channel.  ",
            &push_ups_then_loaded_burpees(),
            &the_catalog(),
        );

        assert_eq!(
            crafted,
            Ok(Circuit {
                name: "Morning rust".into(),
                description: "Found on YouTube, the rusty goblin channel.".into(),
                steps: vec![
                    CircuitStep {
                        exercise: "Push up".into(),
                        met: 3.8,
                        reps: Some(12),
                        minutes: 0.5,
                        load_kg: 0.0,
                    },
                    CircuitStep {
                        exercise: "Burpee".into(),
                        met: 8.0,
                        reps: Some(10),
                        minutes: 1.0,
                        load_kg: 5.0,
                    },
                ],
            })
        );
    }

    #[test]
    fn no_reps_goal_at_all_reads_as_a_step_counted_in_minutes() {
        let crafted = circuit(
            "Plank break",
            "",
            &[asked("Push up", Some(0), 1.0, 0.0)],
            &the_catalog(),
        );

        assert_eq!(
            crafted.map(|kept| kept.steps.into_iter().map(|step| step.reps).collect()),
            Ok(vec![None])
        );
    }

    #[test]
    fn a_circuit_without_a_name_or_without_a_single_step_is_refused() {
        assert_eq!(
            circuit("   ", "", &push_ups_then_loaded_burpees(), &the_catalog()),
            Err(Refusal::NotAName)
        );
        assert_eq!(
            circuit("Morning rust", "", &[], &the_catalog()),
            Err(Refusal::NotASingleStep)
        );
    }

    #[test]
    fn a_step_of_an_exercise_missing_from_the_catalog_is_refused() {
        assert_eq!(
            circuit(
                "Morning rust",
                "",
                &[asked("Star plank", None, 1.0, 0.0)],
                &the_catalog()
            ),
            Err(Refusal::NotAnExercise)
        );
    }

    #[test]
    fn a_step_of_no_minutes_or_of_a_load_that_is_no_number_is_refused() {
        for minutes in [0.0, -1.0, f64::NAN] {
            assert_eq!(
                circuit(
                    "Morning rust",
                    "",
                    &[asked("Push up", None, minutes, 0.0)],
                    &the_catalog()
                ),
                Err(Refusal::NotADuration),
                "{minutes}"
            );
        }
        assert_eq!(
            circuit(
                "Morning rust",
                "",
                &[asked("Push up", None, 1.0, f64::NAN)],
                &the_catalog()
            ),
            Err(Refusal::NotALoad)
        );
    }

    #[test]
    fn a_set_of_a_circuit_burns_the_sum_of_its_steps_above_rest_at_the_body_mass() {
        let crafted = circuit(
            "Morning rust",
            "",
            &push_ups_then_loaded_burpees(),
            &the_catalog(),
        )
        .expect("a circuit");

        assert!(about(
            burnt_per_set(&crafted, 80.0),
            activity_induced_energy_expenditure_from_compendium(80.0, 3.8, 0.5, 0.0)
                + activity_induced_energy_expenditure_from_compendium(80.0, 8.0, 1.0, 5.0)
        ));
    }
}
