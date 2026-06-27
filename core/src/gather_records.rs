use chrono::NaiveDate;

use crate::goals::Goals;
use crate::measurement::Measurement;
use crate::profile::Profile;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GatherRecords {
    pub measurements: Vec<Measurement>,
    pub profiles: Vec<(NaiveDate, Profile)>,
    pub goals: Vec<(NaiveDate, Goals)>,
}
