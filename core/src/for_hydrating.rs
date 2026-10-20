use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::drinking::drinking;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::hydrate::{Hydrate, hydrate_on, hydrate_over};
use crate::hydrate_week::{HydrateWeek, hydrate_week_of};
use crate::refusal::Refusal;
use crate::week::logged_in_the_week;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrinkingForm {
    pub date: String,
    pub drink: String,
    pub volume_ml: Option<f64>,
}

fn day(date: &str) -> Result<NaiveDate, Failure> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Failure::Refused(Refusal::NotADate))
}

fn hydrate_of_the_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    day: NaiveDate,
) -> Result<Hydrate, Failure> {
    Ok(hydrate_on(
        day,
        &hydrate.hydrate_records()?,
        &gather.gather_records()?,
        &burn.burn_records()?,
        &fuel.fuel_records()?,
    ))
}

pub fn hydrate_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    date: &str,
) -> Result<Hydrate, Failure> {
    hydrate_of_the_day(gather, burn, fuel, hydrate, day(date)?)
}

/// The week's average up to a day: its logged days of the seven, today left out while under way.
pub fn hydrate_week(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    date: &str,
    today: &str,
) -> Result<Option<HydrateWeek>, Failure> {
    let (last, today) = (day(date)?, day(today)?);
    let hydrated = hydrate.hydrate_records()?;
    let logged = logged_in_the_week(
        last,
        today,
        hydrated.drinkings.iter().map(|(_, drunk)| drunk.day),
    );
    Ok(hydrate_week_of(&hydrate_over(
        &logged,
        &hydrated,
        &gather.gather_records()?,
        &burn.burn_records()?,
        &fuel.fuel_records()?,
    )))
}

pub fn log_drink(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    form: &DrinkingForm,
) -> Result<Hydrate, Failure> {
    let drink = hydrate
        .hydrate_records()?
        .drinks
        .into_iter()
        .find(|known| known.name == form.drink)
        .ok_or(Failure::Refused(Refusal::NotADrink))?;
    let drunk = drinking(
        &form.date,
        &drink,
        form.volume_ml.unwrap_or(drink.volume_ml),
    )
    .map_err(Failure::Refused)?;
    hydrate.save_drinking(&drunk)?;
    hydrate_of_the_day(gather, burn, fuel, hydrate, drunk.day)
}

