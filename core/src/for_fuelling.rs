use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::fuel::{Fuel, fuel_over};
use crate::intake::intake;
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntakeForm {
    pub date: String,
    pub food: String,
    pub quantity: f64,
}

fn day(date: &str) -> Result<NaiveDate, Failure> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Failure::Refused(Refusal::NotADate))
}

fn fuel_of_the_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    day: NaiveDate,
) -> Result<Fuel, Failure> {
    let fuelled = fuel_over(
        &[day],
        &gather.gather_records()?,
        &burn.burn_records()?,
        &fuel.fuel_records()?,
    );
    Ok(fuelled
        .into_iter()
        .next()
        .unwrap_or_else(|| unreachable!("one day asked, one day fuelled")))
}

pub fn fuel_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    date: &str,
) -> Result<Fuel, Failure> {
    fuel_of_the_day(gather, burn, fuel, day(date)?)
}

pub fn log_intake(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    form: &IntakeForm,
) -> Result<Fuel, Failure> {
    let food = fuel
        .fuel_records()?
        .foods
        .into_iter()
        .find(|known| known.name == form.food)
        .ok_or(Failure::Refused(Refusal::NotAFood))?;
    let eaten = intake(&form.date, &food, form.quantity).map_err(Failure::Refused)?;
    fuel.save_intake(&eaten)?;
    fuel_of_the_day(gather, burn, fuel, eaten.day)
}

pub fn remove_intake(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    date: &str,
    id: i64,
) -> Result<Fuel, Failure> {
    let day = day(date)?;
    fuel.remove_intake(id)?;
    fuel_of_the_day(gather, burn, fuel, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::food::{Food, Nutrients, Unit};
    use crate::testing::{
        BrokenFuelRecords, BurnRecordsInMemory, FuelRecordsInMemory, GatherRecordsInMemory,
    };

    fn a_catalog_with_skyr() -> FuelRecordsInMemory {
        let fuel = FuelRecordsInMemory::default();
        fuel.save_food(&Food {
            name: "Skyr".into(),
            unit: Unit::Gram,
            portion: 100.0,
            nutrients: Nutrients {
                energy_kcal: 54.0,
                protein_g: 9.5,
                carbs_g: 4.0,
                fat_g: 0.0,
                fibre_g: 0.0,
            },
        })
        .expect("a food saves");
        fuel
    }

    fn skyr(date: &str, grams: f64) -> IntakeForm {
        IntakeForm {
            date: date.into(),
            food: "Skyr".into(),
            quantity: grams,
        }
    }

    #[test]
    fn a_logged_intake_comes_back_in_its_days_fuel() {
        let fuel = a_catalog_with_skyr();
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
        );

        let fuelled =
            log_intake(&gather, &burn, &fuel, &skyr("2026-10-10", 200.0)).expect("a fuel");

        assert_eq!(fuelled.intakes.len(), 1);
        assert!((fuelled.eaten.energy_kcal - 108.0).abs() < 1e-9);
        assert_eq!(fuel_day(&gather, &burn, &fuel, "2026-10-10"), Ok(fuelled));
    }

    #[test]
    fn an_intake_of_a_food_missing_from_the_catalog_is_refused() {
        let refused = log_intake(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &FuelRecordsInMemory::default(),
            &skyr("2026-10-10", 200.0),
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotAFood)));
    }

    #[test]
    fn removing_an_intake_leaves_the_others_of_its_day() {
        let fuel = a_catalog_with_skyr();
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
        );
        log_intake(&gather, &burn, &fuel, &skyr("2026-10-10", 200.0)).expect("an intake logs");
        let logged = log_intake(&gather, &burn, &fuel, &skyr("2026-10-10", 100.0)).expect("a fuel");
        let first = logged
            .intakes
            .first()
            .map(|(id, _)| *id)
            .expect("a first intake");

        let fuelled = remove_intake(&gather, &burn, &fuel, "2026-10-10", first).expect("a fuel");

        assert_eq!(
            fuelled
                .intakes
                .iter()
                .map(|(_, eaten)| eaten.quantity)
                .collect::<Vec<_>>(),
            vec![100.0]
        );
    }

    #[test]
    fn a_day_the_calendar_lacks_or_broken_records_give_no_fuel() {
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
        );

        assert_eq!(
            fuel_day(
                &gather,
                &burn,
                &FuelRecordsInMemory::default(),
                "2026-02-30"
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            remove_intake(&gather, &burn, &BrokenFuelRecords, "2026-10-10", 1),
            Err(Failure::Broken("the disk is full".into()))
        );
    }
}
