use serde::{Deserialize, Serialize};

use crate::food::{Food, Nutrients, Unit, nutrients_of, summed};
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

fn scaled(nutrients: Nutrients, share: f64) -> Nutrients {
    Nutrients {
        energy_kcal: nutrients.energy_kcal * share,
        protein_g: nutrients.protein_g * share,
        carbs_g: nutrients.carbs_g * share,
        fat_g: nutrients.fat_g * share,
        fibre_g: nutrients.fibre_g * share,
    }
}

pub fn meal(
    name: &str,
    recipe: &str,
    ingredients: &[(Food, f64)],
    cooking: Cooking,
) -> Result<Meal, Refusal> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Refusal::NotAName);
    }
    if ingredients.is_empty()
        || ingredients
            .iter()
            .any(|(_, quantity)| !(quantity.is_finite() && *quantity > 0.0))
    {
        return Err(Refusal::NotAQuantity);
    }
    let ingredients: Vec<Ingredient> = ingredients
        .iter()
        .map(|(food, quantity)| Ingredient {
            food: food.name.clone(),
            unit: food.unit,
            quantity: *quantity,
            nutrients: nutrients_of(food, *quantity),
        })
        .collect();
    let raw_g: f64 = ingredients
        .iter()
        .filter(|ingredient| ingredient.unit != Unit::Piece)
        .map(|ingredient| ingredient.quantity)
        .sum();
    let cooked_g = match cooking {
        Cooking::Weighed { cooked_g } => cooked_g,
        Cooking::Ratio { cooked_per_raw } => raw_g * cooked_per_raw,
    };
    if !(cooked_g.is_finite() && cooked_g > 0.0) {
        return Err(Refusal::NotAPortion);
    }
    Ok(Meal {
        name: name.into(),
        recipe: recipe.trim().into(),
        totals: summed(ingredients.iter().map(|ingredient| ingredient.nutrients)),
        ingredients,
        raw_g,
        cooked_g,
    })
}

pub fn food_of(meal: &Meal) -> Food {
    Food {
        name: meal.name.clone(),
        unit: Unit::Gram,
        portion: 100.0,
        nutrients: scaled(meal.totals, 100.0 / meal.cooked_g),
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

    #[test]
    fn every_figure_of_a_cooked_meal_is_given_per_100_g_cooked() {
        let lentils = Food {
            name: "Lentils".into(),
            unit: Unit::Gram,
            portion: 100.0,
            nutrients: Nutrients {
                energy_kcal: 350.0,
                protein_g: 24.0,
                carbs_g: 50.0,
                fat_g: 2.0,
                fibre_g: 10.0,
            },
        };
        let dal = meal(
            "Dal",
            "",
            &[(lentils, 200.0)],
            Cooking::Weighed { cooked_g: 500.0 },
        )
        .expect("a meal");

        let per_100_g = food_of(&dal).nutrients;

        let figures = [
            per_100_g.energy_kcal,
            per_100_g.protein_g,
            per_100_g.carbs_g,
            per_100_g.fat_g,
            per_100_g.fibre_g,
        ];
        for (figure, expected) in figures.into_iter().zip([140.0, 9.6, 20.0, 0.8, 4.0]) {
            assert!((figure - expected).abs() < 1e-9, "{figure} {expected}");
        }
    }
}
