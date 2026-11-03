use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::food::Nutrients;
use crate::for_cataloguing::IngredientForm;
use crate::for_reviewing_progress::{Window, first_day_of};
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::fuel::{Fuel, fuel_on, fuel_over, fuel_with};
use crate::fuel_week::{FuelWeek, fuel_week_of};
use crate::intake::intake;
use crate::refusal::Refusal;
use crate::week::logged_between;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntakeForm {
    pub date: String,
    pub food: String,
    pub quantity: Option<f64>,
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
    Ok(fuel_on(
        day,
        &gather.gather_records()?,
        &burn.burn_records()?,
        &fuel.fuel_records()?,
    ))
}

pub fn fuel_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    date: &str,
) -> Result<Fuel, Failure> {
    fuel_of_the_day(gather, burn, fuel, day(date)?)
}

/// The day as it would stand with more eaten, nothing logged: its band lifted by their digestion, its macros
/// weighed with them.
pub fn fuel_day_with(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    date: &str,
    more: &Nutrients,
) -> Result<Fuel, Failure> {
    let day = day(date)?;
    Ok(fuel_with(
        day,
        &gather.gather_records()?,
        &burn.burn_records()?,
        &fuel.fuel_records()?,
        *more,
    ))
}

/// The average over the logged days of a window up to today, today left out while under way.
pub fn fuel_over_the_window(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    today: &str,
    window: Window,
) -> Result<Option<FuelWeek>, Failure> {
    let today = day(today)?;
    let fuelled = fuel.fuel_records()?;
    let logged = logged_between(
        first_day_of(window, today),
        today,
        today,
        fuelled.intakes.iter().map(|(_, eaten)| eaten.day),
    );
    Ok(fuel_week_of(&fuel_over(
        &logged,
        &gather.gather_records()?,
        &burn.burn_records()?,
        &fuelled,
    )))
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
    let eaten = intake(&form.date, &food, form.quantity.unwrap_or(food.portion))
        .map_err(Failure::Refused)?;
    fuel.save_intake(&eaten)?;
    fuel_of_the_day(gather, burn, fuel, eaten.day)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlateForm {
    pub date: String,
    pub ingredients: Vec<IngredientForm>,
}

/// Each ingredient of the plate as an intake of its day, by its raw quantity; none when one of them
/// cannot be.
pub fn log_plate(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    form: &PlateForm,
) -> Result<Fuel, Failure> {
    let foods = fuel.fuel_records()?.foods;
    let eaten = form
        .ingredients
        .iter()
        .map(|ingredient| {
            let food = foods
                .iter()
                .find(|known| known.name == ingredient.food)
                .ok_or(Failure::Refused(Refusal::NotAFood))?;
            intake(&form.date, food, ingredient.quantity).map_err(Failure::Refused)
        })
        .collect::<Result<Vec<_>, _>>()?;
    for each in &eaten {
        fuel.save_intake(each)?;
    }
    fuel_of_the_day(gather, burn, fuel, day(&form.date)?)
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
    use crate::food::{Food, Unit};
    use crate::gender::Gender;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::testing::{
        BrokenFuelRecords, BurnRecordsInMemory, FuelRecordsInMemory, GatherRecordsInMemory,
    };

    fn a_man_with_a_waist() -> GatherRecordsInMemory {
        let gather = GatherRecordsInMemory::default();
        gather
            .save_profile(
                day("2026-03-01").expect("a day"),
                &Profile {
                    birthday: day("1985-04-12").expect("a day"),
                    gender: Gender::Male,
                    height_cm: 180.0,
                    neck_cm: 38.5,
                    hip_cm: None,
                },
            )
            .expect("a profile saves");
        gather
            .save_measurement(&Measurement {
                day: day("2026-10-01").expect("a day"),
                body_mass_kg: 80.0,
                waist_cm: Some(82.0),
            })
            .expect("a measurement saves");
        gather
    }

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
            description: String::new(),
        })
        .expect("a food saves");
        fuel
    }

    fn skyr(date: &str, grams: f64) -> IntakeForm {
        IntakeForm {
            date: date.into(),
            food: "Skyr".into(),
            quantity: Some(grams),
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
    fn the_average_over_a_window_runs_over_its_logged_days_up_to_today_today_left_out() {
        let fuel = a_catalog_with_skyr();
        let (gather, burn) = (a_man_with_a_waist(), BurnRecordsInMemory::default());
        for date in ["2026-08-20", "2026-10-05", "2026-10-10"] {
            log_intake(&gather, &burn, &fuel, &skyr(date, 200.0)).expect("an intake");
        }
        let logged_over = |window| {
            fuel_over_the_window(&gather, &burn, &fuel, "2026-10-10", window)
                .ok()
                .flatten()
                .map(|over| over.logged_days)
        };

        assert_eq!(
            [
                logged_over(Window::Days7),
                logged_over(Window::Days30),
                logged_over(Window::Days90),
                logged_over(Window::Everything)
            ],
            [Some(1), Some(1), Some(2), Some(2)]
        );
    }

    #[test]
    fn the_day_previewed_with_more_eaten_stands_as_it_will_once_they_are_logged() {
        let fuel = a_catalog_with_skyr();
        let (gather, burn) = (a_man_with_a_waist(), BurnRecordsInMemory::default());
        log_intake(&gather, &burn, &fuel, &skyr("2026-10-10", 100.0)).expect("a first intake");
        let two_hundred_grams_of_skyr = Nutrients {
            energy_kcal: 108.0,
            protein_g: 19.0,
            carbs_g: 8.0,
            fat_g: 0.0,
            fibre_g: 0.0,
        };

        let previewed = fuel_day_with(
            &gather,
            &burn,
            &fuel,
            "2026-10-10",
            &two_hundred_grams_of_skyr,
        )
        .expect("a preview");
        let logged =
            log_intake(&gather, &burn, &fuel, &skyr("2026-10-10", 200.0)).expect("a second intake");

        assert_eq!(
            (
                previewed.eaten,
                previewed.band,
                previewed.energy,
                previewed.macros,
                previewed.budget
            ),
            (
                logged.eaten,
                logged.band,
                logged.energy,
                logged.macros,
                logged.budget
            )
        );
        assert_eq!(previewed.intakes.len(), 1);
    }

    #[test]
    fn a_window_up_to_a_today_the_calendar_does_not_have_is_refused() {
        assert_eq!(
            fuel_over_the_window(
                &GatherRecordsInMemory::default(),
                &BurnRecordsInMemory::default(),
                &FuelRecordsInMemory::default(),
                "2026-02-30",
                Window::Days7
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
    }

    #[test]
    fn a_day_whose_fuel_targets_were_never_kept_floors_its_macros_on_the_recommendation() {
        let fuelled = fuel_day(
            &a_man_with_a_waist(),
            &BurnRecordsInMemory::default(),
            &FuelRecordsInMemory::default(),
            "2026-10-10",
        )
        .expect("a fuel");

        let floors = fuelled
            .macros
            .map(|macros| (macros.fat.floor_g, macros.protein.floor_g));
        let lean_kg = crate::composition::body_composition(
            80.0,
            180.0,
            crate::composition::Tapes::Male {
                neck_cm: 38.5,
                waist_cm: 82.0,
            },
        )
        .expect("a composition")
        .lean_mass_kg;
        assert!(
            floors.is_some_and(|(fat_g, protein_g)| {
                (fat_g - 0.8 * 80.0).abs() < 1e-9 && (protein_g - 2.0 * lean_kg).abs() < 1e-9
            }),
            "{floors:?}"
        );
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

    fn a_plate(date: &str, ingredients: &[(&str, f64)]) -> PlateForm {
        PlateForm {
            date: date.into(),
            ingredients: ingredients
                .iter()
                .map(|(food, quantity)| IngredientForm {
                    food: (*food).into(),
                    quantity: *quantity,
                    cooked_per_raw: None,
                })
                .collect(),
        }
    }

    #[test]
    fn a_plate_logs_one_intake_per_ingredient_on_its_day_by_its_raw_quantity() {
        let fuel = a_catalog_with_skyr();
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
        );

        let fuelled = log_plate(
            &gather,
            &burn,
            &fuel,
            &a_plate("2026-10-10", &[("Skyr", 200.0), ("Skyr", 50.0)]),
        )
        .expect("a fuel");

        assert_eq!(
            fuelled
                .intakes
                .iter()
                .map(|(_, eaten)| (eaten.food.as_str(), eaten.quantity))
                .collect::<Vec<_>>(),
            vec![("Skyr", 200.0), ("Skyr", 50.0)]
        );
    }

    #[test]
    fn a_plate_with_a_food_missing_from_the_catalog_logs_none_of_its_ingredients() {
        let fuel = a_catalog_with_skyr();
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
        );

        let refused = log_plate(
            &gather,
            &burn,
            &fuel,
            &a_plate("2026-10-10", &[("Skyr", 200.0), ("Caviar", 30.0)]),
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotAFood)));
        assert_eq!(
            fuel_day(&gather, &burn, &fuel, "2026-10-10").map(|fuelled| fuelled.intakes.len()),
            Ok(0)
        );
    }

    #[test]
    fn an_intake_without_a_quantity_is_one_portion_of_its_food() {
        let fuel = a_catalog_with_skyr();
        let unweighed = IntakeForm {
            quantity: None,
            ..skyr("2026-10-10", 0.0)
        };

        let fuelled = log_intake(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &fuel,
            &unweighed,
        )
        .expect("a fuel");

        assert_eq!(
            fuelled
                .intakes
                .iter()
                .map(|(_, eaten)| eaten.quantity)
                .collect::<Vec<_>>(),
            vec![100.0]
        );
    }
}
