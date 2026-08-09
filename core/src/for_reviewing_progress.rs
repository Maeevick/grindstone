use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::burn::burn_on;
use crate::composition::Composition;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_switch_over::ForStoringSwitchOver;
use crate::fuel::{Fuel, Verdict, fuel_over};
use crate::ledger::Day;
use crate::ledger::ledger;
use crate::progress::{Progress, progress_towards};
use crate::refusal::Refusal;
use crate::stats::{Measure, Summary, correlation, paired, summary};
use crate::streak::{Streak, Ticks, streak_of};
use crate::trend::seven_day_trend;
use crate::version::version_in_force_on;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    Days30,
    Days90,
    Everything,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GatherCard {
    pub masses: Vec<(NaiveDate, f64)>,
    pub trend: Vec<(NaiveDate, f64)>,
    pub target_kg: Option<f64>,
    pub progress: Option<Progress>,
    pub fat_change_kg: Option<f64>,
    pub lean_change_kg: Option<f64>,
}

pub fn gather_card(
    records: &impl ForStoringGatherRecords,
    today: &str,
    window: Window,
) -> Result<GatherCard, Failure> {
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    let first_day = match window {
        Window::Days30 => today - Days::new(29),
        Window::Days90 => today - Days::new(89),
        Window::Everything => NaiveDate::MIN,
    };
    let is_in_window = |day: &NaiveDate| (first_day..=today).contains(day);
    let gathered = records.gather_records()?;
    let mut readings: Vec<(NaiveDate, f64)> = gathered
        .measurements
        .iter()
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    readings.sort_by_key(|(day, _)| *day);
    let trend: Vec<(NaiveDate, f64)> = seven_day_trend(&readings)
        .into_iter()
        .filter(|(day, _)| is_in_window(day))
        .collect();
    let target_kg = version_in_force_on(&gathered.goals, today).map(|goals| goals.body_mass_kg);
    let compositions: Vec<Composition> = ledger(&gathered.measurements, &gathered.profiles)
        .into_iter()
        .filter(|day| is_in_window(&day.day))
        .filter_map(|day| day.composition)
        .collect();
    let first_and_last = compositions.first().zip(compositions.last());
    Ok(GatherCard {
        masses: readings
            .into_iter()
            .filter(|(day, _)| is_in_window(day))
            .collect(),
        progress: progress_towards(target_kg, &trend),
        trend,
        target_kg,
        fat_change_kg: first_and_last.map(|(first, last)| last.fat_mass_kg - first.fat_mass_kg),
        lean_change_kg: first_and_last.map(|(first, last)| last.lean_mass_kg - first.lean_mass_kg),
    })
}

pub fn streak(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    switch_over: &impl ForStoringSwitchOver,
    today: &str,
) -> Result<Streak, Failure> {
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    let switched_on = switch_over.switch_over()?;
    let gathered = gather.gather_records()?;
    let burnt = burn.burn_records()?;
    let fuelled = fuel.fuel_records()?;
    let first_weighed = gathered
        .measurements
        .iter()
        .map(|measured| measured.day)
        .min();
    let first_fuelled = fuelled.intakes.iter().map(|(_, eaten)| eaten.day).min();
    let first_worked_out = burnt.workouts.iter().map(|(_, done)| done.day).min();
    let Some(start) = [first_weighed, first_fuelled, first_worked_out]
        .into_iter()
        .flatten()
        .min()
    else {
        return Ok(streak_of(&[], today));
    };
    let counted_from = |first: Option<NaiveDate>| match switched_on {
        Some(_) => first,
        None => Some(start),
    };
    let (weighed_from, fuelled_from, worked_out_from) = (
        counted_from(first_weighed),
        counted_from(first_fuelled),
        counted_from(first_worked_out),
    );
    let macros_from = switched_on.unwrap_or(start);
    let days: Vec<NaiveDate> = start.iter_days().take_while(|day| *day <= today).collect();
    let ticked: Vec<(NaiveDate, Ticks)> = days
        .iter()
        .zip(fuel_over(&days, &gathered, &burnt, &fuelled))
        .map(|(day, fuelled_day)| {
            let ticked_from =
                |from: Option<NaiveDate>, held: bool| from.filter(|from| from <= day).map(|_| held);
            (
                *day,
                Ticks {
                    weighed: ticked_from(
                        weighed_from,
                        gathered
                            .measurements
                            .iter()
                            .any(|measured| measured.day == *day),
                    ),
                    fuelled: ticked_from(fuelled_from, in_band(&fuelled_day, *day >= macros_from)),
                    worked_out: ticked_from(
                        worked_out_from,
                        burnt.workouts.iter().any(|(_, done)| done.day == *day),
                    ),
                    hydrated: None,
                },
            )
        })
        .collect();
    Ok(streak_of(&ticked, today))
}

