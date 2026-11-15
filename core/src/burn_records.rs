use chrono::NaiveDate;

use crate::circuit::Circuit;
use crate::exercise::Exercise;
use crate::kept::Kept;
use crate::workout::Workout;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BurnRecords {
    pub exercises: Vec<Exercise>,
    pub workouts: Vec<(i64, Workout)>,
    pub circuits: Vec<Circuit>,
    pub targets: Vec<(NaiveDate, Kept<f64>)>,
}
