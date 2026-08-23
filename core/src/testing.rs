use std::cell::{Cell, RefCell};

use crate::burn_records::BurnRecords;
use crate::circuit::{Circuit, CircuitStep};
use crate::drink::Drink;
use crate::drinking::Drinking;
use crate::exercise::Exercise;
use crate::failure::Broken;
use crate::food::{Food, Nutrients, Unit};
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::for_storing_switch_over::ForStoringSwitchOver;
use crate::fuel_records::{FuelRecords, FuelTargets};
use crate::gather_records::GatherRecords;
use crate::goals::Goals;
use crate::hydrate_records::HydrateRecords;
use crate::intake::Intake;
use crate::meal::Meal;
use crate::measurement::Measurement;
use crate::profile::Profile;
use crate::volitional::Volitional;
use crate::workout::Workout;
use chrono::NaiveDate;

#[derive(Default)]
pub struct GatherRecordsInMemory(RefCell<GatherRecords>);

impl ForStoringGatherRecords for GatherRecordsInMemory {
    fn save_measurement(&self, measurement: &Measurement) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records
            .measurements
            .retain(|kept| kept.day != measurement.day);
        records.measurements.push(*measurement);
        Ok(())
    }

    fn remove_measurement(&self, day: NaiveDate) -> Result<(), Broken> {
        self.0
            .borrow_mut()
            .measurements
            .retain(|kept| kept.day != day);
        Ok(())
    }

    fn save_profile(&self, from: NaiveDate, profile: &Profile) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.profiles.retain(|(kept_from, _)| *kept_from != from);
        records.profiles.push((from, *profile));
        Ok(())
    }

    fn save_goals(&self, from: NaiveDate, goals: &Goals) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.goals.retain(|(kept_from, _)| *kept_from != from);
        records.goals.push((from, *goals));
        Ok(())
    }

    fn replace_gather_records(&self, records: &GatherRecords) -> Result<(), Broken> {
        *self.0.borrow_mut() = records.clone();
        Ok(())
    }

    fn gather_records(&self) -> Result<GatherRecords, Broken> {
        let mut gathered = self.0.borrow().clone();
        gathered.measurements.sort_by_key(|measured| measured.day);
        gathered.profiles.sort_by_key(|(from, _)| *from);
        gathered.goals.sort_by_key(|(from, _)| *from);
        Ok(gathered)
    }
}

pub struct BrokenGatherRecords;

impl ForStoringGatherRecords for BrokenGatherRecords {
    fn save_measurement(&self, _: &Measurement) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_measurement(&self, _: NaiveDate) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_profile(&self, _: NaiveDate, _: &Profile) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_goals(&self, _: NaiveDate, _: &Goals) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn replace_gather_records(&self, _: &GatherRecords) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn gather_records(&self) -> Result<GatherRecords, Broken> {
        Err(Broken("the disk is full".into()))
    }
}

#[derive(Default)]
pub struct BurnRecordsInMemory(RefCell<BurnRecords>);

fn keeping_one_per_day(
    kept: &mut Vec<(NaiveDate, Volitional)>,
    day: NaiveDate,
    volitional: &Volitional,
) {
    kept.retain(|(kept_on, _)| *kept_on != day);
    kept.push((day, *volitional));
}

impl ForStoringBurnRecords for BurnRecordsInMemory {
    fn save_exercise(&self, exercise: &Exercise) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.exercises.retain(|kept| kept.name != exercise.name);
        records.exercises.push(exercise.clone());
        Ok(())
    }

    fn remove_exercise(&self, name: &str) -> Result<(), Broken> {
        self.0
            .borrow_mut()
            .exercises
            .retain(|kept| kept.name != name);
        Ok(())
    }

    fn save_workout(&self, workout: &Workout) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        let id = records
            .workouts
            .iter()
            .map(|(id, _)| *id)
            .max()
            .unwrap_or(0)
            + 1;
        records.workouts.push((id, workout.clone()));
        Ok(())
    }

    fn remove_workout(&self, id: i64) -> Result<(), Broken> {
        self.0.borrow_mut().workouts.retain(|(kept, _)| *kept != id);
        Ok(())
    }

    fn save_day_off_the_clock(
        &self,
        day: NaiveDate,
        volitional: &Volitional,
    ) -> Result<(), Broken> {
        keeping_one_per_day(&mut self.0.borrow_mut().days_off_the_clock, day, volitional);
        Ok(())
    }

    fn save_usual_day(&self, from: NaiveDate, volitional: &Volitional) -> Result<(), Broken> {
        keeping_one_per_day(&mut self.0.borrow_mut().usual_days, from, volitional);
        Ok(())
    }

    fn replace_burn_records(&self, records: &BurnRecords) -> Result<(), Broken> {
        let mut replaced = records.clone();
        for (id, (kept_id, _)) in (1..).zip(replaced.workouts.iter_mut()) {
            *kept_id = id;
        }
        *self.0.borrow_mut() = replaced;
        Ok(())
    }

    fn burn_records(&self) -> Result<BurnRecords, Broken> {
        let mut burnt = self.0.borrow().clone();
        burnt
            .exercises
            .sort_by(|one, other| one.name.cmp(&other.name));
        burnt.workouts.sort_by_key(|(id, done)| (done.day, *id));
        burnt.days_off_the_clock.sort_by_key(|(day, _)| *day);
        burnt.usual_days.sort_by_key(|(from, _)| *from);
        burnt
            .circuits
            .sort_by(|one, other| one.name.cmp(&other.name));
        Ok(burnt)
    }

    fn save_circuit(&self, circuit: &Circuit) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.circuits.retain(|kept| kept.name != circuit.name);
        records.circuits.push(circuit.clone());
        Ok(())
    }

    fn remove_circuit(&self, name: &str) -> Result<(), Broken> {
        self.0
            .borrow_mut()
            .circuits
            .retain(|kept| kept.name != name);
        Ok(())
    }
}

