use serde::{Deserialize, Serialize};

use crate::food::{Food, Nutrients, Unit};
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub food: String,
    pub unit: Unit,
    pub quantity: f64,
    pub nutrients: Nutrients,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Cooking {
    Weighed { cooked_g: f64 },
    Ratio { cooked_per_raw: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Meal {
    pub name: String,
    pub recipe: String,
    pub ingredients: Vec<Ingredient>,
    pub raw_g: f64,
    pub cooked_g: f64,
    pub totals: Nutrients,
}

pub fn meal(
    name: &str,
    recipe: &str,
    ingredients: &[(Food, f64)],
    cooking: Cooking,
) -> Result<Meal, Refusal> {
    let _ = (name, recipe, ingredients, cooking);
    Err(Refusal::NotAName)
}

pub fn food_of(meal: &Meal) -> Food {
    let _ = meal;
    Food {
        name: String::new(),
        unit: Unit::Gram,
        portion: 100.0,
        nutrients: Nutrients::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn per_100_g(name: &str, energy_kcal: f64, protein_g: f64) -> Food {
        Food {
            name: name.into(),
            unit: Unit::Gram,
            portion: 100.0,
            nutrients: Nutrients {
                energy_kcal,
                protein_g,
                carbs_g: 0.0,
                fat_g: 0.0,
                fibre_g: 0.0,
            },
        }
    }

    fn an_egg() -> Food {
        Food {
            name: "Egg".into(),
            unit: Unit::Piece,
            portion: 1.0,
            nutrients: Nutrients {
                energy_kcal: 70.0,
                protein_g: 6.0,
                ..Nutrients::default()
            },
        }
    }

    fn rice_and_chicken(cooking: Cooking) -> Result<Meal, Refusal> {
        meal(
            " Rice and chicken ",
            "Boil the rice, sear the chicken.",
            &[
                (per_100_g("Rice", 350.0, 7.0), 200.0),
                (per_100_g("Chicken", 120.0, 23.0), 300.0),
            ],
            cooking,
        )
    }

    #[test]
    fn a_meal_adds_up_its_ingredients_before_cooking_and_keeps_its_recipe() {
        let cooked = rice_and_chicken(Cooking::Weighed { cooked_g: 800.0 }).expect("a meal");

        assert_eq!(cooked.name, "Rice and chicken");
        assert_eq!(cooked.recipe, "Boil the rice, sear the chicken.");
        assert_eq!(cooked.raw_g, 500.0);
        assert!((cooked.totals.energy_kcal - 1060.0).abs() < 1e-9);
        assert!((cooked.totals.protein_g - 83.0).abs() < 1e-9);
    }

    #[test]
    fn the_cooked_mass_is_weighed_or_the_raw_mass_times_its_ratio() {
        let weighed = rice_and_chicken(Cooking::Weighed { cooked_g: 800.0 }).expect("a meal");
        let shrunk = rice_and_chicken(Cooking::Ratio {
            cooked_per_raw: 0.8,
        })
        .expect("a meal");

        assert_eq!(weighed.cooked_g, 800.0);
        assert!((shrunk.cooked_g - 400.0).abs() < 1e-9);
    }

    #[test]
    fn a_cooked_meal_joins_the_foods_with_its_figures_per_100_g_of_cooked_mass() {
        let cooked = rice_and_chicken(Cooking::Weighed { cooked_g: 800.0 }).expect("a meal");

        let food = food_of(&cooked);

        assert_eq!(
            (food.name.as_str(), food.unit, food.portion),
            ("Rice and chicken", Unit::Gram, 100.0)
        );
        assert!((food.nutrients.energy_kcal - 132.5).abs() < 1e-9);
        assert!((food.nutrients.protein_g - 10.375).abs() < 1e-9);
    }

    #[test]
    fn pieces_bring_their_figures_but_no_raw_mass_so_the_pot_must_be_weighed() {
        let omelette = meal(
            "Omelette",
            "",
            &[(an_egg(), 3.0)],
            Cooking::Ratio {
                cooked_per_raw: 0.9,
            },
        );
        let weighed = meal(
            "Omelette",
            "",
            &[(an_egg(), 3.0)],
            Cooking::Weighed { cooked_g: 150.0 },
        )
        .expect("a meal");

        assert_eq!(omelette, Err(Refusal::NotAPortion));
        assert!((weighed.totals.energy_kcal - 210.0).abs() < 1e-9);
    }

    #[test]
    fn a_nameless_meal_no_ingredient_or_no_cooked_mass_is_refused() {
        let weighed = Cooking::Weighed { cooked_g: 800.0 };

        assert_eq!(
            meal(" ", "", &[(per_100_g("Rice", 350.0, 7.0), 200.0)], weighed),
            Err(Refusal::NotAName)
        );
        assert_eq!(meal("Rice", "", &[], weighed), Err(Refusal::NotAQuantity));
        assert_eq!(
            meal("Rice", "", &[(per_100_g("Rice", 350.0, 7.0), 0.0)], weighed),
            Err(Refusal::NotAQuantity)
        );
        assert_eq!(
            rice_and_chicken(Cooking::Weighed { cooked_g: 0.0 }),
            Err(Refusal::NotAPortion)
        );
    }
}
