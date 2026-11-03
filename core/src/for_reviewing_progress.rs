use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::burn::burn_on;
use crate::burn_records::BurnRecords;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::fuel::{Fuel, fuel_over};
use crate::gather_records::GatherRecords;
use crate::hydrate::{Hydrate, WATER_SHARE_FLOOR_PERCENT, hydrate_over};
use crate::refusal::Refusal;
use crate::streak::{Streak, Ticks, streak_of};
use crate::verdict::{Verdict, verdict_of};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    Days7,
    Days30,
    Days90,
    Everything,
}

pub fn streak(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    today: &str,
) -> Result<Streak, Failure> {
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    let gathered = gather.gather_records()?;
    let burnt = burn.burn_records()?;
    let fuelled = fuel.fuel_records()?;
    let hydrated = hydrate.hydrate_records()?;
    let Some(start) = gathered.profiles.iter().map(|(from, _)| *from).min() else {
        return Ok(streak_of(&[], today));
    };
    let days: Vec<NaiveDate> = start.iter_days().take_while(|day| *day <= today).collect();
    let hydrated_days = hydrate_over(&days, &hydrated, &gathered, &burnt, &fuelled);
    let ticked: Vec<(NaiveDate, Ticks)> = days
        .iter()
        .zip(fuel_over(&days, &gathered, &burnt, &fuelled))
        .zip(&hydrated_days)
        .map(|((day, fuelled_day), hydrated_day)| {
            (
                *day,
                Ticks {
                    weighed: Some(
                        gathered
                            .measurements
                            .iter()
                            .any(|measured| measured.day == *day),
                    ),
                    fuelled: Some(fuelled_in_band(&fuelled_day)),
                    worked_out: Some(worked_out(*day, &burnt, &gathered)),
                    hydrated: Some(hydrated_in_band(hydrated_day)),
                },
            )
        })
        .collect();
    Ok(streak_of(&ticked, today))
}

/// A day's energy stands in its band and each of its four macros in its own, as the gauge and the table show
/// them.
fn fuelled_in_band(fuelled: &Fuel) -> bool {
    fuelled.energy == Some(Verdict::InBand)
        && fuelled.macros.is_some_and(|macros| {
            [macros.protein, macros.fat, macros.fibre, macros.carbs]
                .iter()
                .all(|row| row.verdict == Verdict::InBand)
        })
}

/// A day's drinks stand in their band and at least four fifths of them are water, as shown.
fn hydrated_in_band(hydrated: &Hydrate) -> bool {
    hydrated.verdict == Verdict::InBand
        && verdict_of(hydrated.water_share(), WATER_SHARE_FLOOR_PERCENT, 100.0) == Verdict::InBand
}

/// A day's workouts reach its burn target as both are shown; before any weigh-in prices them, a single
/// workout will do.
fn worked_out(day: NaiveDate, burnt: &BurnRecords, gathered: &GatherRecords) -> bool {
    let burn = burn_on(day, burnt, gathered);
    !burn.workouts.is_empty()
        && burn
            .verdict
            .is_none_or(|verdict| verdict == Verdict::InBand)
}

