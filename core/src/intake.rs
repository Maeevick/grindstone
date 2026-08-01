use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::food::{Food, Nutrients, Unit, nutrients_of};
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intake {
    pub day: NaiveDate,
    pub food: String,
    pub unit: Unit,
    pub quantity: f64,
    pub nutrients: Nutrients,
}

pub fn intake(date: &str, food: &Food, quantity: f64) -> Result<Intake, Refusal> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Refusal::NotADate)?;
    if !(quantity.is_finite() && quantity > 0.0) {
        return Err(Refusal::NotAQuantity);
    }
    Ok(Intake {
        day,
        food: food.name.clone(),
        unit: food.unit,
        quantity,
        nutrients: nutrients_of(food, quantity),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skyr() -> Food {
        Food {
            name: "Skyr".into(),
            unit: Unit::Gram,
            portion: 100.0,
            nutrients: Nutrients {
                energy_kcal: 54.0,
                protein_g: 9.5,
                carbs_g: 4.0,
                fat_g: 0.0,
                fibre_g: 0.0,
            },
        }
    }

    #[test]
    fn an_intake_freezes_what_its_quantity_of_the_food_brings_on_its_day() {
        let eaten = intake("2026-10-10", &skyr(), 200.0).expect("an intake");

        assert_eq!(
            (
                eaten.day.to_string(),
                eaten.food,
                eaten.quantity,
                eaten.nutrients.energy_kcal
            ),
            ("2026-10-10".into(), "Skyr".into(), 200.0, 108.0)
        );
    }

    #[test]
    fn an_intake_on_no_real_day_or_of_no_quantity_is_refused() {
        assert_eq!(intake("2026-02-30", &skyr(), 200.0), Err(Refusal::NotADate));
        assert_eq!(
            intake("2026-10-10", &skyr(), 0.0),
            Err(Refusal::NotAQuantity)
        );
    }
}
