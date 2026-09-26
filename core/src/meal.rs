use serde::{Deserialize, Serialize};

use crate::food::{Food, Nutrients, Unit, nutrients_of, summed};
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ingredient {
    pub food: String,
    pub unit: Unit,
    pub quantity: f64,
    pub cooked_per_raw: f64,
    pub nutrients: Nutrients,
}

impl Ingredient {
    pub fn cooked_g(&self) -> f64 {
        self.quantity * self.cooked_per_raw
    }
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

/// The cooking ratio of an ingredient whose mass cooking leaves as it was.
pub const MASS_KEPT: f64 = 1.0;

/// A meal from its ingredients, each with its quantity and the ratio of its cooked mass to its raw
/// one: 2.5 for pasta that drinks its water, 0.66 for cream that boils down. Its energy and macros
/// add up from the raw quantities, its cooked mass from each quantity times its ratio. Millilitres
/// weigh as grams until the catalog knows densities; an ingredient counted in pieces has no mass yet.
pub fn meal(name: &str, recipe: &str, ingredients: &[(Food, f64, f64)]) -> Result<Meal, Refusal> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Refusal::NotAName);
    }
    if ingredients.is_empty()
        || ingredients
            .iter()
            .any(|(_, quantity, _)| !(quantity.is_finite() && *quantity > 0.0))
    {
        return Err(Refusal::NotAQuantity);
    }
    if ingredients
        .iter()
        .any(|(_, _, cooked_per_raw)| !(cooked_per_raw.is_finite() && *cooked_per_raw > 0.0))
    {
        return Err(Refusal::NotACookingRatio);
    }
    if ingredients
        .iter()
        .any(|(food, _, _)| food.unit == Unit::Piece)
    {
        return Err(Refusal::NotWeighed);
    }
    let ingredients: Vec<Ingredient> = ingredients
        .iter()
        .map(|(food, quantity, cooked_per_raw)| Ingredient {
            food: food.name.clone(),
            unit: food.unit,
            quantity: *quantity,
            cooked_per_raw: *cooked_per_raw,
            nutrients: nutrients_of(food, *quantity),
        })
        .collect();
    Ok(Meal {
        name: name.into(),
        recipe: recipe.trim().into(),
        totals: summed(ingredients.iter().map(|ingredient| ingredient.nutrients)),
        raw_g: ingredients
            .iter()
            .map(|ingredient| ingredient.quantity)
            .sum(),
        cooked_g: ingredients.iter().map(Ingredient::cooked_g).sum(),
        ingredients,
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

    fn pasta_in_cream() -> Result<Meal, Refusal> {
        meal(
            " Pasta in cream ",
            "Boil the pasta, reduce the cream, stir.",
            &[
                (per_100_g("Pasta", 360.0, 12.0), 100.0, 2.5),
                (per_100_g("Cream", 300.0, 2.0), 200.0, 0.66),
            ],
        )
    }

    #[test]
    fn a_meal_adds_up_its_ingredients_raw_and_keeps_its_recipe() {
        let cooked = pasta_in_cream().expect("a meal");

        assert_eq!(cooked.name, "Pasta in cream");
        assert_eq!(cooked.recipe, "Boil the pasta, reduce the cream, stir.");
        assert_eq!(cooked.raw_g, 300.0);
        assert!((cooked.totals.energy_kcal - 960.0).abs() < 1e-9);
        assert!((cooked.totals.protein_g - 16.0).abs() < 1e-9);
    }

    #[test]
    fn each_ingredient_cooks_to_its_quantity_times_its_ratio() {
        let cooked = pasta_in_cream().expect("a meal");

        assert!((cooked.cooked_g - (250.0 + 132.0)).abs() < 1e-9);
        assert_eq!(
            cooked
                .ingredients
                .iter()
                .map(|ingredient| ingredient.cooked_per_raw)
                .collect::<Vec<_>>(),
            vec![2.5, 0.66]
        );
    }

    #[test]
    fn a_cooked_meal_joins_the_foods_with_its_figures_per_100_g_of_cooked_mass() {
        let cooked = meal(
            "Rice and chicken",
            "",
            &[
                (per_100_g("Rice", 350.0, 7.0), 200.0, 2.5),
                (per_100_g("Chicken", 120.0, 23.0), 300.0, 1.0),
            ],
        )
        .expect("a meal");

        let food = food_of(&cooked);

        assert_eq!(
            (food.name.as_str(), food.unit, food.portion),
            ("Rice and chicken", Unit::Gram, 100.0)
        );
        assert!((food.nutrients.energy_kcal - 132.5).abs() < 1e-9);
        assert!((food.nutrients.protein_g - 10.375).abs() < 1e-9);
    }

    #[test]
    fn millilitres_weigh_as_grams_until_the_catalog_knows_densities() {
        let milk = Food {
            unit: Unit::Millilitre,
            ..per_100_g("Milk", 46.0, 3.4)
        };

        let porridge = meal("Porridge", "", &[(milk, 250.0, 0.9)]).expect("a meal");

        assert_eq!(porridge.raw_g, 250.0);
        assert!((porridge.cooked_g - 225.0).abs() < 1e-9);
    }

    #[test]
    fn an_ingredient_counted_in_pieces_is_refused_until_the_catalog_weighs_its_pieces() {
        assert_eq!(
            meal("Omelette", "", &[(an_egg(), 3.0, 1.0)]),
            Err(Refusal::NotWeighed)
        );
    }

    #[test]
    fn a_nameless_meal_no_ingredient_no_quantity_or_no_cooking_ratio_is_refused() {
        let rice = per_100_g("Rice", 350.0, 7.0);

        assert_eq!(
            meal(" ", "", &[(rice.clone(), 200.0, 2.5)]),
            Err(Refusal::NotAName)
        );
        assert_eq!(meal("Rice", "", &[]), Err(Refusal::NotAQuantity));
        assert_eq!(
            meal("Rice", "", &[(rice.clone(), 0.0, 2.5)]),
            Err(Refusal::NotAQuantity)
        );
        for ratio in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                meal("Rice", "", &[(rice.clone(), 200.0, ratio)]),
                Err(Refusal::NotACookingRatio),
                "{ratio}"
            );
        }
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
        let dal = meal("Dal", "", &[(lentils, 200.0, 2.5)]).expect("a meal");

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

    #[test]
    fn each_ingredient_tells_the_mass_it_cooks_to() {
        let cooked = pasta_in_cream().expect("a meal");

        let masses: Vec<f64> = cooked
            .ingredients
            .iter()
            .map(Ingredient::cooked_g)
            .collect();

        assert!((masses[0] - 250.0).abs() < 1e-9 && (masses[1] - 132.0).abs() < 1e-9);
    }
}
