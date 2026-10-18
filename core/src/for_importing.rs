use serde::{Deserialize, Serialize};

use crate::burn_records::BurnRecords;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::for_storing_switch_over::ForStoringSwitchOver;
use crate::fuel_records::FuelRecords;
use crate::gather_records::GatherRecords;
use crate::hydrate_records::HydrateRecords;
use crate::refusal::Refusal;
use chrono::NaiveDate;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Imported {
    pub measurements: usize,
    pub profiles: usize,
    pub goals: usize,
    pub exercises: usize,
    pub workouts: usize,
    pub foods: usize,
    pub intakes: usize,
    pub drinks: usize,
    pub drinkings: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Drafted {
    pub gather: GatherRecords,
    pub burn: BurnRecords,
    pub fuel: FuelRecords,
    pub hydrate: HydrateRecords,
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
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    switch_over: &impl ForStoringSwitchOver,
    drafted: &Drafted,
    today: &str,
) -> Result<Imported, Failure> {
    let switched_on = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    let gathered = import_gather(gather, &drafted.gather)?;
    burn.replace_burn_records(&drafted.burn)?;
    fuel.replace_fuel_records(&drafted.fuel)?;
    let (drinks, drinkings) = import_hydrate(hydrate, &drafted.hydrate)?;
    switch_over.save_switch_over(switched_on)?;
    Ok(Imported {
        exercises: drafted.burn.exercises.len(),
        workouts: drafted.burn.workouts.len(),
        foods: drafted.fuel.foods.len(),
        intakes: drafted.fuel.intakes.len(),
        drinks,
        drinkings,
        ..gathered
    })
}

pub fn import_hydrate(
    hydrate: &impl ForStoringHydrateRecords,
    imported: &HydrateRecords,
) -> Result<(usize, usize), Failure> {
    hydrate.replace_hydrate_records(imported)?;
    Ok((imported.drinks.len(), imported.drinkings.len()))
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
                    circuit: None,
                },
            )],
            ..BurnRecords::default()
        }
    }

    fn drafted_fuel() -> FuelRecords {
        FuelRecords {
            foods: vec![crate::food::Food {
                name: "Skyr".into(),
                unit: crate::food::Unit::Gram,
                portion: 100.0,
                nutrients: crate::food::Nutrients::default(),
            }],
            intakes: vec![(
                0,
                crate::intake::Intake {
                    day: day("2026-08-02"),
                    food: "Skyr".into(),
                    unit: crate::food::Unit::Gram,
                    quantity: 200.0,
                    nutrients: crate::food::Nutrients::default(),
                },
            )],
            ..FuelRecords::default()
        }
    }

    fn drafted_hydrate() -> HydrateRecords {
        let expresso = crate::drink::Drink {
            name: "Expresso".into(),
            volume_ml: 80.0,
            kind: crate::drink::DrinkKind::Other,
        };
        HydrateRecords {
            drinkings: vec![(
                0,
                crate::drinking::drinking("2026-08-02", &expresso, 1.0).expect("a drinking"),
            )],
            drinks: vec![expresso],
            ..HydrateRecords::default()
        }
    }

    #[test]
    fn a_draft_import_whose_hydrate_records_cannot_be_replaced_gives_no_import() {
        assert_eq!(
            import_drafted(
                &GatherRecordsInMemory::default(),
                &crate::testing::BurnRecordsInMemory::default(),
                &crate::testing::FuelRecordsInMemory::default(),
                &crate::testing::BrokenHydrateRecords,
                &crate::testing::SwitchOverInMemory::default(),
                &Drafted::default(),
                "2026-10-10",
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn fuel_records_that_cannot_be_replaced_give_no_import() {
        assert_eq!(
            import_drafted(
                &GatherRecordsInMemory::default(),
                &crate::testing::BurnRecordsInMemory::default(),
                &crate::testing::BrokenFuelRecords,
                &crate::testing::HydrateRecordsInMemory::default(),
                &crate::testing::SwitchOverInMemory::default(),
                &Drafted::default(),
                "2026-10-10",
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn importing_the_draft_replaces_gather_and_burn_and_counts_what_it_brought() {
        let gather = GatherRecordsInMemory::default();
        let burn = crate::testing::BurnRecordsInMemory::default();

        let fuel = crate::testing::FuelRecordsInMemory::default();

        let imported = import_drafted(
            &gather,
            &burn,
            &fuel,
            &crate::testing::HydrateRecordsInMemory::default(),
            &crate::testing::SwitchOverInMemory::default(),
            &Drafted {
                gather: drafted(),
                burn: drafted_burn(),
                fuel: drafted_fuel(),
                hydrate: drafted_hydrate(),
            },
            "2026-10-10",
        );

        assert_eq!(
            imported,
            Ok(Imported {
                measurements: 2,
                profiles: 1,
                goals: 1,
                exercises: 1,
                workouts: 1,
                foods: 1,
                intakes: 1,
                drinks: 1,
                drinkings: 1,
            })
        );
        assert_eq!(
            fuel.fuel_records().map(|fuelled| fuelled.foods),
            Ok(drafted_fuel().foods)
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
                &crate::testing::FuelRecordsInMemory::default(),
                &crate::testing::HydrateRecordsInMemory::default(),
                &crate::testing::SwitchOverInMemory::default(),
                &Drafted::default(),
                "2026-10-10",
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn importing_the_draft_makes_its_day_the_switch_over() {
        let switch_over = crate::testing::SwitchOverInMemory::default();

        import_drafted(
            &GatherRecordsInMemory::default(),
            &crate::testing::BurnRecordsInMemory::default(),
            &crate::testing::FuelRecordsInMemory::default(),
            &crate::testing::HydrateRecordsInMemory::default(),
            &switch_over,
            &Drafted::default(),
            "2026-10-10",
        )
        .expect("an import");

        assert_eq!(switch_over.switch_over(), Ok(Some(day("2026-10-10"))));
    }

    #[test]
    fn an_import_on_a_day_the_calendar_lacks_or_without_a_switch_over_kept_is_refused() {
        let import = |switch_over: &dyn Fn() -> Result<Imported, Failure>| switch_over();

        assert_eq!(
            import(&|| import_drafted(
                &GatherRecordsInMemory::default(),
                &crate::testing::BurnRecordsInMemory::default(),
                &crate::testing::FuelRecordsInMemory::default(),
                &crate::testing::HydrateRecordsInMemory::default(),
                &crate::testing::SwitchOverInMemory::default(),
                &Drafted::default(),
                "2026-02-30",
            )),
            Err(Failure::Refused(crate::refusal::Refusal::NotADate))
        );
        assert_eq!(
            import(&|| import_drafted(
                &GatherRecordsInMemory::default(),
                &crate::testing::BurnRecordsInMemory::default(),
                &crate::testing::FuelRecordsInMemory::default(),
                &crate::testing::HydrateRecordsInMemory::default(),
                &crate::testing::BrokenSwitchOver,
                &Drafted::default(),
                "2026-10-10",
            )),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    fn drafted_drinks() -> HydrateRecords {
        let expresso = |date| crate::drinking::Drinking {
            day: day(date),
            drink: "Expresso".into(),
            volume_ml: 80.0,
            kind: crate::drink::DrinkKind::Other,
            servings: 1.0,
        };
        HydrateRecords {
            drinks: vec![
                crate::drink::Drink {
                    name: "Expresso".into(),
                    volume_ml: 80.0,
                    kind: crate::drink::DrinkKind::Other,
                },
                crate::drink::Drink {
                    name: "Tea".into(),
                    volume_ml: 200.0,
                    kind: crate::drink::DrinkKind::Other,
                },
            ],
            drinkings: vec![(1, expresso("2026-08-02")), (2, expresso("2026-08-03"))],
            bases: vec![],
            counted_from: vec![],
        }
    }

    #[test]
    fn importing_hydrate_records_replaces_what_was_drunk() {
        let hydrate = crate::testing::HydrateRecordsInMemory::default();
        hydrate
            .save_base(day("2026-03-01"), 2400.0)
            .expect("a base saves");

        import_hydrate(&hydrate, &drafted_drinks()).expect("an import");

        assert_eq!(hydrate.hydrate_records(), Ok(drafted_drinks()));
    }

    #[test]
    fn a_hydrate_import_counts_its_drinks_and_its_drinkings() {
        assert_eq!(
            import_hydrate(
                &crate::testing::HydrateRecordsInMemory::default(),
                &drafted_drinks()
            ),
            Ok((2, 2))
        );
    }

    #[test]
    fn hydrate_records_that_cannot_be_replaced_give_no_import() {
        assert_eq!(
            import_hydrate(&crate::testing::BrokenHydrateRecords, &drafted_drinks()),
            Err(Failure::Broken("the disk is full".into()))
        );
    }
}
