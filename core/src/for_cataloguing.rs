use crate::exercise::{Exercise, exercise};
use crate::failure::Failure;
use crate::food::{Food, Nutrients, Unit};
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use serde::{Deserialize, Serialize};

pub fn exercises(burn: &impl ForStoringBurnRecords) -> Result<Vec<Exercise>, Failure> {
    Ok(burn.burn_records()?.exercises)
}

pub fn add_exercise(
    burn: &impl ForStoringBurnRecords,
    name: &str,
    met: f64,
) -> Result<Vec<Exercise>, Failure> {
    burn.save_exercise(&exercise(name, met).map_err(Failure::Refused)?)?;
    exercises(burn)
}

pub fn remove_exercise(
    burn: &impl ForStoringBurnRecords,
    name: &str,
) -> Result<Vec<Exercise>, Failure> {
    burn.remove_exercise(name)?;
    exercises(burn)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoodForm {
    pub name: String,
    pub unit: Unit,
    pub portion: f64,
    pub nutrients: Nutrients,
}

pub fn foods(fuel: &impl ForStoringFuelRecords) -> Result<Vec<Food>, Failure> {
    let _ = fuel;
    Ok(vec![])
}

pub fn add_food(fuel: &impl ForStoringFuelRecords, form: &FoodForm) -> Result<Vec<Food>, Failure> {
    let _ = (fuel, form);
    Ok(vec![])
}

pub fn remove_food(fuel: &impl ForStoringFuelRecords, name: &str) -> Result<Vec<Food>, Failure> {
    let _ = (fuel, name);
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refusal::Refusal;
    use crate::testing::{BrokenBurnRecords, BurnRecordsInMemory};

    fn named(name: &str, met: f64) -> Exercise {
        Exercise {
            name: name.into(),
            met,
        }
    }

    #[test]
    fn added_exercises_come_back_in_the_catalog_by_name() {
        let burn = BurnRecordsInMemory::default();
        add_exercise(&burn, "Walking", 4.0).expect("an exercise adds");

        let catalog = add_exercise(&burn, "Biking", 8.0);

        assert_eq!(
            catalog,
            Ok(vec![named("Biking", 8.0), named("Walking", 4.0)])
        );
        assert_eq!(exercises(&burn), catalog);
    }

    #[test]
    fn an_exercise_added_again_under_its_name_takes_its_new_met() {
        let burn = BurnRecordsInMemory::default();
        add_exercise(&burn, "Walking", 3.5).expect("an exercise adds");

        assert_eq!(
            add_exercise(&burn, "Walking", 4.0),
            Ok(vec![named("Walking", 4.0)])
        );
    }

    #[test]
    fn a_removed_exercise_leaves_the_catalog() {
        let burn = BurnRecordsInMemory::default();
        add_exercise(&burn, "Walking", 4.0).expect("an exercise adds");

        assert_eq!(remove_exercise(&burn, "Walking"), Ok(vec![]));
    }

    #[test]
    fn a_nameless_exercise_or_broken_records_change_nothing() {
        assert_eq!(
            add_exercise(&BurnRecordsInMemory::default(), " ", 4.0),
            Err(Failure::Refused(Refusal::NotAName))
        );
        assert_eq!(
            remove_exercise(&BrokenBurnRecords, "Walking"),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    fn a_food_form(name: &str, energy_kcal: f64) -> FoodForm {
        FoodForm {
            name: name.into(),
            unit: Unit::Gram,
            portion: 100.0,
            nutrients: Nutrients {
                energy_kcal,
                protein_g: 9.5,
                carbs_g: 4.0,
                fat_g: 0.0,
                fibre_g: 0.0,
            },
        }
    }

    #[test]
    fn added_foods_come_back_in_the_catalog_by_name_and_a_food_added_again_takes_its_new_figures() {
        let fuel = crate::testing::FuelRecordsInMemory::default();
        add_food(&fuel, &a_food_form("Skyr", 50.0)).expect("a food adds");
        add_food(&fuel, &a_food_form("Muesli", 365.0)).expect("a food adds");

        let catalog = add_food(&fuel, &a_food_form("Skyr", 54.0)).expect("a catalog");

        assert_eq!(
            catalog
                .iter()
                .map(|food| (food.name.as_str(), food.nutrients.energy_kcal))
                .collect::<Vec<_>>(),
            vec![("Muesli", 365.0), ("Skyr", 54.0)]
        );
        assert_eq!(foods(&fuel), Ok(catalog));
    }

    #[test]
    fn a_removed_food_leaves_the_catalog_and_a_nameless_one_is_refused() {
        let fuel = crate::testing::FuelRecordsInMemory::default();
        add_food(&fuel, &a_food_form("Skyr", 54.0)).expect("a food adds");

        assert_eq!(remove_food(&fuel, "Skyr"), Ok(vec![]));
        assert_eq!(
            add_food(&fuel, &a_food_form(" ", 54.0)),
            Err(Failure::Refused(Refusal::NotAName))
        );
    }
}
