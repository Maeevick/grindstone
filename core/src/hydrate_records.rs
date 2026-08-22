use chrono::NaiveDate;

use crate::drink::Drink;
use crate::drinking::Drinking;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HydrateRecords {
    pub drinks: Vec<Drink>,
    pub drinkings: Vec<(i64, Drinking)>,
    pub bases: Vec<(NaiveDate, f64)>,
}
