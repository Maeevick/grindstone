use chrono::NaiveDate;

use crate::failure::Broken;
use crate::gather_records::GatherRecords;
use crate::goals::Goals;
use crate::kept::Kept;
use crate::measurement::Measurement;
use crate::profile::Profile;

pub trait ForStoringGatherRecords {
    fn save_measurement(&self, measurement: &Measurement) -> Result<(), Broken>;
    fn remove_measurement(&self, day: NaiveDate) -> Result<(), Broken>;
    fn save_profile(&self, from: NaiveDate, profile: &Profile) -> Result<(), Broken>;
    fn save_goals(&self, from: NaiveDate, goals: &Kept<Goals>) -> Result<(), Broken>;
    fn replace_gather_records(&self, records: &GatherRecords) -> Result<(), Broken>;
    fn gather_records(&self) -> Result<GatherRecords, Broken>;
}