pub struct BrokenBurnRecords;

impl ForStoringBurnRecords for BrokenBurnRecords {
    fn save_exercise(&self, _: &Exercise) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_exercise(&self, _: &str) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_workout(&self, _: &Workout) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_workout(&self, _: i64) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_day_off_the_clock(&self, _: NaiveDate, _: &Volitional) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_usual_day(&self, _: NaiveDate, _: &Volitional) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn replace_burn_records(&self, _: &BurnRecords) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn burn_records(&self) -> Result<BurnRecords, Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_circuit(&self, _: &Circuit) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_circuit(&self, _: &str) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }
}

pub mod honouring_for_storing_burn_records {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn named(name: &str, met: f64) -> Exercise {
        Exercise {
            name: name.into(),
            met,
        }
    }

    fn done(date: &str, exercise: &str, minutes: f64) -> Workout {
        Workout {
            day: day(date),
            exercise: exercise.into(),
            met: 4.0,
            minutes,
            reps: Some(10),
            sets: 1,
            load_kg: 0.0,
            circuit: None,
        }
    }

    fn sitting(sitting_min: f64) -> Volitional {
        Volitional {
            sitting_min,
            standing_min: 120.0,
            moving_km: 3.0,
        }
    }

    fn without_ids(records: &BurnRecords) -> Vec<Workout> {
        records
            .workouts
            .iter()
            .map(|(_, done)| done.clone())
            .collect()
    }

    pub fn burn_records_come_back_in_the_order_of_their_days(records: &impl ForStoringBurnRecords) {
        records
            .save_exercise(&named("Walking", 4.0))
            .expect("an exercise saves");
        records
            .save_exercise(&named("Biking", 8.0))
            .expect("an exercise saves");
        for workout in [
            done("2026-10-10", "Walking", 30.0),
            done("2026-10-09", "Biking", 45.0),
            done("2026-10-10", "Biking", 20.0),
        ] {
            records.save_workout(&workout).expect("a workout saves");
        }
        records
            .save_day_off_the_clock(day("2026-10-10"), &sitting(600.0))
            .expect("a day off the clock saves");
        records
            .save_day_off_the_clock(day("2026-10-09"), &sitting(500.0))
            .expect("a day off the clock saves");
        records
            .save_usual_day(day("2026-09-01"), &sitting(420.0))
            .expect("a usual day saves");
        records
            .save_usual_day(day("2026-03-01"), &sitting(480.0))
            .expect("a usual day saves");

        let burnt = records.burn_records().expect("the records read");

        assert_eq!(
            burnt.exercises,
            vec![named("Biking", 8.0), named("Walking", 4.0)]
        );
        assert_eq!(
            without_ids(&burnt),
            vec![
                done("2026-10-09", "Biking", 45.0),
                done("2026-10-10", "Walking", 30.0),
                done("2026-10-10", "Biking", 20.0),
            ]
        );
        assert_eq!(
            burnt.days_off_the_clock,
            vec![
                (day("2026-10-09"), sitting(500.0)),
                (day("2026-10-10"), sitting(600.0))
            ]
        );
        assert_eq!(
            burnt.usual_days,
            vec![
                (day("2026-03-01"), sitting(480.0)),
                (day("2026-09-01"), sitting(420.0))
            ]
        );
    }

    pub fn what_is_saved_again_under_its_name_or_day_replaces_the_earlier_one(
        records: &impl ForStoringBurnRecords,
    ) {
        records
            .save_exercise(&named("Walking", 3.5))
            .expect("an exercise saves");
        records
            .save_day_off_the_clock(day("2026-10-10"), &sitting(600.0))
            .expect("a day off the clock saves");
        records
            .save_usual_day(day("2026-03-01"), &sitting(480.0))
            .expect("a usual day saves");

        records
            .save_exercise(&named("Walking", 4.0))
            .expect("an exercise saves");
        records
            .save_day_off_the_clock(day("2026-10-10"), &sitting(300.0))
            .expect("a day off the clock saves");
        records
            .save_usual_day(day("2026-03-01"), &sitting(420.0))
            .expect("a usual day saves");

        let burnt = records.burn_records().expect("the records read");
        assert_eq!(burnt.exercises, vec![named("Walking", 4.0)]);
        assert_eq!(
            burnt.days_off_the_clock,
            vec![(day("2026-10-10"), sitting(300.0))]
        );
        assert_eq!(burnt.usual_days, vec![(day("2026-03-01"), sitting(420.0))]);
    }

    pub fn removing_a_workout_or_an_exercise_leaves_the_others(
        records: &impl ForStoringBurnRecords,
    ) {
        records
            .save_exercise(&named("Walking", 4.0))
            .expect("an exercise saves");
        records
            .save_exercise(&named("Biking", 8.0))
            .expect("an exercise saves");
        records
            .save_workout(&done("2026-10-10", "Walking", 30.0))
            .expect("a workout saves");
        records
            .save_workout(&done("2026-10-10", "Walking", 15.0))
            .expect("a workout saves");
        let first = records
            .burn_records()
            .ok()
            .and_then(|burnt| burnt.workouts.first().map(|(id, _)| *id))
            .expect("a first workout");

        records.remove_workout(first).expect("a workout is removed");
        records
            .remove_exercise("Biking")
            .expect("an exercise is removed");

        let burnt = records.burn_records().expect("the records read");
        assert_eq!(
            without_ids(&burnt),
            vec![done("2026-10-10", "Walking", 15.0)]
        );
        assert_eq!(burnt.exercises, vec![named("Walking", 4.0)]);
    }

