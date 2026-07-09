use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::gather_records::GatherRecords;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Imported {
    pub measurements: usize,
    pub profiles: usize,
    pub goals: usize,
}

pub fn import_gather(
    records: &impl ForStoringGatherRecords,
    imported: &GatherRecords,
) -> Result<Imported, Failure> {
    Ok(Imported::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;
    use crate::goals::Goals;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::testing::GatherRecordsInMemory;
    use chrono::NaiveDate;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn drafted() -> GatherRecords {
        GatherRecords {
            measurements: vec![
                Measurement {
                    day: day("2026-03-01"),
                    body_mass_kg: 79.5,
                    waist_cm: None,
                },
                Measurement {
                    day: day("2026-06-22"),
                    body_mass_kg: 76.4,
                    waist_cm: Some(84.0),
                },
            ],
            profiles: vec![(
                day("2026-03-01"),
                Profile {
                    birthday: day("1985-04-12"),
                    gender: Gender::Male,
                    height_cm: 180.0,
                    neck_cm: 38.5,
                    hip_cm: None,
                },
            )],
            goals: vec![(
                day("2026-03-01"),
                Goals {
                    body_mass_kg: 70.0,
                    fat_ratio: 0.10,
                },
            )],
        }
    }

    #[test]
    fn importing_gather_records_replaces_what_was_gathered() {
        let records = GatherRecordsInMemory::default();
        records
            .save_measurement(&Measurement {
                day: day("2026-10-10"),
                body_mass_kg: 80.0,
                waist_cm: None,
            })
            .expect("a measurement saves");

        import_gather(&records, &drafted()).expect("an import");

        assert_eq!(records.gather_records(), Ok(drafted()));
    }
}
