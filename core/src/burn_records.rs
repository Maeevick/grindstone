use chrono::NaiveDate;

use crate::circuit::Circuit;
use crate::exercise::Exercise;
use crate::kept::Kept;
use crate::volitional::Volitional;
use crate::workout::Workout;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BurnRecords {
    pub exercises: Vec<Exercise>,
    pub workouts: Vec<(i64, Workout)>,
    pub days_off_the_clock: Vec<(NaiveDate, Volitional)>,
    pub usual_days: Vec<(NaiveDate, Kept<Volitional>)>,
    pub circuits: Vec<Circuit>,
    pub targets: Vec<(NaiveDate, Kept<f64>)>,
    pub counted_from: Vec<(NaiveDate, bool)>,
}