    pub fn replacing_the_burn_records_leaves_only_the_new_ones(
        records: &impl ForStoringBurnRecords,
    ) {
        records
            .save_exercise(&named("Walking", 4.0))
            .expect("an exercise saves");
        records
            .save_workout(&done("2026-10-10", "Walking", 30.0))
            .expect("a workout saves");
        records
            .save_circuit(&evening_grind())
            .expect("a circuit saves");
        let replacement = BurnRecords {
            exercises: vec![named("Biking", 8.0)],
            workouts: vec![(0, done("2026-06-22", "Biking", 45.0))],
            days_off_the_clock: vec![(day("2026-06-22"), sitting(500.0))],
            usual_days: vec![(day("2026-03-01"), sitting(480.0))],
            circuits: vec![morning_rust()],
        };

        records
            .replace_burn_records(&replacement)
            .expect("the records are replaced");

        let burnt = records.burn_records().expect("the records read");
        assert_eq!(burnt.exercises, replacement.exercises);
        assert_eq!(without_ids(&burnt), without_ids(&replacement));
        assert_eq!(burnt.days_off_the_clock, replacement.days_off_the_clock);
        assert_eq!(burnt.usual_days, replacement.usual_days);
        assert_eq!(burnt.circuits, replacement.circuits);
    }

    fn step(
        exercise: &str,
        met: f64,
        reps: Option<u32>,
        minutes: f64,
        load_kg: f64,
    ) -> CircuitStep {
        CircuitStep {
            exercise: exercise.into(),
            met,
            reps,
            minutes,
            load_kg,
        }
    }

    fn morning_rust() -> Circuit {
        Circuit {
            name: "Morning rust".into(),
            steps: vec![
                step("Push up", 3.8, Some(12), 0.5, 0.0),
                step("Burpee", 8.0, None, 1.0, 5.0),
            ],
        }
    }

    fn evening_grind() -> Circuit {
        Circuit {
            name: "Evening grind".into(),
            steps: vec![step("Squat", 5.0, Some(15), 1.0, 20.0)],
        }
    }

    pub fn crafted_circuits_come_back_by_name_with_their_steps_in_order(
        records: &impl ForStoringBurnRecords,
    ) {
        records
            .save_circuit(&morning_rust())
            .expect("a circuit saves");
        records
            .save_circuit(&evening_grind())
            .expect("a circuit saves");

        let burnt = records.burn_records().expect("the records read");

        assert_eq!(burnt.circuits, vec![evening_grind(), morning_rust()]);
    }

    pub fn removing_a_circuit_leaves_the_others(records: &impl ForStoringBurnRecords) {
        records
            .save_circuit(&morning_rust())
            .expect("a circuit saves");
        records
            .save_circuit(&evening_grind())
            .expect("a circuit saves");

        records
            .remove_circuit("Morning rust")
            .expect("a circuit is removed");

        assert_eq!(
            records.burn_records().map(|burnt| burnt.circuits),
            Ok(vec![evening_grind()])
        );
    }

    pub fn a_circuit_crafted_again_under_its_name_replaces_the_earlier_one(
        records: &impl ForStoringBurnRecords,
    ) {
        let crafted_again = Circuit {
            steps: vec![step("Squat", 5.0, Some(20), 1.5, 0.0)],
            ..morning_rust()
        };
        records
            .save_circuit(&morning_rust())
            .expect("a circuit saves");

        records
            .save_circuit(&crafted_again)
            .expect("a circuit saves");

        assert_eq!(
            records.burn_records().map(|burnt| burnt.circuits),
            Ok(vec![crafted_again])
        );
    }

    pub fn a_workout_done_in_a_circuit_comes_back_with_its_name(
        records: &impl ForStoringBurnRecords,
    ) {
        let in_a_circuit = Workout {
            circuit: Some("Morning rust".into()),
            ..done("2026-10-10", "Push up", 0.5)
        };

        records
            .save_workout(&in_a_circuit)
            .expect("a workout saves");

        assert_eq!(
            records.burn_records().map(|burnt| without_ids(&burnt)),
            Ok(vec![in_a_circuit])
        );
    }
}

#[derive(Default)]
pub struct FuelRecordsInMemory(RefCell<FuelRecords>);

