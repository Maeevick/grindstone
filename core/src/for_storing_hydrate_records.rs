use chrono::NaiveDate;

use crate::drink::Drink;
use crate::drinking::Drinking;
use crate::failure::Broken;
use crate::hydrate_records::HydrateRecords;

pub trait ForStoringHydrateRecords {
    fn save_drink(&self, drink: &Drink) -> Result<(), Broken>;
    fn remove_drink(&self, name: &str) -> Result<(), Broken>;
    fn save_drinking(&self, drinking: &Drinking) -> Result<(), Broken>;
    fn remove_drinking(&self, id: i64) -> Result<(), Broken>;
    fn save_counted(&self, from: NaiveDate, counted: bool) -> Result<(), Broken>;
    fn replace_hydrate_records(&self, records: &HydrateRecords) -> Result<(), Broken>;
    fn hydrate_records(&self) -> Result<HydrateRecords, Broken>;
}