pub(crate) fn first_day_of(window: Window, today: NaiveDate) -> NaiveDate {
    match window {
        Window::Days7 => today - Days::new(6),
        Window::Days30 => today - Days::new(29),
        Window::Days90 => today - Days::new(89),
        Window::Everything => NaiveDate::MIN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_setting_up_the_profile::{GoalsForm, ProfileForm, set_goals, set_profile};
    use crate::for_tracking::log_measurement;
    use crate::gender::Gender;
    use crate::testing::GatherRecordsInMemory;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    mod the_streak {
        use super::*;
        use crate::burn_target::target_on;
        use crate::food::{Food, Nutrients, Unit};
        use crate::for_burning::{WorkoutForm, log_workout};
        use crate::for_cataloguing::{FoodForm, add_exercise, add_food};
        use crate::for_fuelling::{IntakeForm, log_intake};
        use crate::for_setting_up_the_profile::{
            FuelTargetsForm, UsualDayForm, set_fuel_targets, set_usual_day,
        };
        use crate::streak::Ticks;
        use crate::testing::{BurnRecordsInMemory, FuelRecordsInMemory, HydrateRecordsInMemory};

        struct Records {
            gather: GatherRecordsInMemory,
            burn: BurnRecordsInMemory,
            fuel: FuelRecordsInMemory,
            hydrate: HydrateRecordsInMemory,
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
                description: String::new(),
            };
            FoodForm {
                name: food.name,
                unit: food.unit,
                portion: food.portion,
                nutrients: food.nutrients,
                description: String::new(),
            }
        }

        fn a_man_at_his_desk() -> Records {
            let records = Records {
                gather: GatherRecordsInMemory::default(),
                burn: BurnRecordsInMemory::default(),
                fuel: FuelRecordsInMemory::default(),
                hydrate: HydrateRecordsInMemory::default(),
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
                &records.gather,
                &records.burn,
                &UsualDayForm {
                    from: "2026-03-01".into(),
                    kept: crate::kept::Kept::Own(crate::volitional::Volitional {
                        sitting_min: 480.0,
                        standing_min: 120.0,
                        moving_km: 3.0,
                    }),
                },
            )
            .expect("a usual day set");
            set_fuel_targets(
                &records.gather,
                &records.fuel,
                &FuelTargetsForm {
                    from: "2026-03-01".into(),
                    kept: crate::kept::Kept::Own(crate::fuel_records::FuelTargets {
                        protein_g_per_kg: 2.0,
                        protein_per: crate::fuel_records::Per::BodyMass,
                        fat_g_per_kg: 0.9,
                    }),
                },
            )
            .expect("targets set");
            set_goals(
                &records.gather,
                &GoalsForm {
                    from: "2026-03-01".into(),
                    kept: crate::kept::Kept::Own(crate::for_setting_up_the_profile::TypedGoals {
                        body_mass_kg: 70.0,
                        fat_percentage: 10.0,
                    }),
                },
            )
            .expect("goals set");
            add_exercise(&records.burn, "Walking", 4.0, "").expect("an exercise adds");
            add_food(&records.fuel, &plate("Plate", 35.0)).expect("a food adds");
            add_food(&records.fuel, &plate("Fibreless plate", 5.0)).expect("a food adds");
            crate::for_cataloguing::add_drink(
                &records.hydrate,
                "Water",
                2000.0,
                crate::drink::DrinkKind::Water,
                "",
            )
            .expect("a drink adds");
            records
        }

        const A_WALK_MINUTES: f64 = 30.0;

        /// A waist that leaves him well above the fat of his goals: a cut.
        const HIS_WAIST_CM: f64 = 90.0;

        fn a_full_day(records: &Records, date: &str, plate: &str) {
            a_full_day_walking(records, date, plate, A_WALK_MINUTES);
        }

        fn a_full_day_walking(records: &Records, date: &str, plate: &str, minutes: f64) {
            a_dry_day_walking(records, date, plate, minutes);
            crate::for_hydrating::log_drink(
                &records.gather,
                &records.burn,
                &records.fuel,
                &records.hydrate,
                &crate::for_hydrating::DrinkingForm {
                    date: date.into(),
                    drink: "Water".into(),
                    volume_ml: Some(3000.0),
                },
            )
            .expect("a drink logs");
        }

        fn a_dry_day(records: &Records, date: &str, plate: &str) {
            a_dry_day_walking(records, date, plate, A_WALK_MINUTES);
        }

        fn a_dry_day_walking(records: &Records, date: &str, plate: &str, minutes: f64) {
            log_measurement(&records.gather, date, 80.0, Some(HIS_WAIST_CM))
                .expect("a measurement logs");
            log_workout(
                &records.gather,
                &records.burn,
                &WorkoutForm {
                    date: date.into(),
                    exercise: "Walking".into(),
                    minutes: Some(minutes),
                    reps: None,
                    sets: Some(1),
                    load_kg: Some(0.0),
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
                    quantity: Some(1.0),
                },
            )
            .expect("an intake logs");
        }

        fn streak_on(records: &Records, today: &str) -> Streak {
            streak(
                &records.gather,
                &records.burn,
                &records.fuel,
                &records.hydrate,
                today,
            )
            .expect("a streak")
        }

        fn ticks_on(records: &Records, today: &str, date: &str) -> Option<Ticks> {
            streak_on(records, today)
                .history
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
        fn a_day_in_its_energy_band_with_a_macro_out_of_its_own_leaves_the_fuel_tick_open() {
            let records = a_man_at_his_desk();
            a_full_day(&records, "2026-10-08", "Plate");
            a_full_day(&records, "2026-10-09", "Fibreless plate");
            a_full_day(&records, "2026-10-10", "Plate");

            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-09").and_then(|ticks| ticks.fuelled),
                Some(false)
            );
        }

        #[test]
        fn a_day_drunk_in_its_band_with_less_than_four_fifths_of_water_leaves_the_hydrate_tick_open()
         {
            let records = a_man_at_his_desk();
            crate::for_cataloguing::add_drink(
                &records.hydrate,
                "Tea",
                200.0,
                crate::drink::DrinkKind::Other,
                "",
            )
            .expect("a drink adds");
            a_full_day(&records, "2026-10-09", "Plate");
            a_dry_day(&records, "2026-10-10", "Plate");
            for (drink, volume_ml) in [("Water", 2000.0), ("Tea", 1000.0)] {
                crate::for_hydrating::log_drink(
                    &records.gather,
                    &records.burn,
                    &records.fuel,
                    &records.hydrate,
                    &crate::for_hydrating::DrinkingForm {
                        date: "2026-10-10".into(),
                        drink: drink.into(),
                        volume_ml: Some(volume_ml),
                    },
                )
                .expect("a drink logs");
            }

            assert_eq!(
                [
                    ticks_on(&records, "2026-10-11", "2026-10-09").and_then(|ticks| ticks.hydrated),
                    ticks_on(&records, "2026-10-11", "2026-10-10").and_then(|ticks| ticks.hydrated)
                ],
                [Some(true), Some(false)]
            );
        }

        #[test]
        fn a_day_eaten_below_its_energy_band_leaves_the_fuel_tick_open() {
            let records = a_man_at_his_desk();
            a_full_day(&records, "2026-10-09", "Plate");
            log_intake(
                &records.gather,
                &records.burn,
                &records.fuel,
                &IntakeForm {
                    date: "2026-10-10".into(),
                    food: "Plate".into(),
                    quantity: Some(0.5),
                },
            )
            .expect("an intake logs");

            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-10").and_then(|ticks| ticks.fuelled),
                Some(false)
            );
        }

        #[test]
        fn from_the_profiles_first_day_every_tick_counts_a_tracker_without_a_log_failing_it() {
            let records = a_man_at_his_desk();
            log_measurement(&records.gather, "2026-10-05", 80.0, Some(HIS_WAIST_CM))
                .expect("a measurement logs");
            a_full_day(&records, "2026-10-10", "Plate");

            let streaked = streak_on(&records, "2026-10-10");

            assert_eq!(
                streaked.history.first().map(|first| first.day),
                Some(day("2026-03-01"))
            );
            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-06"),
                Some(Ticks {
                    weighed: Some(false),
                    fuelled: Some(false),
                    worked_out: Some(false),
                    hydrated: Some(false),
                })
            );
        }

        #[test]
        fn without_a_profile_there_is_no_streak_yet() {
            let records = Records {
                gather: GatherRecordsInMemory::default(),
                burn: BurnRecordsInMemory::default(),
                fuel: FuelRecordsInMemory::default(),
                hydrate: HydrateRecordsInMemory::default(),
            };
            log_measurement(&records.gather, "2026-10-10", 80.0, Some(80.0))
                .expect("a measurement logs");

            assert_eq!(streak_on(&records, "2026-10-10").history, vec![]);
        }

        #[test]
        fn no_record_at_all_is_no_streak_yet_and_broken_records_give_none() {
            let records = a_man_at_his_desk();

            assert_eq!(streak_on(&records, "2026-10-10").days, 0);
            assert_eq!(
                streak(
                    &crate::testing::BrokenGatherRecords,
                    &records.burn,
                    &records.fuel,
                    &records.hydrate,
                    "2026-10-10"
                ),
                Err(Failure::Broken("the disk is full".into()))
            );
            assert_eq!(
                streak(
                    &records.gather,
                    &records.burn,
                    &records.fuel,
                    &records.hydrate,
                    "2026-02-30"
                ),
                Err(Failure::Refused(Refusal::NotADate))
            );
        }

        #[test]
        fn a_day_burning_its_target_holds_the_workout_tick_and_a_stroll_short_of_it_does_not() {
            let records = a_man_at_his_desk();
            a_full_day_walking(&records, "2026-10-09", "Plate", 1.0);
            a_full_day_walking(&records, "2026-10-10", "Plate", 30.0);

            let worked_out_on =
                |date| ticks_on(&records, "2026-10-10", date).and_then(|ticks| ticks.worked_out);

            assert_eq!(
                (worked_out_on("2026-10-09"), worked_out_on("2026-10-10")),
                (Some(false), Some(true))
            );
        }

        #[test]
        fn a_burn_short_of_its_target_by_any_margin_misses_the_tick() {
            let records = a_man_at_his_desk();
            let target_kcal = target_on(
                NaiveDate::from_ymd_opt(2026, 10, 10).expect("a day"),
                &[],
                Some(80.0),
            )
            .expect("a target on a weighed body")
            .values;
            let walking = crate::exercise::Exercise {
                name: "Walking".into(),
                met: 4.0,
                description: String::new(),
            };
            let a_minute = crate::workout::burnt_by(
                &crate::workout::workout("2026-10-10", &walking, 1.0, None, 1, 0.0)
                    .expect("a workout"),
                80.0,
            );
            a_full_day_walking(
                &records,
                "2026-10-10",
                "Plate",
                0.97 * target_kcal / a_minute,
            );

            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-10").and_then(|ticks| ticks.worked_out),
                Some(false)
            );
        }

        #[test]
        fn a_burn_that_reaches_its_target_as_shown_holds_the_tick() {
            let records = a_man_at_his_desk();
            let target_kcal = target_on(
                NaiveDate::from_ymd_opt(2026, 10, 10).expect("a day"),
                &[],
                Some(80.0),
            )
            .expect("a target on a weighed body")
            .values;
            let walking = crate::exercise::Exercise {
                name: "Walking".into(),
                met: 4.0,
                description: String::new(),
            };
            let a_minute = crate::workout::burnt_by(
                &crate::workout::workout("2026-10-10", &walking, 1.0, None, 1, 0.0)
                    .expect("a workout"),
                80.0,
            );
            a_full_day_walking(
                &records,
                "2026-10-10",
                "Plate",
                (target_kcal - 0.1) / a_minute,
            );

            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-10").and_then(|ticks| ticks.worked_out),
                Some(true)
            );
        }

        #[test]
        fn before_any_weigh_in_a_single_workout_holds_the_workout_tick() {
            let records = a_man_at_his_desk();
            log_workout(
                &records.gather,
                &records.burn,
                &WorkoutForm {
                    date: "2026-10-10".into(),
                    exercise: "Walking".into(),
                    minutes: Some(1.0),
                    reps: None,
                    sets: Some(1),
                    load_kg: Some(0.0),
                },
            )
            .expect("a workout logs");

            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-10").and_then(|ticks| ticks.worked_out),
                Some(true)
            );
        }

        #[test]
        fn a_day_without_a_workout_reads_its_workout_tick_open() {
            let records = a_man_at_his_desk();
            a_full_day(&records, "2026-10-08", "Plate");
            log_measurement(&records.gather, "2026-10-09", 80.0, Some(HIS_WAIST_CM))
                .expect("a measurement logs");
            a_full_day(&records, "2026-10-10", "Plate");

            assert_eq!(
                ticks_on(&records, "2026-10-10", "2026-10-09").and_then(|ticks| ticks.worked_out),
                Some(false)
            );
        }

        #[test]
        fn a_band_missed_by_any_margin_misses_its_tick() {
            let records = a_man_at_his_desk();
            a_full_day(&records, "2026-10-09", "Plate");
            a_dry_day(&records, "2026-10-10", "Plate");
            let fuelled = crate::for_fuelling::fuel_day(
                &records.gather,
                &records.burn,
                &records.fuel,
                "2026-10-10",
            )
            .expect("a fuelled day");
            let cap_kcal = fuelled.band.map(|band| band.cap_kcal).expect("a band");
            let mut richer = plate("Richer plate", 35.0);
            richer.nutrients.energy_kcal = 1.03 * cap_kcal;
            add_food(&records.fuel, &richer).expect("a food adds");
            for (id, _) in fuelled.intakes {
                crate::for_fuelling::remove_intake(
                    &records.gather,
                    &records.burn,
                    &records.fuel,
                    "2026-10-10",
                    id,
                )
                .expect("an intake removed");
            }
            log_intake(
                &records.gather,
                &records.burn,
                &records.fuel,
                &IntakeForm {
                    date: "2026-10-10".into(),
                    food: "Richer plate".into(),
                    quantity: Some(1.0),
                },
            )
            .expect("an intake logs");
            let floor_ml = crate::for_hydrating::hydrate_day(
                &records.gather,
                &records.burn,
                &records.fuel,
                &records.hydrate,
                "2026-10-10",
            )
            .expect("a hydrate")
            .floor_ml;
            crate::for_hydrating::log_drink(
                &records.gather,
                &records.burn,
                &records.fuel,
                &records.hydrate,
                &crate::for_hydrating::DrinkingForm {
                    date: "2026-10-10".into(),
                    drink: "Water".into(),
                    volume_ml: Some(0.97 * floor_ml),
                },
            )
            .expect("a drink logs");

            let ticks = ticks_on(&records, "2026-10-10", "2026-10-10").expect("the day's ticks");

            assert_eq!((ticks.fuelled, ticks.hydrated), (Some(false), Some(false)));
        }

        #[test]
        fn a_day_must_drink_inside_its_hydrate_band() {
            let records = a_man_at_his_desk();
            a_full_day(&records, "2026-10-09", "Plate");
            a_full_day(&records, "2026-10-10", "Plate");

            let hydrated_on =
                |date| ticks_on(&records, "2026-10-10", date).and_then(|ticks| ticks.hydrated);

            assert_eq!(hydrated_on("2026-10-09"), Some(true));
            assert_eq!(hydrated_on("2026-10-10"), Some(true));
        }
    }
}
