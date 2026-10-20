use chrono::NaiveDate;

use crate::failure::Broken;
use crate::food::Food;
use crate::fuel_records::{FuelRecords, FuelTargets};
use crate::intake::Intake;
use crate::kept::Kept;
use crate::meal::Meal;

pub trait ForStoringFuelRecords {
    fn save_food(&self, food: &Food) -> Result<(), Broken>;
    fn remove_food(&self, name: &str) -> Result<(), Broken>;
    fn save_intake(&self, intake: &Intake) -> Result<(), Broken>;
    fn remove_intake(&self, id: i64) -> Result<(), Broken>;
    fn save_targets(&self, from: NaiveDate, targets: &Kept<FuelTargets>) -> Result<(), Broken>;
    fn save_meal(&self, meal: &Meal) -> Result<(), Broken>;
    fn remove_meal(&self, name: &str) -> Result<(), Broken>;
    fn replace_fuel_records(&self, records: &FuelRecords) -> Result<(), Broken>;
    fn fuel_records(&self) -> Result<FuelRecords, Broken>;
}
