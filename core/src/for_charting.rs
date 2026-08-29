use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn::{Burn, burn_on};
use crate::failure::Failure;
use crate::food::Nutrients;
use crate::for_reviewing_progress::{Window, first_day_of};
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::fuel::fuel_over;
use crate::hydrate::{Hydrate, hydrate_on};
use crate::ledger::{Day, ledger};
use crate::refusal::Refusal;
use crate::version::version_in_force_on;
use crate::workout::Workout;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GatherChart {
    pub masses: Vec<(NaiveDate, f64)>,
    pub lean_masses: Vec<(NaiveDate, f64)>,
    pub goal_kg: Option<f64>,
    pub goal_lean_kg: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BandChart {
    pub amounts: Vec<(NaiveDate, f64)>,
    pub floors: Vec<(NaiveDate, f64)>,
    pub caps: Vec<(NaiveDate, f64)>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BurnChart {
    pub burnt: Vec<(NaiveDate, f64)>,
    pub reps: Vec<(NaiveDate, f64)>,
    pub minutes: Vec<(NaiveDate, f64)>,
}

fn span_of(today: &str, window: Window) -> Result<(NaiveDate, NaiveDate), Failure> {
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    Ok((first_day_of(window, today), today))
}

fn days_within(
    logged: impl Iterator<Item = NaiveDate>,
    (first, last): (NaiveDate, NaiveDate),
) -> Vec<NaiveDate> {
    let mut days: Vec<NaiveDate> = logged.filter(|day| (first..=last).contains(day)).collect();
    days.sort();
    days.dedup();
    days
}

pub fn gather_chart(
    gather: &impl ForStoringGatherRecords,
    today: &str,
    window: Window,
) -> Result<GatherChart, Failure> {
    let (first, last) = span_of(today, window)?;
    let gathered = gather.gather_records()?;
    let days: Vec<Day> = ledger(&gathered.measurements, &gathered.profiles)
        .into_iter()
        .filter(|measured| (first..=last).contains(&measured.day))
        .collect();
    let goals = version_in_force_on(&gathered.goals, last);
    Ok(GatherChart {
        masses: days
            .iter()
            .map(|measured| (measured.day, measured.body_mass_kg))
            .collect(),
        lean_masses: days
            .iter()
            .filter_map(|measured| {
                measured
                    .composition
                    .map(|split| (measured.day, split.lean_mass_kg))
            })
            .collect(),
        goal_kg: goals.map(|goals| goals.body_mass_kg),
        goal_lean_kg: goals.map(|goals| goals.body_mass_kg * (1.0 - goals.fat_ratio)),
    })
}

pub fn fuel_chart(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    today: &str,
    window: Window,
) -> Result<BandChart, Failure> {
    let span = span_of(today, window)?;
    let (gathered, burnt, fuelled) = (
        gather.gather_records()?,
        burn.burn_records()?,
        fuel.fuel_records()?,
    );
    let days = days_within(fuelled.intakes.iter().map(|(_, eaten)| eaten.day), span);
    let fuelled_days = fuel_over(&days, &gathered, &burnt, &fuelled);
    Ok(BandChart {
        amounts: fuelled_days
            .iter()
            .map(|fuelled| (fuelled.day, fuelled.eaten.energy_kcal))
            .collect(),
        floors: fuelled_days
            .iter()
            .filter_map(|fuelled| fuelled.band.map(|band| (fuelled.day, band.floor_kcal)))
            .collect(),
        caps: fuelled_days
            .iter()
            .filter_map(|fuelled| fuelled.band.map(|band| (fuelled.day, band.cap_kcal)))
            .collect(),
    })
}

pub fn hydrate_chart(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    hydrate: &impl ForStoringHydrateRecords,
    today: &str,
    window: Window,
) -> Result<BandChart, Failure> {
    let span = span_of(today, window)?;
    let (gathered, burnt, hydrated) = (
        gather.gather_records()?,
        burn.burn_records()?,
        hydrate.hydrate_records()?,
    );
    let days = days_within(hydrated.drinkings.iter().map(|(_, drunk)| drunk.day), span);
    let hydrated_days: Vec<Hydrate> = days
        .iter()
        .map(|day| hydrate_on(*day, &hydrated, &gathered, &burnt))
        .collect();
    Ok(BandChart {
        amounts: hydrated_days
            .iter()
            .map(|of_the_day| (of_the_day.day, of_the_day.drunk_ml))
            .collect(),
        floors: hydrated_days
            .iter()
            .map(|of_the_day| (of_the_day.day, of_the_day.floor_ml))
            .collect(),
        caps: hydrated_days
            .iter()
            .map(|of_the_day| (of_the_day.day, of_the_day.cap_ml))
            .collect(),
    })
}

pub fn burn_chart(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    today: &str,
    window: Window,
) -> Result<BurnChart, Failure> {
    let span = span_of(today, window)?;
    let (gathered, burnt) = (gather.gather_records()?, burn.burn_records()?);
    let days = days_within(burnt.workouts.iter().map(|(_, done)| done.day), span);
    let burns: Vec<Burn> = days
        .iter()
        .map(|day| burn_on(*day, &burnt, &gathered.measurements))
        .collect();
    let summed = |each: fn(&Workout) -> f64| -> Vec<(NaiveDate, f64)> {
        burns
            .iter()
            .map(|of_the_day| {
                (
                    of_the_day.day,
                    of_the_day
                        .workouts
                        .iter()
                        .map(|done| each(&done.workout))
                        .sum(),
                )
            })
            .collect()
    };
    Ok(BurnChart {
        burnt: burns
            .iter()
            .filter_map(|of_the_day| of_the_day.burnt_kcal.map(|kcal| (of_the_day.day, kcal)))
            .collect(),
        reps: summed(|done| f64::from(done.sets * done.reps.unwrap_or(0))),
        minutes: summed(|done| f64::from(done.sets) * done.minutes),
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Month<Row> {
    pub month: String,
    pub rows: Vec<Row>,
    pub earlier: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GatherRow {
    pub day: NaiveDate,
    pub body_mass_kg: f64,
    pub lean_mass_kg: Option<f64>,
    pub fat_mass_kg: Option<f64>,
    pub fat_ratio: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelRow {
    pub day: NaiveDate,
    pub eaten: Nutrients,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HydrateRow {
    pub day: NaiveDate,
    pub drunk_ml: f64,
    pub water_ratio: f64,
    pub coffee_cups: f64,
    pub tea_cups: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BurnRow {
    pub day: NaiveDate,
    pub burnt_kcal: Option<f64>,
    pub minutes: f64,
    pub reps: f64,
    pub top_exercise: Option<String>,
}

fn nothing_yet<Row>(month: &str) -> Result<Month<Row>, Failure> {
    Ok(Month {
        month: month.into(),
        rows: vec![],
        earlier: None,
    })
}

pub fn gather_month(
    gather: &impl ForStoringGatherRecords,
    month: &str,
) -> Result<Month<GatherRow>, Failure> {
    let _ = gather;
    nothing_yet(month)
}

pub fn fuel_month(
    fuel: &impl ForStoringFuelRecords,
    month: &str,
) -> Result<Month<FuelRow>, Failure> {
    let _ = fuel;
    nothing_yet(month)
}

pub fn hydrate_month(
    hydrate: &impl ForStoringHydrateRecords,
    month: &str,
) -> Result<Month<HydrateRow>, Failure> {
    let _ = hydrate;
    nothing_yet(month)
}

pub fn burn_month(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    month: &str,
) -> Result<Month<BurnRow>, Failure> {
    let _ = (gather, burn);
    nothing_yet(month)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drink::DrinkKind;
    use crate::food::Unit;
    use crate::for_burning::{WorkoutForm, log_workout};
    use crate::for_cataloguing::{FoodForm, add_drink, add_exercise, add_food};
    use crate::for_fuelling::{IntakeForm, log_intake};
    use crate::for_hydrating::{DrinkingForm, log_drink};
    use crate::for_setting_up_the_profile::{GoalsForm, ProfileForm, set_goals, set_profile};
    use crate::for_tracking::log_measurement;
    use crate::gender::Gender;
    use crate::testing::{
        BrokenBurnRecords, BurnRecordsInMemory, FuelRecordsInMemory, GatherRecordsInMemory,
        HydrateRecordsInMemory,
    };

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    struct Logged {
        gather: GatherRecordsInMemory,
        burn: BurnRecordsInMemory,
        fuel: FuelRecordsInMemory,
        hydrate: HydrateRecordsInMemory,
    }

    fn a_logged_man() -> Logged {
        let logged = Logged {
            gather: GatherRecordsInMemory::default(),
            burn: BurnRecordsInMemory::default(),
            fuel: FuelRecordsInMemory::default(),
            hydrate: HydrateRecordsInMemory::default(),
        };
        set_profile(
            &logged.gather,
            &ProfileForm {
                from: "2026-03-01".into(),
                birthday: "1985-04-12".into(),
                gender: Gender::Male,
                height_cm: 180.0,
                neck_cm: 38.5,
                hip_cm: None,
            },
        )
        .expect("a profile set");
        set_goals(
            &logged.gather,
            &GoalsForm {
                from: "2026-03-01".into(),
                body_mass_kg: 70.0,
                fat_percentage: 10.0,
            },
        )
        .expect("goals set");
        add_exercise(&logged.burn, "Push up", 3.8).expect("an exercise adds");
        add_food(
            &logged.fuel,
            &FoodForm {
                name: "Plate".into(),
                unit: Unit::Piece,
                portion: 1.0,
                nutrients: Nutrients {
                    energy_kcal: 2400.0,
                    protein_g: 170.0,
                    carbs_g: 230.0,
                    fat_g: 80.0,
                    fibre_g: 35.0,
                },
            },
        )
        .expect("a food adds");
        add_drink(&logged.hydrate, "Water", 500.0, DrinkKind::Water).expect("a drink adds");
        for (date, kg) in [
            ("2026-08-01", 82.0),
            ("2026-10-09", 80.2),
            ("2026-10-10", 80.0),
        ] {
            log_measurement(&logged.gather, date, kg, Some(80.0)).expect("a measurement logs");
        }
        for date in ["2026-08-01", "2026-10-09", "2026-10-10"] {
            log_intake(
                &logged.gather,
                &logged.burn,
                &logged.fuel,
                &IntakeForm {
                    date: date.into(),
                    food: "Plate".into(),
                    quantity: 1.0,
                },
            )
            .expect("an intake logs");
            log_drink(
                &logged.gather,
                &logged.burn,
                &logged.hydrate,
                &DrinkingForm {
                    date: date.into(),
                    drink: "Water".into(),
                    servings: 3.0,
                },
            )
            .expect("a drink logs");
            log_workout(
                &logged.gather,
                &logged.burn,
                &WorkoutForm {
                    date: date.into(),
                    exercise: "Push up".into(),
                    minutes: 0.5,
                    reps: Some(12),
                    sets: 3,
                    load_kg: 0.0,
                },
            )
            .expect("a workout logs");
        }
        logged
    }

    #[test]
    fn the_gather_chart_draws_the_masses_and_lean_masses_of_the_window_with_both_goals() {
        let logged = a_logged_man();

        let chart = gather_chart(&logged.gather, "2026-10-10", Window::Days30).expect("a chart");

        assert_eq!(
            chart.masses,
            vec![(day("2026-10-09"), 80.2), (day("2026-10-10"), 80.0)]
        );
        assert_eq!(chart.lean_masses.len(), 2);
        assert_eq!(chart.goal_kg, Some(70.0));
        assert!(
            chart
                .goal_lean_kg
                .is_some_and(|kg| (kg - 63.0).abs() < 1e-9)
        );
    }

    #[test]
    fn the_fuel_chart_draws_each_logged_days_energy_between_its_floor_and_cap() {
        let logged = a_logged_man();

        let chart = fuel_chart(
            &logged.gather,
            &logged.burn,
            &logged.fuel,
            "2026-10-10",
            Window::Days30,
        )
        .expect("a chart");

        assert_eq!(
            chart.amounts,
            vec![(day("2026-10-09"), 2400.0), (day("2026-10-10"), 2400.0)]
        );
        assert_eq!((chart.floors.len(), chart.caps.len()), (2, 2));
        assert!(
            chart
                .floors
                .iter()
                .zip(&chart.caps)
                .all(|((_, floor), (_, cap))| floor < cap)
        );
    }

    #[test]
    fn the_hydrate_chart_draws_each_days_drinks_ten_percent_either_side_of_its_target() {
        let logged = a_logged_man();

        let chart = hydrate_chart(
            &logged.gather,
            &logged.burn,
            &logged.hydrate,
            "2026-10-10",
            Window::Everything,
        )
        .expect("a chart");

        assert_eq!(chart.amounts.len(), 3);
        assert!(
            chart
                .amounts
                .iter()
                .all(|(_, ml)| (*ml - 1500.0).abs() < 1e-9)
        );
        assert!(
            chart
                .floors
                .iter()
                .zip(&chart.caps)
                .all(|((_, floor), (_, cap))| (cap / floor - 1.1 / 0.9).abs() < 1e-9)
        );
    }

    #[test]
    fn the_burn_chart_draws_each_days_burn_reps_and_minutes() {
        let logged = a_logged_man();

        let chart = burn_chart(&logged.gather, &logged.burn, "2026-10-10", Window::Days30)
            .expect("a chart");

        assert_eq!(
            chart.reps,
            vec![(day("2026-10-09"), 36.0), (day("2026-10-10"), 36.0)]
        );
        assert_eq!(
            chart.minutes,
            vec![(day("2026-10-09"), 1.5), (day("2026-10-10"), 1.5)]
        );
        assert_eq!(chart.burnt.len(), 2);
    }

    #[test]
    fn a_today_the_calendar_lacks_or_broken_records_chart_nothing() {
        let logged = a_logged_man();

        assert_eq!(
            gather_chart(&logged.gather, "2026-02-30", Window::Days30),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            burn_chart(
                &logged.gather,
                &BrokenBurnRecords,
                "2026-10-10",
                Window::Days30
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn a_month_of_gather_lists_its_days_newest_first_with_their_body_split_and_the_month_before() {
        let logged = a_logged_man();

        let october = gather_month(&logged.gather, "2026-10").expect("a month");

        assert_eq!(
            october.rows.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![day("2026-10-10"), day("2026-10-09")]
        );
        let latest = october.rows[0];
        assert!(
            latest
                .lean_mass_kg
                .zip(latest.fat_mass_kg)
                .is_some_and(|(lean, fat)| (lean + fat - 80.0).abs() < 1e-9)
        );
        assert!(latest.fat_ratio.is_some());
        assert_eq!(october.earlier, Some("2026-08".into()));
    }

    #[test]
    fn a_month_of_fuel_lists_what_each_day_ate() {
        let logged = a_logged_man();

        let august = fuel_month(&logged.fuel, "2026-08").expect("a month");

        assert_eq!(
            august
                .rows
                .iter()
                .map(|row| (row.day, row.eaten.protein_g))
                .collect::<Vec<_>>(),
            vec![(day("2026-08-01"), 170.0)]
        );
        assert_eq!(august.earlier, None);
    }

    #[test]
    fn a_month_of_hydrate_tells_each_days_volume_its_share_of_water_and_its_cups() {
        let logged = a_logged_man();
        add_drink(&logged.hydrate, "Expresso", 80.0, DrinkKind::Coffee).expect("a drink adds");
        log_drink(
            &logged.gather,
            &logged.burn,
            &logged.hydrate,
            &DrinkingForm {
                date: "2026-10-10".into(),
                drink: "Expresso".into(),
                servings: 2.0,
            },
        )
        .expect("a drink logs");

        let october = hydrate_month(&logged.hydrate, "2026-10").expect("a month");

        let today = october.rows[0];
        assert!((today.drunk_ml - 1660.0).abs() < 1e-9);
        assert!((today.water_ratio - 1500.0 / 1660.0).abs() < 1e-9);
        assert_eq!((today.coffee_cups, today.tea_cups), (2.0, 0.0));
    }

    #[test]
    fn a_month_of_burn_tells_each_days_burn_minutes_reps_and_its_top_exercise() {
        let logged = a_logged_man();

        let october = burn_month(&logged.gather, &logged.burn, "2026-10").expect("a month");

        let today = &october.rows[0];
        assert_eq!(
            (today.minutes, today.reps, today.top_exercise.as_deref()),
            (1.5, 36.0, Some("Push up"))
        );
        assert!(today.burnt_kcal.is_some_and(|kcal| kcal > 0.0));
    }

    #[test]
    fn a_month_the_calendar_lacks_is_refused() {
        assert_eq!(
            gather_month(&GatherRecordsInMemory::default(), "2026-13"),
            Err(Failure::Refused(Refusal::NotADate))
        );
    }
}