impl ForStoringFuelRecords for FuelRecordsInMemory {
    fn save_food(&self, food: &Food) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.foods.retain(|kept| kept.name != food.name);
        records.foods.push(food.clone());
        Ok(())
    }

    fn remove_food(&self, name: &str) -> Result<(), Broken> {
        self.0.borrow_mut().foods.retain(|kept| kept.name != name);
        Ok(())
    }

    fn save_intake(&self, intake: &Intake) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        let id = records.intakes.iter().map(|(id, _)| *id).max().unwrap_or(0) + 1;
        records.intakes.push((id, intake.clone()));
        Ok(())
    }

    fn remove_intake(&self, id: i64) -> Result<(), Broken> {
        self.0.borrow_mut().intakes.retain(|(kept, _)| *kept != id);
        Ok(())
    }

    fn save_targets(&self, from: NaiveDate, targets: &FuelTargets) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.targets.retain(|(kept_from, _)| *kept_from != from);
        records.targets.push((from, *targets));
        Ok(())
    }

    fn save_meal(&self, meal: &Meal) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.meals.retain(|kept| kept.name != meal.name);
        records.meals.push(meal.clone());
        Ok(())
    }

    fn remove_meal(&self, name: &str) -> Result<(), Broken> {
        self.0.borrow_mut().meals.retain(|kept| kept.name != name);
        Ok(())
    }

    fn replace_fuel_records(&self, records: &FuelRecords) -> Result<(), Broken> {
        let mut replaced = records.clone();
        for (id, (kept_id, _)) in (1..).zip(replaced.intakes.iter_mut()) {
            *kept_id = id;
        }
        *self.0.borrow_mut() = replaced;
        Ok(())
    }

    fn fuel_records(&self) -> Result<FuelRecords, Broken> {
        let mut fuelled = self.0.borrow().clone();
        fuelled
            .foods
            .sort_by(|one, other| one.name.cmp(&other.name));
        fuelled.intakes.sort_by_key(|(id, eaten)| (eaten.day, *id));
        fuelled.targets.sort_by_key(|(from, _)| *from);
        fuelled
            .meals
            .sort_by(|one, other| one.name.cmp(&other.name));
        Ok(fuelled)
    }
}

pub struct BrokenFuelRecords;

impl ForStoringFuelRecords for BrokenFuelRecords {
    fn save_food(&self, _: &Food) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_food(&self, _: &str) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_intake(&self, _: &Intake) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_intake(&self, _: i64) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_targets(&self, _: NaiveDate, _: &FuelTargets) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_meal(&self, _: &Meal) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_meal(&self, _: &str) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn replace_fuel_records(&self, _: &FuelRecords) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn fuel_records(&self) -> Result<FuelRecords, Broken> {
        Err(Broken("the disk is full".into()))
    }
}

pub mod honouring_for_storing_fuel_records {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn a_food(name: &str, energy_kcal: f64) -> Food {
        Food {
            name: name.into(),
            unit: Unit::Gram,
            portion: 100.0,
            nutrients: Nutrients {
                energy_kcal,
                protein_g: 9.5,
                carbs_g: 4.0,
                fat_g: 0.2,
                fibre_g: 0.0,
            },
        }
    }

    fn eaten(date: &str, food: &str, quantity: f64) -> Intake {
        Intake {
            day: day(date),
            food: food.into(),
            unit: Unit::Piece,
            quantity,
            nutrients: Nutrients {
                energy_kcal: 54.0 * quantity,
                ..Nutrients::default()
            },
        }
    }

    fn aiming(protein_g_per_kg: f64) -> FuelTargets {
        FuelTargets {
            protein_g_per_kg,
            fat_g_per_kg: 0.9,
            fibre_g_per_kg: 0.4,
        }
    }

    fn without_ids(records: &FuelRecords) -> Vec<Intake> {
        records
            .intakes
            .iter()
            .map(|(_, eaten)| eaten.clone())
            .collect()
    }

    pub fn fuel_records_come_back_in_the_order_of_their_days(records: &impl ForStoringFuelRecords) {
        records
            .save_food(&a_food("Skyr", 54.0))
            .expect("a food saves");
        records
            .save_food(&a_food("Muesli", 365.0))
            .expect("a food saves");
        for intake in [
            eaten("2026-10-10", "Skyr", 2.0),
            eaten("2026-10-09", "Muesli", 1.0),
            eaten("2026-10-10", "Muesli", 0.5),
        ] {
            records.save_intake(&intake).expect("an intake saves");
        }
        records
            .save_targets(day("2026-09-01"), &aiming(2.0))
            .expect("targets save");
        records
            .save_targets(day("2026-03-01"), &aiming(1.8))
            .expect("targets save");

        let fuelled = records.fuel_records().expect("the records read");

        assert_eq!(
            fuelled.foods,
            vec![a_food("Muesli", 365.0), a_food("Skyr", 54.0)]
        );
        assert_eq!(
            without_ids(&fuelled),
            vec![
                eaten("2026-10-09", "Muesli", 1.0),
                eaten("2026-10-10", "Skyr", 2.0),
                eaten("2026-10-10", "Muesli", 0.5),
            ]
        );
        assert_eq!(
            fuelled.targets,
            vec![
                (day("2026-03-01"), aiming(1.8)),
                (day("2026-09-01"), aiming(2.0))
            ]
        );
    }

    pub fn a_food_or_targets_saved_again_replace_the_earlier_ones(
        records: &impl ForStoringFuelRecords,
    ) {
        records
            .save_food(&a_food("Skyr", 50.0))
            .expect("a food saves");
        records
            .save_targets(day("2026-03-01"), &aiming(1.8))
            .expect("targets save");

        records
            .save_food(&a_food("Skyr", 54.0))
            .expect("a food saves");
        records
            .save_targets(day("2026-03-01"), &aiming(2.0))
            .expect("targets save");

        let fuelled = records.fuel_records().expect("the records read");
        assert_eq!(fuelled.foods, vec![a_food("Skyr", 54.0)]);
        assert_eq!(fuelled.targets, vec![(day("2026-03-01"), aiming(2.0))]);
    }

