use crate::exercise::{Exercise, exercise};
use crate::failure::Failure;
use crate::food::{Food, Nutrients, Unit, food};
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::meal::{Cooking, Meal, food_of, meal};
use crate::refusal::Refusal;
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
    Ok(fuel.fuel_records()?.foods)
}

pub fn add_food(fuel: &impl ForStoringFuelRecords, form: &FoodForm) -> Result<Vec<Food>, Failure> {
    let added =
        food(&form.name, form.unit, form.portion, form.nutrients).map_err(Failure::Refused)?;
    fuel.save_food(&added)?;
    foods(fuel)
}

pub fn remove_food(fuel: &impl ForStoringFuelRecords, name: &str) -> Result<Vec<Food>, Failure> {
    fuel.remove_food(name)?;
    foods(fuel)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IngredientForm {
    pub food: String,
    pub quantity: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MealForm {
    pub name: String,
    pub recipe: String,
    pub ingredients: Vec<IngredientForm>,
    pub cooking: Cooking,
}

pub fn meals(fuel: &impl ForStoringFuelRecords) -> Result<Vec<Meal>, Failure> {
    Ok(fuel.fuel_records()?.meals)
}

pub fn preview_meal(fuel: &impl ForStoringFuelRecords, form: &MealForm) -> Result<Meal, Failure> {
    let catalog = fuel.fuel_records()?.foods;
    let ingredients = form
        .ingredients
        .iter()
        .map(|ingredient| {
            catalog
                .iter()
                .find(|known| known.name == ingredient.food)
                .map(|known| (known.clone(), ingredient.quantity))
                .ok_or(Failure::Refused(Refusal::NotAFood))
        })
        .collect::<Result<Vec<_>, _>>()?;
    meal(&form.name, &form.recipe, &ingredients, form.cooking).map_err(Failure::Refused)
}

pub fn cook_meal(fuel: &impl ForStoringFuelRecords, form: &MealForm) -> Result<Vec<Meal>, Failure> {
    let cooked = preview_meal(fuel, form)?;
    fuel.save_meal(&cooked)?;
    fuel.save_food(&food_of(&cooked))?;
    meals(fuel)
}

pub fn remove_meal(fuel: &impl ForStoringFuelRecords, name: &str) -> Result<Vec<Meal>, Failure> {
    fuel.remove_meal(name)?;
    meals(fuel)
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

    fn a_kitchen() -> crate::testing::FuelRecordsInMemory {
        let fuel = crate::testing::FuelRecordsInMemory::default();
        for (name, energy_kcal) in [("Rice", 350.0), ("Chicken", 120.0)] {
            add_food(&fuel, &a_food_form(name, energy_kcal)).expect("a food adds");
        }
        fuel
    }

    fn rice_and_chicken() -> MealForm {
        MealForm {
            name: "Rice and chicken".into(),
            recipe: "Boil, sear, stir.".into(),
            ingredients: vec![
                IngredientForm {
                    food: "Rice".into(),
                    quantity: 200.0,
                },
                IngredientForm {
                    food: "Chicken".into(),
                    quantity: 300.0,
                },
            ],
            cooking: Cooking::Weighed { cooked_g: 800.0 },
        }
    }

    #[test]
    fn a_previewed_meal_tells_its_totals_without_cooking_anything() {
        let fuel = a_kitchen();

        let previewed = preview_meal(&fuel, &rice_and_chicken()).expect("a meal");

        assert!((previewed.totals.energy_kcal - 1060.0).abs() < 1e-9);
        assert_eq!(meals(&fuel), Ok(vec![]));
    }

    #[test]
    fn a_cooked_meal_keeps_its_recipe_and_joins_the_foods_per_100_g_cooked() {
        let fuel = a_kitchen();

        let cooked = cook_meal(&fuel, &rice_and_chicken()).expect("the meals");

        assert_eq!(
            cooked
                .iter()
                .map(|meal| meal.recipe.as_str())
                .collect::<Vec<_>>(),
            vec!["Boil, sear, stir."]
        );
        let as_a_food = foods(&fuel)
            .expect("the foods")
            .into_iter()
            .find(|food| food.name == "Rice and chicken")
            .expect("the meal as a food");
        assert!((as_a_food.nutrients.energy_kcal - 132.5).abs() < 1e-9);
    }

    #[test]
    fn a_meal_of_a_food_missing_from_the_catalog_is_refused_and_a_removed_meal_leaves() {
        let fuel = a_kitchen();
        cook_meal(&fuel, &rice_and_chicken()).expect("a meal cooks");
        let unknown = MealForm {
            ingredients: vec![IngredientForm {
                food: "Saffron".into(),
                quantity: 1.0,
            }],
            ..rice_and_chicken()
        };

        assert_eq!(
            preview_meal(&fuel, &unknown),
            Err(Failure::Refused(Refusal::NotAFood))
        );
        assert_eq!(remove_meal(&fuel, "Rice and chicken"), Ok(vec![]));
    }
}