pub fn remove_drinking(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    date: &str,
    id: i64,
) -> Result<Hydrate, Failure> {
    let day = day(date)?;
    hydrate.remove_drinking(id)?;
    hydrate_of_the_day(gather, burn, fuel, hydrate, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drink::Drink;
    use crate::exercise::Exercise;
    use crate::for_burning::{WorkoutForm, log_workout};
    use crate::for_tracking::log_measurement;
    use crate::refusal::Refusal;
    use crate::testing::{
        BrokenBurnRecords, BrokenFuelRecords, BrokenGatherRecords, BrokenHydrateRecords,
        BurnRecordsInMemory, FuelRecordsInMemory, GatherRecordsInMemory, HydrateRecordsInMemory,
    };

    fn a_catalog_with_tea() -> HydrateRecordsInMemory {
        let hydrate = HydrateRecordsInMemory::default();
        hydrate
            .save_drink(&Drink {
                name: "Tea".into(),
                volume_ml: 200.0,
                kind: crate::drink::DrinkKind::Other,
                description: String::new(),
            })
            .expect("a drink saves");
        hydrate
    }

    fn tea(date: &str, volume_ml: f64) -> DrinkingForm {
        DrinkingForm {
            date: date.into(),
            drink: "Tea".into(),
            volume_ml: Some(volume_ml),
        }
    }

    fn volumes(hydrate: &Hydrate) -> Vec<f64> {
        hydrate
            .drinkings
            .iter()
            .map(|(_, drunk)| drunk.volume_ml)
            .collect()
    }

    #[test]
    fn a_logged_drink_comes_back_in_its_days_hydrate_at_its_volume() {
        let hydrate = a_catalog_with_tea();

        let hydrated = log_drink(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &FuelRecordsInMemory::default(),
            &hydrate,
            &tea("2026-10-10", 300.0),
        )
        .expect("a hydrate");

        assert_eq!(volumes(&hydrated), vec![300.0]);
        assert_eq!(hydrated.drunk_ml, 300.0);
    }

    #[test]
    fn a_weeks_average_runs_over_the_logged_days_of_the_seven_up_to_the_day_today_left_out_until_it_is_over()
     {
        let (gather, burn, fuel, hydrate) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
            FuelRecordsInMemory::default(),
            a_catalog_with_tea(),
        );
        for date in ["2026-10-02", "2026-10-04", "2026-10-06", "2026-10-10"] {
            log_drink(&gather, &burn, &fuel, &hydrate, &tea(date, 400.0)).expect("a drink logs");
        }

        let under_way = hydrate_week(&gather, &burn, &fuel, &hydrate, "2026-10-10", "2026-10-10");
        let over = hydrate_week(&gather, &burn, &fuel, &hydrate, "2026-10-10", "2026-10-11");

        assert_eq!(
            [under_way, over].map(|week| {
                week.ok()
                    .flatten()
                    .map(|week| (week.logged_days, week.drunk.per_day))
            }),
            [Some((2, 400.0)), Some((3, 400.0))]
        );
    }

    #[test]
    fn a_week_up_to_a_day_the_calendar_does_not_have_is_refused() {
        let (gather, burn, fuel, hydrate) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
            FuelRecordsInMemory::default(),
            HydrateRecordsInMemory::default(),
        );

        assert_eq!(
            [
                hydrate_week(&gather, &burn, &fuel, &hydrate, "2026-02-30", "2026-10-10"),
                hydrate_week(&gather, &burn, &fuel, &hydrate, "2026-10-10", "2026-02-30")
            ],
            [
                Err(Failure::Refused(Refusal::NotADate)),
                Err(Failure::Refused(Refusal::NotADate))
            ]
        );
    }

    #[test]
    fn a_drink_missing_from_the_catalog_is_refused() {
        let refused = log_drink(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &FuelRecordsInMemory::default(),
            &HydrateRecordsInMemory::default(),
            &tea("2026-10-10", 200.0),
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotADrink)));
    }

    #[test]
    fn removing_a_drinking_leaves_the_others_of_its_day() {
        let gather = GatherRecordsInMemory::default();
        let burn = BurnRecordsInMemory::default();
        let hydrate = a_catalog_with_tea();
        log_drink(
            &gather,
            &burn,
            &FuelRecordsInMemory::default(),
            &hydrate,
            &tea("2026-10-10", 200.0),
        )
        .expect("a drink logs");
        let logged = log_drink(
            &gather,
            &burn,
            &FuelRecordsInMemory::default(),
            &hydrate,
            &tea("2026-10-10", 100.0),
        )
        .expect("a hydrate");
        let first = logged
            .drinkings
            .first()
            .map(|(id, _)| *id)
            .expect("a first drinking");

        let hydrated = remove_drinking(
            &gather,
            &burn,
            &FuelRecordsInMemory::default(),
            &hydrate,
            "2026-10-10",
            first,
        )
        .expect("a hydrate");

        assert_eq!(volumes(&hydrated), vec![100.0]);
    }

    #[test]
    fn a_days_hydrate_reads_back_what_was_drunk_on_it() {
        let gather = GatherRecordsInMemory::default();
        let burn = BurnRecordsInMemory::default();
        let hydrate = a_catalog_with_tea();
        log_drink(
            &gather,
            &burn,
            &FuelRecordsInMemory::default(),
            &hydrate,
            &tea("2026-10-10", 400.0),
        )
        .expect("a drink logs");

        let hydrated = hydrate_day(
            &gather,
            &burn,
            &FuelRecordsInMemory::default(),
            &hydrate,
            "2026-10-10",
        )
        .expect("a hydrate");

        assert_eq!(volumes(&hydrated), vec![400.0]);
    }

    #[test]
    fn a_days_target_grows_by_a_millilitre_per_kcal_its_workouts_burnt() {
        let gather = GatherRecordsInMemory::default();
        log_measurement(&gather, "2026-10-10", 80.0, None).expect("a measurement logs");
        let burn = BurnRecordsInMemory::default();
        burn.save_exercise(&Exercise {
            name: "Walking".into(),
            met: 4.0,
            description: String::new(),
        })
        .expect("an exercise saves");
        log_workout(
            &gather,
            &burn,
            &WorkoutForm {
                date: "2026-10-10".into(),
                exercise: "Walking".into(),
                minutes: Some(60.0),
                reps: None,
                sets: Some(1),
                load_kg: Some(0.0),
            },
        )
        .expect("a workout logs");

        let hydrated = hydrate_day(
            &gather,
            &burn,
            &FuelRecordsInMemory::default(),
            &HydrateRecordsInMemory::default(),
            "2026-10-10",
        )
        .expect("a hydrate");

        assert!((hydrated.target_ml - (2000.0 + 252.0)).abs() < 1e-9);
    }

    #[test]
    fn a_day_the_calendar_lacks_a_drinking_of_no_volume_or_broken_records_give_no_hydrate() {
        let gather = GatherRecordsInMemory::default();
        let burn = BurnRecordsInMemory::default();
        let broken = Failure::Broken("the disk is full".into());

        assert_eq!(
            hydrate_day(
                &gather,
                &burn,
                &FuelRecordsInMemory::default(),
                &a_catalog_with_tea(),
                "2026-02-30"
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            log_drink(
                &gather,
                &burn,
                &FuelRecordsInMemory::default(),
                &a_catalog_with_tea(),
                &tea("2026-10-10", 0.0)
            ),
            Err(Failure::Refused(Refusal::NotAVolume))
        );
        assert_eq!(
            hydrate_day(
                &gather,
                &burn,
                &FuelRecordsInMemory::default(),
                &BrokenHydrateRecords,
                "2026-10-10"
            ),
            Err(broken.clone())
        );
        assert_eq!(
            hydrate_day(
                &BrokenGatherRecords,
                &burn,
                &FuelRecordsInMemory::default(),
                &a_catalog_with_tea(),
                "2026-10-10"
            ),
            Err(broken.clone())
        );
        assert_eq!(
            hydrate_day(
                &gather,
                &BrokenBurnRecords,
                &FuelRecordsInMemory::default(),
                &a_catalog_with_tea(),
                "2026-10-10"
            ),
            Err(broken.clone())
        );
        assert_eq!(
            hydrate_day(
                &gather,
                &burn,
                &BrokenFuelRecords,
                &a_catalog_with_tea(),
                "2026-10-10"
            ),
            Err(broken.clone())
        );
        assert_eq!(
            remove_drinking(
                &gather,
                &burn,
                &FuelRecordsInMemory::default(),
                &BrokenHydrateRecords,
                "2026-10-10",
                1
            ),
            Err(broken)
        );
    }

    #[test]
    fn a_drink_logged_without_a_volume_is_one_serving_of_it() {
        let unmeasured = DrinkingForm {
            volume_ml: None,
            ..tea("2026-10-10", 0.0)
        };

        let hydrated = log_drink(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &FuelRecordsInMemory::default(),
            &a_catalog_with_tea(),
            &unmeasured,
        )
        .expect("a hydrate");

        assert_eq!(volumes(&hydrated), vec![200.0]);
    }
}
