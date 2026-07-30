use serde::{Deserialize, Serialize};

use crate::burn_records::BurnRecords;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::gather_records::GatherRecords;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Imported {
    pub measurements: usize,
    pub profiles: usize,
    pub goals: usize,
    pub exercises: usize,
    pub workouts: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drafted {
    pub gather: GatherRecords,
    pub burn: BurnRecords,
}

pub fn import_gather(
    records: &impl ForStoringGatherRecords,
    imported: &GatherRecords,
) -> Result<Imported, Failure> {
    records.replace_gather_records(imported)?;
    Ok(Imported {
        measurements: imported.measurements.len(),
        profiles: imported.profiles.len(),
        goals: imported.goals.len(),
        ..Imported::default()
    })
}

pub fn import_drafted(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    drafted: &Drafted,
) -> Result<Imported, Failure> {
    let gathered = import_gather(gather, &drafted.gather)?;
    burn.replace_burn_records(&drafted.burn)?;
    Ok(Imported {
        exercises: drafted.burn.exercises.len(),
        workouts: drafted.burn.workouts.len(),
        ..gathered
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;
    use crate::goals::Goals;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::testing::{BrokenGatherRecords, GatherRecordsInMemory};
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

    #[test]
    fn an_import_counts_what_it_brought() {
        let imported = import_gather(&GatherRecordsInMemory::default(), &drafted());

        assert_eq!(
            imported,
            Ok(Imported {
                measurements: 2,
                profiles: 1,
                goals: 1,
                ..Imported::default()
            })
        );
    }

    #[test]
    fn importing_the_same_records_twice_ends_where_importing_once_did() {
        let records = GatherRecordsInMemory::default();
        import_gather(&records, &drafted()).expect("a first import");

        import_gather(&records, &drafted()).expect("a second import");

        assert_eq!(records.gather_records(), Ok(drafted()));
    }

    #[test]
    fn records_that_cannot_be_replaced_give_no_import() {
        assert_eq!(
            import_gather(&BrokenGatherRecords, &drafted()),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    fn drafted_burn() -> BurnRecords {
        BurnRecords {
            exercises: vec![crate::exercise::Exercise {
                name: "Walking".into(),
                met: 4.0,
            }],
            workouts: vec![(
                0,
                crate::workout::Workout {
                    day: day("2026-08-10"),
                    exercise: "Walking".into(),
                    met: 4.0,
                    minutes: 30.0,
                    reps: None,
                    sets: 1,
                    load_kg: 0.0,
                },
            )],
            ..BurnRecords::default()
        }
    }

    #[test]
    fn importing_the_draft_replaces_gather_and_burn_and_counts_what_it_brought() {
        let gather = GatherRecordsInMemory::default();
        let burn = crate::testing::BurnRecordsInMemory::default();

        let imported = import_drafted(
            &gather,
            &burn,
            &Drafted {
                gather: drafted(),
                burn: drafted_burn(),
            },
        );

        assert_eq!(
            imported,
            Ok(Imported {
                measurements: 2,
                profiles: 1,
                goals: 1,
                exercises: 1,
                workouts: 1,
            })
        );
        assert_eq!(gather.gather_records(), Ok(drafted()));
        assert_eq!(
            burn.burn_records().map(|burnt| burnt.exercises),
            Ok(drafted_burn().exercises)
        );
    }

    #[test]
    fn burn_records_that_cannot_be_replaced_give_no_import() {
        assert_eq!(
            import_drafted(
                &GatherRecordsInMemory::default(),
                &crate::testing::BrokenBurnRecords,
                &Drafted::default(),
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }
}
