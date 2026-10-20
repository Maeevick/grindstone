use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::food::Food;
use crate::intake::Intake;
use crate::kept::Kept;
use crate::meal::Meal;
use crate::refusal::Refusal;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelTargets {
    pub protein_g_per_kg: f64,
    pub fat_g_per_kg: f64,
    pub fibre_g_per_kg: f64,
}

pub const DEFAULT_FUEL_TARGETS: FuelTargets = FuelTargets {
    protein_g_per_kg: 1.6,
    fat_g_per_kg: 0.9,
    fibre_g_per_kg: 0.4,
};

pub fn fuel_targets(
    protein_g_per_kg: f64,
    fat_g_per_kg: f64,
    fibre_g_per_kg: f64,
) -> Result<FuelTargets, Refusal> {
    if ![protein_g_per_kg, fat_g_per_kg, fibre_g_per_kg]
        .into_iter()
        .all(|figure| figure.is_finite() && figure >= 0.0)
    {
        return Err(Refusal::NotANutrient);
    }
    Ok(FuelTargets {
        protein_g_per_kg,
        fat_g_per_kg,
        fibre_g_per_kg,
    })
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FuelRecords {
    pub foods: Vec<Food>,
    pub intakes: Vec<(i64, Intake)>,
    pub targets: Vec<(NaiveDate, Kept<FuelTargets>)>,
    pub meals: Vec<Meal>,
}