fn in_band(fuelled: &Fuel, macros_checked: bool) -> bool {
    let macros_in_band = fuelled.macros.is_some_and(|macros| {
        [macros.protein, macros.fat, macros.fibre, macros.carbs]
            .iter()
            .all(|row| row.verdict == Verdict::InBand)
    });
    fuelled.energy == Some(Verdict::InBand) && (!macros_checked || macros_in_band)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Explored {
    pub one: Vec<(NaiveDate, f64)>,
    pub other: Vec<(NaiveDate, f64)>,
    pub one_summary: Option<Summary>,
    pub other_summary: Option<Summary>,
    pub pairs: Vec<(f64, f64)>,
    pub correlation: Option<f64>,
}

pub fn explore(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    today: &str,
    window: Window,
    measures: (Measure, Measure),
) -> Result<Explored, Failure> {
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    let first_day = first_day_of(window, today);
    let is_in_window = |day: &NaiveDate| (first_day..=today).contains(day);
    let gathered = gather.gather_records()?;
    let burnt = burn.burn_records()?;
    let fuelled = fuel.fuel_records()?;
    let days_logged = |logged: Vec<NaiveDate>| -> Vec<NaiveDate> {
        let mut logged: Vec<NaiveDate> = logged.into_iter().filter(is_in_window).collect();
        logged.sort();
        logged.dedup();
        logged
    };
    let body: Vec<Day> = ledger(&gathered.measurements, &gathered.profiles)
        .into_iter()
        .filter(|measured| is_in_window(&measured.day))
        .collect();
    let fuel_days = fuel_over(
        &days_logged(fuelled.intakes.iter().map(|(_, eaten)| eaten.day).collect()),
        &gathered,
        &burnt,
        &fuelled,
    );
    let burn_days: Vec<(NaiveDate, Option<f64>)> =
        days_logged(burnt.workouts.iter().map(|(_, done)| done.day).collect())
            .into_iter()
            .map(|day| (day, burn_on(day, &burnt, &gathered.measurements).burnt_kcal))
            .collect();
    let series_of = |measure: Measure| -> Vec<(NaiveDate, f64)> {
        let from_body = |read: fn(&Day) -> Option<f64>| {
            body.iter()
                .filter_map(|measured| read(measured).map(|value| (measured.day, value)))
                .collect()
        };
        let from_fuel = |read: fn(&Fuel) -> Option<f64>| {
            fuel_days
                .iter()
                .filter_map(|fuelled| read(fuelled).map(|value| (fuelled.day, value)))
                .collect()
        };
        match measure {
            Measure::BodyMass => from_body(|measured| Some(measured.body_mass_kg)),
            Measure::Waist => from_body(|measured| measured.waist_cm),
            Measure::FatRatio => {
                from_body(|measured| measured.composition.map(|split| split.fat_ratio))
            }
            Measure::LeanMass => {
                from_body(|measured| measured.composition.map(|split| split.lean_mass_kg))
            }
            Measure::Eaten => from_fuel(|fuelled| Some(fuelled.eaten.energy_kcal)),
            Measure::Balance => from_fuel(|fuelled| {
                fuelled
                    .band
                    .map(|band| fuelled.eaten.energy_kcal - band.cap_kcal)
            }),
            Measure::Protein => from_fuel(|fuelled| Some(fuelled.eaten.protein_g)),
            Measure::Carbs => from_fuel(|fuelled| Some(fuelled.eaten.carbs_g)),
            Measure::Fat => from_fuel(|fuelled| Some(fuelled.eaten.fat_g)),
            Measure::Fibre => from_fuel(|fuelled| Some(fuelled.eaten.fibre_g)),
            Measure::Burnt => burn_days
                .iter()
                .filter_map(|(day, kcal)| kcal.map(|kcal| (*day, kcal)))
                .collect(),
        }
    };
    let (one, other) = (series_of(measures.0), series_of(measures.1));
    let pairs = paired(&one, &other);
    Ok(Explored {
        one_summary: summary(&one),
        other_summary: summary(&other),
        correlation: correlation(&pairs),
        pairs,
        one,
        other,
    })
}

fn first_day_of(window: Window, today: NaiveDate) -> NaiveDate {
    match window {
        Window::Days30 => today - Days::new(29),
        Window::Days90 => today - Days::new(89),
        Window::Everything => NaiveDate::MIN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::{Tapes, body_composition};
    use crate::for_setting_up_the_profile::{GoalsForm, ProfileForm, set_goals, set_profile};
    use crate::for_tracking::log_measurement;
    use crate::gender::Gender;
    use crate::testing::{BrokenGatherRecords, GatherRecordsInMemory};

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn measured(records: &GatherRecordsInMemory, days_and_masses: &[(&str, f64)]) {
        for (date, kg) in days_and_masses {
            log_measurement(records, date, *kg, None).expect("a measurement logs");
        }
    }

    #[test]
    fn the_gather_card_draws_the_masses_of_the_window_ending_today() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-08-01", 82.0),
                ("2026-09-20", 80.4),
                ("2026-10-10", 80.0),
            ],
        );

        let card = gather_card(&records, "2026-10-10", Window::Days30);

        assert_eq!(
            card.map(|card| card.masses),
            Ok(vec![(day("2026-09-20"), 80.4), (day("2026-10-10"), 80.0)])
        );
    }

    #[test]
    fn the_card_traces_the_trend_of_those_masses_with_their_week_before_the_window() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-09-08", 80.6),
                ("2026-09-12", 80.2),
                ("2026-09-14", 80.0),
            ],
        );

        let trend = gather_card(&records, "2026-10-10", Window::Days30)
            .map(|card| card.trend)
            .expect("a card");

        let expected = [
            (day("2026-09-12"), 80.4),
            (day("2026-09-14"), (80.6 + 80.2 + 80.0) / 3.0),
        ];
        assert_eq!(trend.len(), expected.len());
        assert!(
            trend
                .iter()
                .zip(expected)
                .all(|((day, mean), (expected_day, expected_mean))| {
                    *day == expected_day && (mean - expected_mean).abs() < 1e-9
                })
        );
    }

    #[test]
    fn the_card_reads_the_weekly_rate_towards_the_target_in_force_today() {
        let records = GatherRecordsInMemory::default();
        set_goals(
            &records,
            &GoalsForm {
                from: "2026-03-01".into(),
                body_mass_kg: 70.0,
                fat_percentage: 10.0,
            },
        )
        .expect("goals set");
        measured(
            &records,
            &[
                ("2026-09-19", 80.0),
                ("2026-09-26", 79.5),
                ("2026-10-03", 79.0),
                ("2026-10-10", 78.5),
            ],
        );

        let card = gather_card(&records, "2026-10-10", Window::Days30).expect("a card");

        assert_eq!(card.target_kg, Some(70.0));
        assert!(
            card.progress
                .is_some_and(|progress| (progress.per_week + 0.5).abs() < 1e-9)
        );
    }

    #[test]
    fn without_a_target_the_card_still_reads_the_weekly_rate() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-09-19", 80.0),
                ("2026-09-26", 79.5),
                ("2026-10-03", 79.0),
                ("2026-10-10", 78.5),
            ],
        );

        let card = gather_card(&records, "2026-10-10", Window::Days30).expect("a card");

        assert!(
            card.progress
                .is_some_and(|progress| (progress.per_week + 0.5).abs() < 1e-9)
        );
        assert_eq!(card.progress.and_then(|progress| progress.reached_on), None);
    }

    #[test]
    fn the_card_tells_how_much_fat_and_lean_mass_changed_over_the_window() {
        let records = GatherRecordsInMemory::default();
        set_profile(
            &records,
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
        log_measurement(&records, "2026-09-20", 80.0, Some(80.0)).expect("a measurement logs");
        log_measurement(&records, "2026-10-10", 78.0, Some(78.0)).expect("a measurement logs");
        let tapes = |waist_cm| Tapes::Male {
            neck_cm: 38.5,
            waist_cm,
        };
        let before = body_composition(80.0, 180.0, tapes(80.0)).expect("a composition");
        let after = body_composition(78.0, 180.0, tapes(78.0)).expect("a composition");

        let card = gather_card(&records, "2026-10-10", Window::Days30).expect("a card");

        assert!(card.fat_change_kg.is_some_and(|change| {
            (change - (after.fat_mass_kg - before.fat_mass_kg)).abs() < 1e-9
        }));
        assert!(card.lean_change_kg.is_some_and(|change| {
            (change - (after.lean_mass_kg - before.lean_mass_kg)).abs() < 1e-9
        }));
    }

    #[test]
    fn wider_windows_draw_older_masses() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-06-01", 83.0),
                ("2026-08-01", 82.0),
                ("2026-10-10", 80.0),
            ],
        );
        let masses_over = |window| {
            gather_card(&records, "2026-10-10", window)
                .map(|card| card.masses.iter().map(|(day, _)| *day).collect::<Vec<_>>())
        };

        assert_eq!(
            masses_over(Window::Days90),
            Ok(vec![day("2026-08-01"), day("2026-10-10")])
        );
        assert_eq!(
            masses_over(Window::Everything),
            Ok(vec![
                day("2026-06-01"),
                day("2026-08-01"),
                day("2026-10-10")
            ])
        );
    }

    #[test]
    fn the_thirty_day_window_counts_today_and_the_twenty_nine_days_before_it() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-09-10", 81.0),
                ("2026-09-11", 80.9),
                ("2026-10-10", 80.0),
            ],
        );

        let first_drawn = gather_card(&records, "2026-10-10", Window::Days30)
            .map(|card| card.masses.first().map(|(day, _)| *day));

        assert_eq!(first_drawn, Ok(Some(day("2026-09-11"))));
    }

    #[test]
    fn a_today_the_calendar_lacks_or_broken_records_give_no_card() {
        assert_eq!(
            gather_card(
                &GatherRecordsInMemory::default(),
                "2026-02-30",
                Window::Days30
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            gather_card(&BrokenGatherRecords, "2026-10-10", Window::Days30),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    mod the_streak {
        use super::*;
        use crate::food::{Food, Nutrients, Unit};
        use crate::for_burning::{WorkoutForm, log_workout};
        use crate::for_cataloguing::{FoodForm, add_exercise, add_food};
        use crate::for_fuelling::{IntakeForm, log_intake};
        use crate::for_setting_up_the_profile::{
            FuelTargetsForm, UsualDayForm, set_fuel_targets, set_usual_day,
        };
        use crate::streak::Ticks;
        use crate::testing::{BurnRecordsInMemory, FuelRecordsInMemory, SwitchOverInMemory};

        struct Records {
            gather: GatherRecordsInMemory,
            burn: BurnRecordsInMemory,
            fuel: FuelRecordsInMemory,
            switch_over: SwitchOverInMemory,
        }

        fn plate(name: &str, fibre_g: f64) -> FoodForm {
            let food = Food {
                name: name.into(),
                unit: Unit::Piece,
                portion: 1.0,
                nutrients: Nutrients {
                    energy_kcal: 4.0 * 170.0 + 4.0 * 230.0 + 9.0 * 80.0 + 2.0 * fibre_g,
                    protein_g: 170.0,
                    carbs_g: 230.0,
                    fat_g: 80.0,
                    fibre_g,
                },
            };
            FoodForm {
                name: food.name,
                unit: food.unit,
                portion: food.portion,
                nutrients: food.nutrients,
            }
        }

        fn a_man_at_his_desk() -> Records {
            let records = Records {
                gather: GatherRecordsInMemory::default(),
                burn: BurnRecordsInMemory::default(),
                fuel: FuelRecordsInMemory::default(),
                switch_over: SwitchOverInMemory::default(),
            };
            set_profile(
                &records.gather,
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
            set_usual_day(
                &records.burn,
                &UsualDayForm {
                    from: "2026-03-01".into(),
                    sitting_min: 480.0,
                    standing_min: 120.0,
                    moving_km: 3.0,
                },
            )
            .expect("a usual day set");
            set_fuel_targets(
                &records.fuel,
                &FuelTargetsForm {
                    from: "2026-03-01".into(),
                    protein_g_per_kg: 2.0,
                    fat_g_per_kg: 0.9,
                    fibre_g_per_kg: 0.4,
                },
            )
            .expect("targets set");
            add_exercise(&records.burn, "Walking", 4.0).expect("an exercise adds");
            add_food(&records.fuel, &plate("Plate", 35.0)).expect("a food adds");
            add_food(&records.fuel, &plate("Fibreless plate", 5.0)).expect("a food adds");
            records
        }

        fn a_full_day(records: &Records, date: &str, plate: &str) {
            log_measurement(&records.gather, date, 80.0, Some(80.0)).expect("a measurement logs");
            log_workout(
                &records.gather,
                &records.burn,
                &WorkoutForm {
                    date: date.into(),
                    exercise: "Walking".into(),
                    minutes: 1.0,
                    reps: None,
                    sets: 1,
                    load_kg: 0.0,
                },
            )
            .expect("a workout logs");
            log_intake(
                &records.gather,
                &records.burn,
                &records.fuel,
                &IntakeForm {
                    date: date.into(),
                    food: plate.into(),
                    quantity: 1.0,
                },
            )
            .expect("an intake logs");
        }

        fn streak_on(records: &Records, today: &str) -> Streak {
            streak(
                &records.gather,
                &records.burn,
                &records.fuel,
                &records.switch_over,
                today,
            )
            .expect("a streak")
        }

        fn ticks_on(records: &Records, today: &str, date: &str) -> Option<Ticks> {
            streak_on(records, today)
                .recent
                .into_iter()
                .find(|streaked| streaked.day == day(date))
                .map(|streaked| streaked.ticks)
        }

        #[test]
        fn days_weighed_fuelled_in_band_and_worked_out_make_the_streak() {
            let records = a_man_at_his_desk();
            for date in ["2026-10-08", "2026-10-09", "2026-10-10"] {
                a_full_day(&records, date, "Plate");
            }

            assert_eq!(streak_on(&records, "2026-10-10").days, 3);
        }

        #[test]
        fn before_the_switch_over_a_day_is_fuelled_on_its_energy_alone_and_after_it_on_its_macros_too()
         {
            let records = a_man_at_his_desk();
            a_full_day(&records, "2026-10-08", "Plate");
            a_full_day(&records, "2026-10-09", "Fibreless plate");
            a_full_day(&records, "2026-10-10", "Plate");

            let without_switch_over = ticks_on(&records, "2026-10-10", "2026-10-09");
            records
                .switch_over
                .save_switch_over(day("2026-10-10"))
                .expect("a switch-over saves");
            let after_the_import = ticks_on(&records, "2026-10-10", "2026-10-09");

            assert_eq!(
                without_switch_over.and_then(|ticks| ticks.fuelled),
                Some(false)
            );
            assert_eq!(after_the_import.and_then(|ticks| ticks.fuelled), Some(true));
        }

        #[test]
        fn once_imported_each_tick_counts_from_its_oldest_record() {
            let records = a_man_at_his_desk();
            log_measurement(&records.gather, "2026-10-05", 80.0, Some(80.0))
                .expect("a measurement logs");
            a_full_day(&records, "2026-10-10", "Plate");
            records
                .switch_over
                .save_switch_over(day("2026-10-10"))
                .expect("a switch-over saves");

            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-06"),
                Some(Ticks {
                    weighed: Some(false),
                    fuelled: None,
                    worked_out: None,
                    hydrated: None,
                })
            );
        }

        #[test]
        fn no_record_at_all_is_no_streak_yet_and_a_broken_switch_over_gives_none() {
            let records = a_man_at_his_desk();

            assert_eq!(streak_on(&records, "2026-10-10").days, 0);
            assert_eq!(
                streak(
                    &records.gather,
                    &records.burn,
                    &records.fuel,
                    &crate::testing::BrokenSwitchOver,
                    "2026-10-10"
                ),
                Err(Failure::Broken("the disk is full".into()))
            );
            assert_eq!(
                streak(
                    &records.gather,
                    &records.burn,
                    &records.fuel,
                    &records.switch_over,
                    "2026-02-30"
                ),
                Err(Failure::Refused(Refusal::NotADate))
            );
        }
    }

    mod exploring {
        use super::*;
        use crate::food::{Nutrients, Unit};
        use crate::for_burning::{WorkoutForm, log_workout};
        use crate::for_cataloguing::{FoodForm, add_exercise, add_food};
        use crate::for_fuelling::{IntakeForm, log_intake};
        use crate::stats::MEASURES;
        use crate::testing::{BurnRecordsInMemory, FuelRecordsInMemory};

        fn a_logged_man() -> (
            GatherRecordsInMemory,
            BurnRecordsInMemory,
            FuelRecordsInMemory,
        ) {
            let (gather, burn, fuel) = (
                GatherRecordsInMemory::default(),
                BurnRecordsInMemory::default(),
                FuelRecordsInMemory::default(),
            );
            set_profile(
                &gather,
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
            add_exercise(&burn, "Walking", 4.0).expect("an exercise adds");
            add_food(
                &fuel,
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
            for (date, kg) in [
                ("2026-10-08", 80.4),
                ("2026-10-09", 80.2),
                ("2026-10-10", 80.0),
            ] {
                log_measurement(&gather, date, kg, Some(80.0)).expect("a measurement logs");
            }
            for date in ["2026-10-09", "2026-10-10"] {
                log_intake(
                    &gather,
                    &burn,
                    &fuel,
                    &IntakeForm {
                        date: date.into(),
                        food: "Plate".into(),
                        quantity: 1.0,
                    },
                )
                .expect("an intake logs");
                log_workout(
                    &gather,
                    &burn,
                    &WorkoutForm {
                        date: date.into(),
                        exercise: "Walking".into(),
                        minutes: 30.0,
                        reps: None,
                        sets: 1,
                        load_kg: 0.0,
                    },
                )
                .expect("a workout logs");
            }
            (gather, burn, fuel)
        }

        #[test]
        fn exploring_mass_against_energy_eaten_pairs_the_days_both_were_logged() {
            let (gather, burn, fuel) = a_logged_man();

            let explored = explore(
                &gather,
                &burn,
                &fuel,
                "2026-10-10",
                Window::Days30,
                (Measure::BodyMass, Measure::Eaten),
            )
            .expect("an exploration");

            assert_eq!(explored.one.len(), 3);
            assert_eq!(explored.pairs, vec![(80.2, 2400.0), (80.0, 2400.0)]);
            assert_eq!(
                explored.one_summary.map(|summary| summary.first),
                Some(80.4)
            );
            assert_eq!(explored.correlation, None);
        }

        #[test]
        fn every_measure_reads_a_series_from_the_logged_days() {
            let (gather, burn, fuel) = a_logged_man();

            let empty: Vec<Measure> = MEASURES
                .iter()
                .filter(|measure| {
                    explore(
                        &gather,
                        &burn,
                        &fuel,
                        "2026-10-10",
                        Window::Everything,
                        (**measure, **measure),
                    )
                    .map_or(true, |explored| explored.one.is_empty())
                })
                .copied()
                .collect();

            assert_eq!(empty, vec![]);
        }

        #[test]
        fn a_today_the_calendar_lacks_or_broken_records_explore_nothing() {
            let (gather, burn, fuel) = a_logged_man();

            assert_eq!(
                explore(
                    &gather,
                    &burn,
                    &fuel,
                    "2026-02-30",
                    Window::Days30,
                    (Measure::Fat, Measure::Burnt)
                ),
                Err(Failure::Refused(Refusal::NotADate))
            );
            assert_eq!(
                explore(
                    &gather,
                    &crate::testing::BrokenBurnRecords,
                    &fuel,
                    "2026-10-10",
                    Window::Days30,
                    (Measure::Fat, Measure::Burnt)
                ),
                Err(Failure::Broken("the disk is full".into()))
            );
        }
    }
}
