use chrono::NaiveDate;

use crate::drink::Drink;
use crate::drinking::Drinking;
use crate::kept::Kept;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HydrateRecords {
    pub drinks: Vec<Drink>,
    pub drinkings: Vec<(i64, Drinking)>,
    pub targets: Vec<(NaiveDate, Kept<f64>)>,
}