    pub fn removing_an_intake_or_a_food_leaves_the_others(records: &impl ForStoringFuelRecords) {
        records
            .save_food(&a_food("Skyr", 54.0))
            .expect("a food saves");
        records
            .save_food(&a_food("Muesli", 365.0))
            .expect("a food saves");
        records
            .save_intake(&eaten("2026-10-10", "Skyr", 2.0))
            .expect("an intake saves");
        records
            .save_intake(&eaten("2026-10-10", "Skyr", 1.0))
            .expect("an intake saves");
        let first = records
            .fuel_records()
            .ok()
            .and_then(|fuelled| fuelled.intakes.first().map(|(id, _)| *id))
            .expect("a first intake");

        records.remove_intake(first).expect("an intake is removed");
        records.remove_food("Muesli").expect("a food is removed");

        let fuelled = records.fuel_records().expect("the records read");
        assert_eq!(
            without_ids(&fuelled),
            vec![eaten("2026-10-10", "Skyr", 1.0)]
        );
        assert_eq!(fuelled.foods, vec![a_food("Skyr", 54.0)]);
    }

    fn cooked(name: &str, cooked_g: f64) -> Meal {
        Meal {
            name: name.into(),
            recipe: "Stir.".into(),
            ingredients: vec![crate::meal::Ingredient {
                food: "Rice".into(),
                unit: Unit::Gram,
                quantity: 200.0,
                nutrients: Nutrients {
                    energy_kcal: 700.0,
                    ..Nutrients::default()
                },
            }],
            raw_g: 200.0,
            cooked_g,
            totals: Nutrients {
                energy_kcal: 700.0,
                ..Nutrients::default()
            },
        }
    }

    pub fn meals_come_back_by_name_a_meal_cooked_again_replaces_the_earlier_and_a_removed_one_leaves(
        records: &impl ForStoringFuelRecords,
    ) {
        records
            .save_meal(&cooked("Risotto", 500.0))
            .expect("a meal saves");
        records
            .save_meal(&cooked("Dal", 600.0))
            .expect("a meal saves");
        records
            .save_meal(&cooked("Bowl", 400.0))
            .expect("a meal saves");
        records
            .save_meal(&cooked("Risotto", 550.0))
            .expect("a meal saves");

        records.remove_meal("Bowl").expect("a meal is removed");

        assert_eq!(
            records.fuel_records().map(|fuelled| fuelled.meals),
            Ok(vec![cooked("Dal", 600.0), cooked("Risotto", 550.0)])
        );
    }

    pub fn replacing_the_fuel_records_leaves_only_the_new_ones(
        records: &impl ForStoringFuelRecords,
    ) {
        records
            .save_food(&a_food("Skyr", 54.0))
            .expect("a food saves");
        records
            .save_intake(&eaten("2026-10-10", "Skyr", 2.0))
            .expect("an intake saves");
        let replacement = FuelRecords {
            foods: vec![a_food("Muesli", 365.0)],
            intakes: vec![(0, eaten("2026-08-02", "Muesli", 1.0))],
            targets: vec![(day("2026-03-01"), aiming(2.025))],
            meals: vec![cooked("Dal", 600.0)],
        };

        records
            .replace_fuel_records(&replacement)
            .expect("the records are replaced");

        let fuelled = records.fuel_records().expect("the records read");
        assert_eq!(fuelled.foods, replacement.foods);
        assert_eq!(without_ids(&fuelled), without_ids(&replacement));
        assert_eq!(fuelled.targets, replacement.targets);
        assert_eq!(fuelled.meals, replacement.meals);
    }
}

#[derive(Default)]
pub struct SwitchOverInMemory(Cell<Option<NaiveDate>>);

impl ForStoringSwitchOver for SwitchOverInMemory {
    fn save_switch_over(&self, day: NaiveDate) -> Result<(), Broken> {
        self.0.set(Some(day));
        Ok(())
    }

    fn switch_over(&self) -> Result<Option<NaiveDate>, Broken> {
        Ok(self.0.get())
    }
}

pub struct BrokenSwitchOver;

impl ForStoringSwitchOver for BrokenSwitchOver {
    fn save_switch_over(&self, _: NaiveDate) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn switch_over(&self) -> Result<Option<NaiveDate>, Broken> {
        Err(Broken("the disk is full".into()))
    }
}

pub mod honouring_for_storing_switch_over {
    use super::*;

    pub fn no_switch_over_until_one_is_saved_and_the_latest_one_stands(
        records: &impl ForStoringSwitchOver,
    ) {
        let day = |date| NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day");
        assert_eq!(records.switch_over(), Ok(None));

        records
            .save_switch_over(day("2026-10-10"))
            .expect("a switch-over saves");
        records
            .save_switch_over(day("2026-10-11"))
            .expect("a switch-over saves");

        assert_eq!(records.switch_over(), Ok(Some(day("2026-10-11"))));
    }
}

pub mod honouring_for_storing_gather_records {
    use super::*;
    use crate::gender::Gender;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn measured(date: &str, body_mass_kg: f64, waist_cm: Option<f64>) -> Measurement {
        Measurement {
            day: day(date),
            body_mass_kg,
            waist_cm,
        }
    }

    fn a_man(neck_cm: f64) -> Profile {
        Profile {
            birthday: day("1985-04-12"),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm,
            hip_cm: None,
        }
    }

    fn targeting(body_mass_kg: f64) -> Goals {
        Goals {
            body_mass_kg,
            fat_ratio: 0.10,
        }
    }

