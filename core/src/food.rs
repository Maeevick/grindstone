use serde::{Deserialize, Serialize};

use crate::refusal::Refusal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Gram,
    Millilitre,
    Piece,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Nutrients {
    pub energy_kcal: f64,
    pub protein_g: f64,
    pub carbs_g: f64,
    pub fat_g: f64,
    pub fibre_g: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Food {
    pub name: String,
    pub unit: Unit,
    pub portion: f64,
    pub nutrients: Nutrients,
    #[serde(default)]
    pub description: String,
}

fn is_nothing_or_more(figure: f64) -> bool {
    figure.is_finite() && figure >= 0.0
}

pub fn food(
    name: &str,
    unit: Unit,
    portion: f64,
    nutrients: Nutrients,
    description: &str,
) -> Result<Food, Refusal> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Refusal::NotAName);
    }
    if !(portion.is_finite() && portion > 0.0) {
        return Err(Refusal::NotAPortion);
    }
    let Nutrients {
        energy_kcal,
        protein_g,
        carbs_g,
        fat_g,
        fibre_g,
    } = nutrients;
    if ![energy_kcal, protein_g, carbs_g, fat_g, fibre_g]
        .into_iter()
        .all(is_nothing_or_more)
    {
        return Err(Refusal::NotANutrient);
    }
    Ok(Food {
        name: name.into(),
        unit,
        portion,
        nutrients,
        description: description.trim().into(),
    })
}

pub fn nutrients_of(food: &Food, quantity: f64) -> Nutrients {
    let share = quantity / food.portion;
    let Nutrients {
        energy_kcal,
        protein_g,
        carbs_g,
        fat_g,
        fibre_g,
    } = food.nutrients;
    Nutrients {
        energy_kcal: energy_kcal * share,
        protein_g: protein_g * share,
        carbs_g: carbs_g * share,
        fat_g: fat_g * share,
        fibre_g: fibre_g * share,
    }
}

pub fn summed(all: impl IntoIterator<Item = Nutrients>) -> Nutrients {
    all.into_iter()
        .fold(Nutrients::default(), |sum, more| Nutrients {
            energy_kcal: sum.energy_kcal + more.energy_kcal,
            protein_g: sum.protein_g + more.protein_g,
            carbs_g: sum.carbs_g + more.carbs_g,
            fat_g: sum.fat_g + more.fat_g,
            fibre_g: sum.fibre_g + more.fibre_g,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKYR: Nutrients = Nutrients {
        energy_kcal: 54.0,
        protein_g: 9.5,
        carbs_g: 4.0,
        fat_g: 0.0,
        fibre_g: 0.0,
    };

    #[test]
    fn a_food_keeps_its_trimmed_name_and_its_nutrients_per_portion() {
        assert_eq!(
            food(" Skyr ", Unit::Gram, 100.0, SKYR, ""),
            Ok(Food {
                name: "Skyr".into(),
                unit: Unit::Gram,
                portion: 100.0,
                nutrients: SKYR,
                description: String::new(),
            })
        );
    }

    #[test]
    fn a_food_without_a_name_a_portion_or_readable_nutrients_is_refused() {
        assert_eq!(
            food(" ", Unit::Gram, 100.0, SKYR, ""),
            Err(Refusal::NotAName)
        );
        assert_eq!(
            food("Skyr", Unit::Gram, 0.0, SKYR, ""),
            Err(Refusal::NotAPortion)
        );
        assert_eq!(
            food(
                "Skyr",
                Unit::Gram,
                100.0,
                Nutrients {
                    protein_g: -1.0,
                    ..SKYR
                },
                ""
            ),
            Err(Refusal::NotANutrient)
        );
        assert_eq!(
            food(
                "Skyr",
                Unit::Gram,
                100.0,
                Nutrients {
                    energy_kcal: f64::NAN,
                    ..SKYR
                },
                ""
            ),
            Err(Refusal::NotANutrient)
        );
    }

    #[test]
    fn a_quantity_eaten_scales_the_nutrients_of_the_portion() {
        let skyr = food("Skyr", Unit::Gram, 100.0, SKYR, "").expect("a food");

        assert_eq!(
            nutrients_of(&skyr, 150.0),
            Nutrients {
                energy_kcal: 81.0,
                protein_g: 14.25,
                carbs_g: 6.0,
                fat_g: 0.0,
                fibre_g: 0.0,
            }
        );
    }

    #[test]
    fn every_nutrient_of_the_portion_scales_with_the_quantity_eaten() {
        let muesli = food(
            "Muesli",
            Unit::Gram,
            100.0,
            Nutrients {
                energy_kcal: 365.0,
                protein_g: 21.0,
                carbs_g: 43.0,
                fat_g: 9.4,
                fibre_g: 12.0,
            },
            "",
        )
        .expect("a food");

        let eaten = nutrients_of(&muesli, 50.0);

        assert_eq!(
            [
                eaten.energy_kcal,
                eaten.protein_g,
                eaten.carbs_g,
                eaten.fat_g,
                eaten.fibre_g
            ],
            [182.5, 10.5, 21.5, 4.7, 6.0]
        );
    }

    #[test]
    fn nutrients_sum_up_figure_by_figure() {
        let some = Nutrients {
            energy_kcal: 100.0,
            protein_g: 10.0,
            carbs_g: 20.0,
            fat_g: 3.0,
            fibre_g: 4.0,
        };
        let more = Nutrients {
            energy_kcal: 50.0,
            protein_g: 5.0,
            carbs_g: 1.0,
            fat_g: 2.0,
            fibre_g: 0.5,
        };

        assert_eq!(
            summed([some, more]),
            Nutrients {
                energy_kcal: 150.0,
                protein_g: 15.0,
                carbs_g: 21.0,
                fat_g: 5.0,
                fibre_g: 4.5,
            }
        );
        assert_eq!(summed([]), Nutrients::default());
    }
}
