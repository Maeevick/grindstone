use chrono::NaiveDate;

use crate::burn_records::BurnRecords;
use crate::circuit::Circuit;
use crate::exercise::Exercise;
use crate::failure::Broken;
use crate::kept::Kept;
use crate::volitional::Volitional;
use crate::workout::Workout;

pub trait ForStoringBurnRecords {
    fn save_exercise(&self, exercise: &Exercise) -> Result<(), Broken>;
    fn remove_exercise(&self, name: &str) -> Result<(), Broken>;
    fn save_workout(&self, workout: &Workout) -> Result<(), Broken>;
    fn remove_workout(&self, id: i64) -> Result<(), Broken>;
    fn save_day_off_the_clock(&self, day: NaiveDate, volitional: &Volitional)
    -> Result<(), Broken>;
    fn save_usual_day(&self, from: NaiveDate, usual: &Kept<Volitional>) -> Result<(), Broken>;
    fn save_burn_target(&self, from: NaiveDate, kcal: Kept<f64>) -> Result<(), Broken>;
    fn replace_burn_records(&self, records: &BurnRecords) -> Result<(), Broken>;
    fn burn_records(&self) -> Result<BurnRecords, Broken>;
    fn save_circuit(&self, circuit: &Circuit) -> Result<(), Broken>;
    fn remove_circuit(&self, name: &str) -> Result<(), Broken>;
}