    pub fn gather_records_come_back_in_the_order_of_their_days(
        records: &impl ForStoringGatherRecords,
    ) {
        for measurement in [
            measured("2026-10-10", 80.0, None),
            measured("2026-10-08", 80.4, Some(84.0)),
            measured("2026-10-09", 80.2, None),
        ] {
            records
                .save_measurement(&measurement)
                .expect("a measurement saves");
        }
        records
            .save_profile(day("2026-09-01"), &a_man(38.0))
            .expect("a profile saves");
        records
            .save_profile(day("2026-03-01"), &a_man(38.5))
            .expect("a profile saves");
        records
            .save_goals(day("2026-09-01"), &targeting(70.0))
            .expect("goals save");
        records
            .save_goals(day("2026-03-01"), &targeting(72.0))
            .expect("goals save");

        assert_eq!(
            records.gather_records(),
            Ok(GatherRecords {
                measurements: vec![
                    measured("2026-10-08", 80.4, Some(84.0)),
                    measured("2026-10-09", 80.2, None),
                    measured("2026-10-10", 80.0, None),
                ],
                profiles: vec![
                    (day("2026-03-01"), a_man(38.5)),
                    (day("2026-09-01"), a_man(38.0))
                ],
                goals: vec![
                    (day("2026-03-01"), targeting(72.0)),
                    (day("2026-09-01"), targeting(70.0))
                ],
            })
        );
    }

    pub fn a_day_measured_twice_keeps_its_second_measurement(
        records: &impl ForStoringGatherRecords,
    ) {
        records
            .save_measurement(&measured("2026-10-10", 80.4, None))
            .expect("a measurement saves");

        records
            .save_measurement(&measured("2026-10-10", 80.0, Some(84.0)))
            .expect("a measurement saves");

        assert_eq!(
            records
                .gather_records()
                .map(|gathered| gathered.measurements),
            Ok(vec![measured("2026-10-10", 80.0, Some(84.0))])
        );
    }

    pub fn removing_a_measured_day_leaves_the_others(records: &impl ForStoringGatherRecords) {
        records
            .save_measurement(&measured("2026-10-09", 80.2, None))
            .expect("a measurement saves");
        records
            .save_measurement(&measured("2026-10-10", 80.0, None))
            .expect("a measurement saves");

        records
            .remove_measurement(day("2026-10-10"))
            .expect("a day is removed");

        assert_eq!(
            records
                .gather_records()
                .map(|gathered| gathered.measurements),
            Ok(vec![measured("2026-10-09", 80.2, None)])
        );
    }

    pub fn a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones(
        records: &impl ForStoringGatherRecords,
    ) {
        records
            .save_profile(day("2026-03-01"), &a_man(38.5))
            .expect("a profile saves");
        records
            .save_goals(day("2026-03-01"), &targeting(72.0))
            .expect("goals save");

        records
            .save_profile(day("2026-03-01"), &a_man(38.0))
            .expect("a profile saves");
        records
            .save_goals(day("2026-03-01"), &targeting(70.0))
            .expect("goals save");

        let gathered = records.gather_records().expect("the records read");
        assert_eq!(gathered.profiles, vec![(day("2026-03-01"), a_man(38.0))]);
        assert_eq!(gathered.goals, vec![(day("2026-03-01"), targeting(70.0))]);
    }

    pub fn replacing_the_gather_records_leaves_only_the_new_ones(
        records: &impl ForStoringGatherRecords,
    ) {
        records
            .save_measurement(&measured("2026-10-10", 80.0, None))
            .expect("a measurement saves");
        records
            .save_profile(day("2026-03-01"), &a_man(38.5))
            .expect("a profile saves");
        let replacement = GatherRecords {
            measurements: vec![measured("2026-06-22", 76.4, Some(84.0))],
            profiles: vec![(day("2026-06-01"), a_man(38.0))],
            goals: vec![(day("2026-06-01"), targeting(70.0))],
        };

        records
            .replace_gather_records(&replacement)
            .expect("the records are replaced");

        assert_eq!(records.gather_records(), Ok(replacement));
    }

    pub fn a_womans_profile_comes_back_with_her_hip(records: &impl ForStoringGatherRecords) {
        let a_woman = Profile {
            birthday: day("1990-06-01"),
            gender: Gender::Female,
            height_cm: 165.0,
            neck_cm: 33.0,
            hip_cm: Some(98.0),
        };

        records
            .save_profile(day("2026-03-01"), &a_woman)
            .expect("a profile saves");

        assert_eq!(
            records.gather_records().map(|gathered| gathered.profiles),
            Ok(vec![(day("2026-03-01"), a_woman)])
        );
    }
}

#[derive(Default)]
pub struct HydrateRecordsInMemory(RefCell<HydrateRecords>);

