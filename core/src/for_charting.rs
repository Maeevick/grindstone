use chrono::{Months, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::burn::{Burn, burn_on};
use crate::drink::DrinkKind;
use crate::drinking::Drinking;
use crate::failure::Failure;
use crate::food::{Nutrients, summed};
use crate::for_reviewing_progress::{Window, first_day_of};
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::fuel::{MacroRow, Macros, fuel_over};
use crate::hydrate::{Hydrate, hydrate_on};
use crate::ledger::{Day, ledger};
use crate::refusal::Refusal;
use crate::verdict::Verdict;
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
        minutes: summed(|done| f64::from(done.sets) * done.minutes),
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub low: usize,
    pub in_band: usize,
    pub high: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Insights {
    pub days: usize,
    pub weighed_days: usize,
    pub fuelled_days: usize,
    pub energy: Tally,
    pub protein: Tally,
    pub fat: Tally,
    pub fibre: Tally,
    pub carbs: Tally,
    pub hydrated_days: usize,
    pub hydrate: Tally,
    pub active_days: usize,
    pub burnt_per_active_day: Option<f64>,
}

pub fn insights(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    today: &str,
    window: Window,
) -> Result<Insights, Failure> {
    let (opening, last) = span_of(today, window)?;
    let (gathered, burnt, fuelled, hydrated) = (
        gather.gather_records()?,
        burn.burn_records()?,
        fuel.fuel_records()?,
        hydrate.hydrate_records()?,
    );
    let measured = gathered.measurements.iter().map(|measured| measured.day);
    let eaten = fuelled.intakes.iter().map(|(_, eaten)| eaten.day);
    let drunk = hydrated.drinkings.iter().map(|(_, drunk)| drunk.day);
    let worked_out = burnt.workouts.iter().map(|(_, done)| done.day);
    let first_record = measured
        .clone()
        .chain(eaten.clone())
        .chain(drunk.clone())
        .chain(worked_out.clone())
        .min()
        .unwrap_or(last);
    let span = (opening.max(first_record), last);
    let fuelled_days = fuel_over(&days_within(eaten, span), &gathered, &burnt, &fuelled);
    let hydrated_days: Vec<Hydrate> = days_within(drunk, span)
        .iter()
        .map(|day| hydrate_on(*day, &hydrated, &gathered, &burnt))
        .collect();
    let burns: Vec<f64> = days_within(worked_out, span)
        .iter()
        .filter_map(|day| burn_on(*day, &burnt, &gathered.measurements).burnt_kcal)
        .collect();
    let tally_of = |verdicts: Vec<Verdict>| Tally {
        low: verdicts
            .iter()
            .filter(|verdict| **verdict == Verdict::Low)
            .count(),
        in_band: verdicts
            .iter()
            .filter(|verdict| **verdict == Verdict::InBand)
            .count(),
        high: verdicts
            .iter()
            .filter(|verdict| **verdict == Verdict::High)
            .count(),
    };
    let macro_tally = |row: fn(&Macros) -> MacroRow| {
        tally_of(
            fuelled_days
                .iter()
                .filter_map(|fuelled| fuelled.macros.as_ref().map(|macros| row(macros).verdict))
                .collect(),
        )
    };
    Ok(Insights {
        days: ((span.1 - span.0).num_days() + 1).max(0) as usize,
        weighed_days: days_within(measured, span).len(),
        fuelled_days: fuelled_days.len(),
        energy: tally_of(
            fuelled_days
                .iter()
                .filter_map(|fuelled| fuelled.energy)
                .collect(),
        ),
        protein: macro_tally(|macros| macros.protein),
        fat: macro_tally(|macros| macros.fat),
        fibre: macro_tally(|macros| macros.fibre),
        carbs: macro_tally(|macros| macros.carbs),
        hydrated_days: hydrated_days.len(),
        hydrate: tally_of(
            hydrated_days
                .iter()
                .map(|of_the_day| of_the_day.verdict)
                .collect(),
        ),
        active_days: burns.len(),
        burnt_per_active_day: (!burns.is_empty())
            .then(|| burns.iter().sum::<f64>() / burns.len() as f64),
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BurnRow {
    pub day: NaiveDate,
    pub burnt_kcal: Option<f64>,
    pub minutes: f64,
    pub reps: f64,
    pub top_exercise: Option<String>,
}

fn month_of(month: &str) -> Result<(NaiveDate, NaiveDate), Failure> {
    let first = NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    let last = first
        .checked_add_months(Months::new(1))
        .and_then(|next| next.pred_opt())
        .unwrap_or(first);
    Ok((first, last))
}

fn month_in(
    month: &str,
    logged: impl Iterator<Item = NaiveDate> + Clone,
) -> Result<(Vec<NaiveDate>, Option<String>), Failure> {
    let span = month_of(month)?;
    let mut days = days_within(logged.clone(), span);
    days.reverse();
    let earlier = logged
        .filter(|day| *day < span.0)
        .max()
        .map(|day| day.format("%Y-%m").to_string());
    Ok((days, earlier))
}

pub fn gather_month(
    gather: &impl ForStoringGatherRecords,
    month: &str,
) -> Result<Month<GatherRow>, Failure> {
    let gathered = gather.gather_records()?;
    let ledger_days = ledger(&gathered.measurements, &gathered.profiles);
    let (days, earlier) = month_in(month, ledger_days.iter().map(|measured| measured.day))?;
    Ok(Month {
        month: month.into(),
        rows: days
            .iter()
            .filter_map(|day| ledger_days.iter().find(|measured| measured.day == *day))
            .map(|measured| GatherRow {
                day: measured.day,
                body_mass_kg: measured.body_mass_kg,
                lean_mass_kg: measured.composition.map(|split| split.lean_mass_kg),
                fat_mass_kg: measured.composition.map(|split| split.fat_mass_kg),
                fat_ratio: measured.composition.map(|split| split.fat_ratio),
            })
            .collect(),
        earlier,
    })
}

pub fn fuel_month(
    fuel: &impl ForStoringFuelRecords,
    month: &str,
) -> Result<Month<FuelRow>, Failure> {
    let fuelled = fuel.fuel_records()?;
    let (days, earlier) = month_in(month, fuelled.intakes.iter().map(|(_, eaten)| eaten.day))?;
    Ok(Month {
        month: month.into(),
        rows: days
            .iter()
            .map(|day| FuelRow {
                day: *day,
                eaten: summed(
                    fuelled
                        .intakes
                        .iter()
                        .filter(|(_, eaten)| eaten.day == *day)
                        .map(|(_, eaten)| eaten.nutrients),
                ),
            })
            .collect(),
        earlier,
    })
}

pub fn hydrate_month(
    hydrate: &impl ForStoringHydrateRecords,
    month: &str,
) -> Result<Month<HydrateRow>, Failure> {
    let hydrated = hydrate.hydrate_records()?;
    let (days, earlier) = month_in(month, hydrated.drinkings.iter().map(|(_, drunk)| drunk.day))?;
    Ok(Month {
        month: month.into(),
        rows: days
            .iter()
            .map(|day| {
                let drunk: Vec<&Drinking> = hydrated
                    .drinkings
                    .iter()
                    .filter(|(_, drunk)| drunk.day == *day)
                    .map(|(_, drunk)| drunk)
                    .collect();
                let volume_of = |kind: DrinkKind| -> f64 {
                    drunk
                        .iter()
                        .filter(|drunk| drunk.kind == kind)
                        .map(|drunk| drunk.volume_ml)
                        .sum()
                };
                let drunk_ml: f64 = drunk.iter().map(|drunk| drunk.volume_ml).sum();
                HydrateRow {
                    day: *day,
                    drunk_ml,
                    water_ratio: volume_of(DrinkKind::Water) / drunk_ml,
                }
            })
            .collect(),
        earlier,
    })
}

pub fn burn_month(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    month: &str,
) -> Result<Month<BurnRow>, Failure> {
    let (gathered, burnt) = (gather.gather_records()?, burn.burn_records()?);
    let (days, earlier) = month_in(month, burnt.workouts.iter().map(|(_, done)| done.day))?;
    Ok(Month {
        month: month.into(),
        rows: days
            .iter()
            .map(|day| {
                let of_the_day = burn_on(*day, &burnt, &gathered.measurements);
                let mut by_exercise: Vec<(String, f64)> = Vec::new();
                for done in &of_the_day.workouts {
                    let kcal = done.burnt_kcal.unwrap_or(0.0);
                    match by_exercise
                        .iter_mut()
                        .find(|(name, _)| *name == done.workout.exercise)
                    {
                        Some((_, sum)) => *sum += kcal,
                        None => by_exercise.push((done.workout.exercise.clone(), kcal)),
                    }
                }
                BurnRow {
                    day: *day,
                    burnt_kcal: of_the_day.burnt_kcal,
                    minutes: of_the_day
                        .workouts
                        .iter()
                        .map(|done| f64::from(done.workout.sets) * done.workout.minutes)
                        .sum(),
                    reps: of_the_day
                        .workouts
                        .iter()
                        .map(|done| f64::from(done.workout.sets * done.workout.reps.unwrap_or(0)))
                        .sum(),
                    top_exercise: by_exercise
                        .into_iter()
                        .max_by(|(_, one), (_, other)| one.total_cmp(other))
                        .map(|(name, _)| name),
                }
            })
            .collect(),
        earlier,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
                    quantity: Some(1.0),
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
                    volume_ml: Some(1500.0),
                },
            )
            .expect("a drink logs");
            log_workout(
                &logged.gather,
                &logged.burn,
                &WorkoutForm {
                    date: date.into(),
                    exercise: "Push up".into(),
                    minutes: Some(0.5),
                    reps: Some(12),
                    sets: Some(3),
                    load_kg: Some(0.0),
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
    fn the_hydrate_chart_draws_each_days_drinks_twenty_percent_either_side_of_its_target() {
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
                .all(|((_, floor), (_, cap))| (cap / floor - 1.2 / 0.8).abs() < 1e-9)
        );
    }

    #[test]
    fn the_burn_chart_draws_each_days_burn_and_minutes() {
        let logged = a_logged_man();

        let chart = burn_chart(&logged.gather, &logged.burn, "2026-10-10", Window::Days30)
            .expect("a chart");

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
            august.rows.iter().map(|row| row.day).collect::<Vec<_>>(),
            vec![day("2026-08-01")]
        );
        assert_eq!(
            august.rows[0].eaten,
            Nutrients {
                energy_kcal: 2400.0,
                protein_g: 170.0,
                carbs_g: 230.0,
                fat_g: 80.0,
                fibre_g: 35.0,
            }
        );
        assert_eq!(august.earlier, None);
    }

    #[test]
    fn a_month_of_hydrate_tells_each_days_volume_and_its_share_of_water() {
        let logged = a_logged_man();
        add_drink(&logged.hydrate, "Expresso", 80.0, DrinkKind::Other).expect("a drink adds");
        log_drink(
            &logged.gather,
            &logged.burn,
            &logged.hydrate,
            &DrinkingForm {
                date: "2026-10-10".into(),
                drink: "Expresso".into(),
                volume_ml: Some(160.0),
            },
        )
        .expect("a drink logs");

        let october = hydrate_month(&logged.hydrate, "2026-10").expect("a month");

        let today = october.rows[0];
        assert!((today.drunk_ml - 1660.0).abs() < 1e-9);
        assert!((today.water_ratio - 1500.0 / 1660.0).abs() < 1e-9);
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

    #[test]
    fn the_top_exercise_of_a_day_sums_all_its_workouts() {
        let logged = a_logged_man();
        add_exercise(&logged.burn, "Walking", 4.0).expect("an exercise adds");
        for (exercise, minutes) in [("Walking", 3.0), ("Push up", 2.0)] {
            log_workout(
                &logged.gather,
                &logged.burn,
                &WorkoutForm {
                    date: "2026-10-10".into(),
                    exercise: exercise.into(),
                    minutes: Some(minutes),
                    reps: None,
                    sets: Some(1),
                    load_kg: Some(0.0),
                },
            )
            .expect("a workout logs");
        }

        let october = burn_month(&logged.gather, &logged.burn, "2026-10").expect("a month");

        assert_eq!(october.rows[0].top_exercise.as_deref(), Some("Push up"));
    }

    #[test]
    fn insights_tally_each_habit_under_inside_and_over_its_band_over_the_window() {
        let logged = a_logged_man();

        let found = insights(
            &logged.gather,
            &logged.burn,
            &logged.fuel,
            &logged.hydrate,
            "2026-10-10",
            Window::Days30,
        )
        .expect("insights");

        assert_eq!(found.days, 30);
        assert_eq!(
            (
                found.weighed_days,
                found.fuelled_days,
                found.hydrated_days,
                found.active_days
            ),
            (2, 2, 2, 2)
        );
        for tally in [
            found.energy,
            found.protein,
            found.fat,
            found.fibre,
            found.carbs,
            found.hydrate,
        ] {
            assert_eq!(tally.low + tally.in_band + tally.high, 2, "{tally:?}");
        }
        assert_eq!(found.hydrate.low, 2);
        assert!(found.burnt_per_active_day.is_some_and(|kcal| kcal > 0.0));
    }

    #[test]
    fn over_everything_the_window_opens_on_the_first_record() {
        let logged = a_logged_man();

        let found = insights(
            &logged.gather,
            &logged.burn,
            &logged.fuel,
            &logged.hydrate,
            "2026-10-10",
            Window::Everything,
        )
        .expect("insights");

        assert_eq!(found.days, 71);
        assert_eq!(found.weighed_days, 3);
    }

    #[test]
    fn insights_read_the_burn_of_an_active_day_as_the_mean_of_its_days() {
        let logged = a_logged_man();
        let burnt_on = |date| {
            crate::for_burning::burn_day(&logged.gather, &logged.burn, date)
                .ok()
                .and_then(|burnt| burnt.burnt_kcal)
                .expect("a burn")
        };
        let mean = (burnt_on("2026-10-09") + burnt_on("2026-10-10")) / 2.0;

        let found = insights(
            &logged.gather,
            &logged.burn,
            &logged.fuel,
            &logged.hydrate,
            "2026-10-10",
            Window::Days30,
        )
        .expect("insights");

        assert!(
            found
                .burnt_per_active_day
                .is_some_and(|kcal| (kcal - mean).abs() < 1e-9)
        );
    }

    #[test]
    fn the_top_exercise_adds_up_its_own_workouts_whatever_came_between() {
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
        );
        log_measurement(&gather, "2026-10-05", 80.0, None).expect("a measurement logs");
        add_exercise(&burn, "Rowing", 4.0).expect("an exercise adds");
        add_exercise(&burn, "Squat", 4.0).expect("an exercise adds");
        for (exercise, minutes) in [("Rowing", 0.45), ("Squat", 0.25), ("Squat", 0.25)] {
            log_workout(
                &gather,
                &burn,
                &WorkoutForm {
                    date: "2026-10-05".into(),
                    exercise: exercise.into(),
                    minutes: Some(minutes),
                    reps: None,
                    sets: Some(1),
                    load_kg: Some(0.0),
                },
            )
            .expect("a workout logs");
        }

        let october = burn_month(&gather, &burn, "2026-10").expect("a month");

        assert_eq!(october.rows[0].top_exercise.as_deref(), Some("Squat"));
    }

    #[test]
    fn a_week_window_charts_the_seven_days_ending_today() {
        let logged = a_logged_man();
        log_measurement(&logged.gather, "2026-10-01", 81.0, Some(80.0))
            .expect("a measurement logs");

        let week = gather_chart(&logged.gather, "2026-10-10", Window::Days7).expect("a chart");
        let month = gather_chart(&logged.gather, "2026-10-10", Window::Days30).expect("a chart");

        assert_eq!(
            week.masses.iter().map(|(day, _)| *day).collect::<Vec<_>>(),
            vec![day("2026-10-09"), day("2026-10-10")]
        );
        assert_eq!(month.masses.len(), 3);
    }
}
