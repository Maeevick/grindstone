use crate::failure::Broken;
use crate::gather_records::GatherRecords;
use crate::measurement::Measurement;

pub trait ForStoringGatherRecords {
    fn save_measurement(&self, measurement: &Measurement) -> Result<(), Broken>;
    fn gather_records(&self) -> Result<GatherRecords, Broken>;
}