impl ForStoringHydrateRecords for HydrateRecordsInMemory {
    fn save_drink(&self, drink: &Drink) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.drinks.retain(|kept| kept.name != drink.name);
        records.drinks.push(drink.clone());
        Ok(())
    }

    fn remove_drink(&self, name: &str) -> Result<(), Broken> {
        self.0.borrow_mut().drinks.retain(|kept| kept.name != name);
        Ok(())
    }

    fn save_drinking(&self, drinking: &Drinking) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        let id = records
            .drinkings
            .iter()
            .map(|(id, _)| *id)
            .max()
            .unwrap_or(0)
            + 1;
        records.drinkings.push((id, drinking.clone()));
        Ok(())
    }

    fn remove_drinking(&self, id: i64) -> Result<(), Broken> {
        self.0
            .borrow_mut()
            .drinkings
            .retain(|(kept, _)| *kept != id);
        Ok(())
    }

    fn save_base(&self, from: NaiveDate, base_ml: f64) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.bases.retain(|(kept_from, _)| *kept_from != from);
        records.bases.push((from, base_ml));
        Ok(())
    }

    fn replace_hydrate_records(&self, records: &HydrateRecords) -> Result<(), Broken> {
        let mut replaced = records.clone();
        for (id, (kept_id, _)) in (1..).zip(replaced.drinkings.iter_mut()) {
            *kept_id = id;
        }
        *self.0.borrow_mut() = replaced;
        Ok(())
    }

    fn hydrate_records(&self) -> Result<HydrateRecords, Broken> {
        let mut drunk = self.0.borrow().clone();
        drunk.drinks.sort_by(|one, other| one.name.cmp(&other.name));
        drunk
            .drinkings
            .sort_by_key(|(id, drinking)| (drinking.day, *id));
        drunk.bases.sort_by_key(|(from, _)| *from);
        Ok(drunk)
    }
}

pub struct BrokenHydrateRecords;

impl ForStoringHydrateRecords for BrokenHydrateRecords {
    fn save_drink(&self, _: &Drink) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_drink(&self, _: &str) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_drinking(&self, _: &Drinking) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_drinking(&self, _: i64) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_base(&self, _: NaiveDate, _: f64) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn replace_hydrate_records(&self, _: &HydrateRecords) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn hydrate_records(&self) -> Result<HydrateRecords, Broken> {
        Err(Broken("the disk is full".into()))
    }
}

pub mod honouring_for_storing_hydrate_records {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn served(name: &str, volume_ml: f64) -> Drink {
        Drink {
            name: name.into(),
            volume_ml,
        }
    }

    fn drunk(date: &str, drink: &str, volume_ml: f64) -> Drinking {
        Drinking {
            day: day(date),
            drink: drink.into(),
            volume_ml,
        }
    }

    fn without_ids(records: &HydrateRecords) -> Vec<Drinking> {
        records
            .drinkings
            .iter()
            .map(|(_, drinking)| drinking.clone())
            .collect()
    }

    pub fn hydrate_records_come_back_in_the_order_of_their_days(
        records: &impl ForStoringHydrateRecords,
    ) {
        records
            .save_drink(&served("Tea", 200.0))
            .expect("a drink saves");
        records
            .save_drink(&served("Expresso", 80.0))
            .expect("a drink saves");
        for drinking in [
            drunk("2026-10-10", "Tea", 200.0),
            drunk("2026-10-09", "Expresso", 80.0),
            drunk("2026-10-10", "Expresso", 160.0),
        ] {
            records.save_drinking(&drinking).expect("a drinking saves");
        }
        records
            .save_base(day("2026-09-01"), 2200.0)
            .expect("a base saves");
        records
            .save_base(day("2026-03-01"), 2000.0)
            .expect("a base saves");

        let drunk_so_far = records.hydrate_records().expect("the records read");

        assert_eq!(
            drunk_so_far.drinks,
            vec![served("Expresso", 80.0), served("Tea", 200.0)]
        );
        assert_eq!(
            without_ids(&drunk_so_far),
            vec![
                drunk("2026-10-09", "Expresso", 80.0),
                drunk("2026-10-10", "Tea", 200.0),
                drunk("2026-10-10", "Expresso", 160.0),
            ]
        );
        assert_eq!(
            drunk_so_far.bases,
            vec![(day("2026-03-01"), 2000.0), (day("2026-09-01"), 2200.0)]
        );
    }

    pub fn a_drink_or_a_base_saved_again_under_its_name_or_day_replaces_the_earlier_one(
        records: &impl ForStoringHydrateRecords,
    ) {
        records
            .save_drink(&served("Tea", 200.0))
            .expect("a drink saves");
        records
            .save_base(day("2026-03-01"), 2000.0)
            .expect("a base saves");

        records
            .save_drink(&served("Tea", 250.0))
            .expect("a drink saves");
        records
            .save_base(day("2026-03-01"), 2400.0)
            .expect("a base saves");

        let drunk_so_far = records.hydrate_records().expect("the records read");
        assert_eq!(drunk_so_far.drinks, vec![served("Tea", 250.0)]);
        assert_eq!(drunk_so_far.bases, vec![(day("2026-03-01"), 2400.0)]);
    }

    pub fn removing_a_drinking_or_a_drink_leaves_the_others(
        records: &impl ForStoringHydrateRecords,
    ) {
        records
            .save_drink(&served("Tea", 200.0))
            .expect("a drink saves");
        records
            .save_drink(&served("Expresso", 80.0))
            .expect("a drink saves");
        records
            .save_drinking(&drunk("2026-10-10", "Tea", 200.0))
            .expect("a drinking saves");
        records
            .save_drinking(&drunk("2026-10-10", "Tea", 100.0))
            .expect("a drinking saves");
        let first = records
            .hydrate_records()
            .ok()
            .and_then(|drunk_so_far| drunk_so_far.drinkings.first().map(|(id, _)| *id))
            .expect("a first drinking");

        records
            .remove_drinking(first)
            .expect("a drinking is removed");
        records
            .remove_drink("Expresso")
            .expect("a drink is removed");

        let drunk_so_far = records.hydrate_records().expect("the records read");
        assert_eq!(
            without_ids(&drunk_so_far),
            vec![drunk("2026-10-10", "Tea", 100.0)]
        );
        assert_eq!(drunk_so_far.drinks, vec![served("Tea", 200.0)]);
    }

