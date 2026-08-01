use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::food::Food;
use crate::intake::Intake;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelTargets {
    pub protein_g_per_kg: f64,
    pub fat_g_per_kg: f64,
    pub fibre_g_per_kg: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FuelRecords {
    pub foods: Vec<Food>,
    pub intakes: Vec<(i64, Intake)>,
    pub targets: Vec<(NaiveDate, FuelTargets)>,
}
