use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::food::Food;
use crate::intake::Intake;
use crate::kept::Kept;
use crate::meal::Meal;
use crate::refusal::Refusal;

/// The goals of a day's four macros, each per kilo of body mass, none until one is painted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FuelTargets {
    pub protein_g_per_kg: Option<f64>,
    pub fat_g_per_kg: Option<f64>,
    pub fibre_g_per_kg: Option<f64>,
    pub carbs_g_per_kg: Option<f64>,
}

/// Goals of no figure, or below nothing, are refused; any other, healthy or not, is the body's own to set.
pub fn fuel_targets(targets: FuelTargets) -> Result<FuelTargets, Refusal> {
    if ![
        targets.protein_g_per_kg,
        targets.fat_g_per_kg,
        targets.fibre_g_per_kg,
        targets.carbs_g_per_kg,
    ]
    .into_iter()
    .flatten()
    .all(|figure| figure.is_finite() && figure >= 0.0)
    {
        return Err(Refusal::NotANutrient);
    }
    Ok(targets)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FuelRecords {
    pub foods: Vec<Food>,
    pub intakes: Vec<(i64, Intake)>,
    pub targets: Vec<(NaiveDate, Kept<FuelTargets>)>,
    pub meals: Vec<Meal>,
}