    pub fn replacing_the_hydrate_records_leaves_only_the_new_ones(
        records: &impl ForStoringHydrateRecords,
    ) {
        records
            .save_drink(&served("Tea", 200.0))
            .expect("a drink saves");
        records
            .save_drinking(&drunk("2026-10-10", "Tea", 200.0))
            .expect("a drinking saves");
        let replacement = HydrateRecords {
            drinks: vec![served("Expresso", 80.0)],
            drinkings: vec![(0, drunk("2026-06-22", "Expresso", 80.0))],
            bases: vec![(day("2026-03-01"), 2000.0)],
        };

        records
            .replace_hydrate_records(&replacement)
            .expect("the records are replaced");

        let drunk_so_far = records.hydrate_records().expect("the records read");
        assert_eq!(drunk_so_far.drinks, replacement.drinks);
        assert_eq!(without_ids(&drunk_so_far), without_ids(&replacement));
        assert_eq!(drunk_so_far.bases, replacement.bases);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_gather_records::gather_records_come_back_in_the_order_of_their_days(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_day_measured_twice_keeps_its_second_measurement() {
        honouring_for_storing_gather_records::a_day_measured_twice_keeps_its_second_measurement(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn removing_a_measured_day_leaves_the_others() {
        honouring_for_storing_gather_records::removing_a_measured_day_leaves_the_others(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones() {
        honouring_for_storing_gather_records::a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn replacing_the_gather_records_leaves_only_the_new_ones() {
        honouring_for_storing_gather_records::replacing_the_gather_records_leaves_only_the_new_ones(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_womans_profile_comes_back_with_her_hip() {
        honouring_for_storing_gather_records::a_womans_profile_comes_back_with_her_hip(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn burn_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_burn_records::burn_records_come_back_in_the_order_of_their_days(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn what_is_saved_again_under_its_name_or_day_replaces_the_earlier_one() {
        honouring_for_storing_burn_records::what_is_saved_again_under_its_name_or_day_replaces_the_earlier_one(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn removing_a_workout_or_an_exercise_leaves_the_others() {
        honouring_for_storing_burn_records::removing_a_workout_or_an_exercise_leaves_the_others(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn replacing_the_burn_records_leaves_only_the_new_ones() {
        honouring_for_storing_burn_records::replacing_the_burn_records_leaves_only_the_new_ones(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn fuel_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_fuel_records::fuel_records_come_back_in_the_order_of_their_days(
            &FuelRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_food_or_targets_saved_again_replace_the_earlier_ones() {
        honouring_for_storing_fuel_records::a_food_or_targets_saved_again_replace_the_earlier_ones(
            &FuelRecordsInMemory::default(),
        );
    }

    #[test]
    fn removing_an_intake_or_a_food_leaves_the_others() {
        honouring_for_storing_fuel_records::removing_an_intake_or_a_food_leaves_the_others(
            &FuelRecordsInMemory::default(),
        );
    }

    #[test]
    fn replacing_the_fuel_records_leaves_only_the_new_ones() {
        honouring_for_storing_fuel_records::replacing_the_fuel_records_leaves_only_the_new_ones(
            &FuelRecordsInMemory::default(),
        );
    }

    #[test]
    fn meals_come_back_by_name_a_meal_cooked_again_replaces_the_earlier_and_a_removed_one_leaves() {
        honouring_for_storing_fuel_records::meals_come_back_by_name_a_meal_cooked_again_replaces_the_earlier_and_a_removed_one_leaves(
            &FuelRecordsInMemory::default(),
        );
    }

    #[test]
    fn no_switch_over_until_one_is_saved_and_the_latest_one_stands() {
        honouring_for_storing_switch_over::no_switch_over_until_one_is_saved_and_the_latest_one_stands(
            &SwitchOverInMemory::default(),
        );
    }

    #[test]
    fn crafted_circuits_come_back_by_name_with_their_steps_in_order() {
        honouring_for_storing_burn_records::crafted_circuits_come_back_by_name_with_their_steps_in_order(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn removing_a_circuit_leaves_the_others() {
        honouring_for_storing_burn_records::removing_a_circuit_leaves_the_others(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_circuit_crafted_again_under_its_name_replaces_the_earlier_one() {
        honouring_for_storing_burn_records::a_circuit_crafted_again_under_its_name_replaces_the_earlier_one(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_workout_done_in_a_circuit_comes_back_with_its_name() {
        honouring_for_storing_burn_records::a_workout_done_in_a_circuit_comes_back_with_its_name(
            &BurnRecordsInMemory::default(),
        );
    }

    #[test]
    fn hydrate_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_hydrate_records::hydrate_records_come_back_in_the_order_of_their_days(
            &HydrateRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_drink_or_a_base_saved_again_under_its_name_or_day_replaces_the_earlier_one() {
        honouring_for_storing_hydrate_records::a_drink_or_a_base_saved_again_under_its_name_or_day_replaces_the_earlier_one(
            &HydrateRecordsInMemory::default(),
        );
    }

    #[test]
    fn removing_a_drinking_or_a_drink_leaves_the_others() {
        honouring_for_storing_hydrate_records::removing_a_drinking_or_a_drink_leaves_the_others(
            &HydrateRecordsInMemory::default(),
        );
    }

    #[test]
    fn replacing_the_hydrate_records_leaves_only_the_new_ones() {
        honouring_for_storing_hydrate_records::replacing_the_hydrate_records_leaves_only_the_new_ones(
            &HydrateRecordsInMemory::default(),
        );
    }
}
