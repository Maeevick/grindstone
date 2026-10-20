use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::body::body_on;
use crate::burn::burn_on;
use crate::burn_records::BurnRecords;
use crate::drinking::Drinking;
use crate::fuel::{Fuel, fuel_on, fuel_over};
use crate::fuel_records::FuelRecords;
use crate::gather_records::GatherRecords;
use crate::hydrate_base::default_base_ml;
use crate::hydrate_records::HydrateRecords;
use crate::kept::{InForce, Kept, in_force};
use crate::ledger::ledger;
use crate::verdict::{Verdict, verdict_of};
use crate::version::{dated_version_in_force_on, version_in_force_on};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hydrate {
    pub day: NaiveDate,
    pub drinkings: Vec<(i64, Drinking)>,
    pub drunk_ml: f64,
    pub target_ml: f64,
    pub floor_ml: f64,
    pub cap_ml: f64,
    pub verdict: Verdict,
}

const ML_PER_KCAL_SPENT: f64 = 1.0;
const BAND_SHARE_EITHER_SIDE: f64 = 0.20;
const WATER_SHARE_OF_LEAN_MASS: f64 = 0.732;
const BODY_WATER_TURNED_OVER_A_DAY: f64 = 0.07;
const ML_PER_KCAL_BURNT_AT_BEST: f64 = 1.25;
const SHARE_LEFT_TO_DRINKS: f64 = 0.9;
const ML_PER_L: f64 = 1000.0;

pub fn hydrate_on(
    day: NaiveDate,
    records: &HydrateRecords,
    gather: &GatherRecords,
    burn: &BurnRecords,
    fuel: &FuelRecords,
) -> Hydrate {
    hydrated(
        day,
        records,
        gather,
        burn,
        &fuel_on(day, gather, burn, fuel),
    )
}

pub fn hydrate_over(
    days: &[NaiveDate],
    records: &HydrateRecords,
    gather: &GatherRecords,
    burn: &BurnRecords,
    fuel: &FuelRecords,
) -> Vec<Hydrate> {
    days.iter()
        .zip(fuel_over(days, gather, burn, fuel))
        .map(|(day, fuelled)| hydrated(*day, records, gather, burn, &fuelled))
        .collect()
}

fn lean_kg_on(day: NaiveDate, gather: &GatherRecords) -> Option<f64> {
    body_on(
        day,
        &ledger(&gather.measurements, &gather.profiles),
        &gather.profiles,
    )
    .map(|body| body.lean_kg)
}

fn turned_over_ml(lean_kg: f64) -> f64 {
    lean_kg * WATER_SHARE_OF_LEAN_MASS * BODY_WATER_TURNED_OVER_A_DAY * ML_PER_L
}

/// The water target in force on a day: kept from the latest day up to it (the earliest before any), its own
/// base or the recommendation; never kept, the recommendation: once a lean mass is known, nine tenths of the
/// water it turns over in a day, before, EFSA's base.
pub(crate) fn water_target_in_force(
    day: NaiveDate,
    records: &HydrateRecords,
    gather: &GatherRecords,
) -> Option<InForce<f64>> {
    in_force(dated_version_in_force_on(&records.targets, day), || {
        Some(match lean_kg_on(day, gather) {
            Some(lean_kg) => SHARE_LEFT_TO_DRINKS * turned_over_ml(lean_kg),
            None => default_base_ml(
                version_in_force_on(&gather.profiles, day).map(|profile| profile.gender),
            ),
        })
    })
}

/// The day's drinks against its band. A water target of one's own is a base: it plus a millilitre per kcal
/// burnt, the band a fifth either side. Otherwise, once a lean mass is known, the target is the water that
/// mass turns over in a day plus 1.25 mL per kcal burnt, and the floor a millilitre per kcal the day spends
/// (its budget and burn) but never above the target, each nine tenths from drinks, food bringing the rest;
/// before, EFSA's base plus a millilitre per kcal burnt, the floor a fifth under it.
///
/// - Lean mass is 73.2% water: Wang Z. et al. (1999), *Hydration of fat-free body mass: new
///   physiological modeling approach*, Am J Physiol Endocrinol Metab 276(6):E995-E1003.
/// - About 7% of the body's water turned over a day: Yamada Y. et al. (2022), *Variation in human
///   water turnover associated with environmental and lifestyle factors*, Science 378(6622):909-915.
/// - Water lost per kcal spent, 0.84 to 1.46 mL: Westerterp K. R., Plasqui G., Goris A. H. C. (2005),
///   *Water loss as a function of energy intake, physical activity and season*, Br J Nutr 93(2):199-203.
fn hydrated(
    day: NaiveDate,
    records: &HydrateRecords,
    gather: &GatherRecords,
    burn: &BurnRecords,
    fuelled: &Fuel,
) -> Hydrate {
    let drinkings: Vec<(i64, Drinking)> = records
        .drinkings
        .iter()
        .filter(|(_, drunk)| drunk.day == day)
        .cloned()
        .collect();
    let drunk_ml: f64 = drinkings.iter().map(|(_, drunk)| drunk.volume_ml).sum();
    let burnt_kcal = burn_on(day, burn, gather).burnt_kcal.unwrap_or(0.0);
    let own_base_ml = version_in_force_on(&records.targets, day)
        .and_then(Kept::own)
        .copied();
    let (target_ml, floor_ml) = match (own_base_ml, lean_kg_on(day, gather), fuelled.band) {
        (Some(base_ml), _, _) => {
            let target_ml = base_ml + burnt_kcal * ML_PER_KCAL_SPENT;
            (target_ml, target_ml * (1.0 - BAND_SHARE_EITHER_SIDE))
        }
        (None, Some(lean_kg), Some(band)) => {
            let target_ml = SHARE_LEFT_TO_DRINKS
                * (turned_over_ml(lean_kg) + burnt_kcal * ML_PER_KCAL_BURNT_AT_BEST);
            let floor_ml = SHARE_LEFT_TO_DRINKS * band.cap_kcal * ML_PER_KCAL_SPENT;
            (target_ml, floor_ml.min(target_ml))
        }
        _ => {
            let gender = version_in_force_on(&gather.profiles, day).map(|profile| profile.gender);
            let target_ml = default_base_ml(gender) + burnt_kcal * ML_PER_KCAL_SPENT;
            (target_ml, target_ml * (1.0 - BAND_SHARE_EITHER_SIDE))
        }
    };
    let cap_ml = target_ml * (1.0 + BAND_SHARE_EITHER_SIDE);
    Hydrate {
        day,
        drinkings,
        drunk_ml,
        target_ml,
        floor_ml,
        cap_ml,
        verdict: verdict_of(drunk_ml, floor_ml, cap_ml),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;
    use crate::kept::Kept;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::volitional::Volitional;
    use crate::workout::Workout;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    fn drunk(date: &str, drink: &str, volume_ml: f64) -> Drinking {
        Drinking {
            day: day(date),
            drink: drink.into(),
            volume_ml,
            kind: crate::drink::DrinkKind::Other,
            servings: 1.0,
        }
    }

    fn having_drunk(drinkings: Vec<(i64, Drinking)>) -> HydrateRecords {
        HydrateRecords {
            drinkings,
            ..HydrateRecords::default()
        }
    }

    fn walked_an_hour(date: &str) -> BurnRecords {
        BurnRecords {
            workouts: vec![(
                1,
                Workout {
                    day: day(date),
                    exercise: "Walking".into(),
                    met: 4.0,
                    minutes: 60.0,
                    reps: None,
                    sets: 1,
                    load_kg: 0.0,
                    circuit: None,
                },
            )],
            ..BurnRecords::default()
        }
    }

    fn weighed(date: &str, body_mass_kg: f64) -> GatherRecords {
        GatherRecords {
            measurements: vec![Measurement {
                day: day(date),
                body_mass_kg,
                waist_cm: None,
            }],
            ..GatherRecords::default()
        }
    }

    fn a_man_measured_with_his_waist() -> GatherRecords {
        GatherRecords {
            measurements: vec![Measurement {
                day: day("2026-10-01"),
                body_mass_kg: 80.0,
                waist_cm: Some(85.0),
            }],
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
            goals: vec![],
            counted_from: vec![],
        }
    }

    fn lean_mass_kg_of(gather: &GatherRecords) -> f64 {
        crate::ledger::ledger(&gather.measurements, &gather.profiles)
            .iter()
            .find_map(|measured| measured.composition)
            .map(|composition| composition.lean_mass_kg)
            .expect("a lean mass read from the tapes")
    }

    #[test]
    fn a_day_aims_for_ninety_percent_of_its_lean_mass_water_turnover_and_a_millilitre_and_a_quarter_per_kcal_burnt()
     {
        let gather = a_man_measured_with_his_waist();

        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &gather,
            &walked_an_hour("2026-10-10"),
            &FuelRecords::default(),
        );

        let optimal_ml = 0.9 * (lean_mass_kg_of(&gather) * 0.732 * 0.07 * 1000.0 + 1.25 * 252.0);
        assert!(about(hydrate.target_ml, optimal_ml));
        assert!(about(hydrate.cap_ml, 1.2 * optimal_ml));
    }

    #[test]
    fn a_day_floors_at_ninety_percent_of_a_millilitre_per_kcal_of_its_budget_and_burn() {
        let gather = a_man_measured_with_his_waist();
        let burn = walked_an_hour("2026-10-10");
        let fuel = FuelRecords::default();

        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &gather,
            &burn,
            &fuel,
        );

        let spent_kcal = fuel_on(day("2026-10-10"), &gather, &burn, &fuel)
            .band
            .map(|band| band.cap_kcal)
            .expect("a band for a measured body");
        assert!(about(hydrate.floor_ml, 0.9 * spent_kcal));
    }

    #[test]
    fn the_floor_never_rises_above_the_optimal() {
        let walking_thirty_km_a_day = BurnRecords {
            usual_days: vec![(
                day("2026-03-01"),
                Kept::Own(Volitional {
                    sitting_min: 480.0,
                    standing_min: 120.0,
                    moving_km: 30.0,
                }),
            )],
            ..BurnRecords::default()
        };

        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &a_man_measured_with_his_waist(),
            &walking_thirty_km_a_day,
            &FuelRecords::default(),
        );

        assert!(about(hydrate.floor_ml, hydrate.target_ml));
    }

    #[test]
    fn a_day_drinks_the_sum_of_its_own_drinkings() {
        let records = having_drunk(vec![
            (1, drunk("2026-10-10", "Expresso", 80.0)),
            (2, drunk("2026-10-10", "Tea", 200.0)),
            (3, drunk("2026-10-11", "Tea", 400.0)),
        ]);

        let hydrate = hydrate_on(
            day("2026-10-10"),
            &records,
            &GatherRecords::default(),
            &BurnRecords::default(),
            &FuelRecords::default(),
        );

        assert_eq!(
            hydrate.drinkings,
            vec![
                (1, drunk("2026-10-10", "Expresso", 80.0)),
                (2, drunk("2026-10-10", "Tea", 200.0)),
            ]
        );
        assert!(about(hydrate.drunk_ml, 280.0));
    }

    #[test]
    fn a_day_before_any_lean_mass_aims_for_efsas_base_and_a_millilitre_more_per_kcal_burnt() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &weighed("2026-10-01", 80.0),
            &walked_an_hour("2026-10-10"),
            &FuelRecords::default(),
        );

        assert!(about(hydrate.target_ml, 2000.0 + 252.0));
    }

    #[test]
    fn a_burn_of_unknown_size_without_any_weigh_in_adds_nothing_to_the_target() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &GatherRecords::default(),
            &walked_an_hour("2026-10-10"),
            &FuelRecords::default(),
        );

        assert!(about(hydrate.target_ml, 2000.0));
    }

    #[test]
    fn a_day_before_any_lean_mass_keeps_efsa_twenty_percent_either_side() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &GatherRecords::default(),
            &BurnRecords::default(),
            &FuelRecords::default(),
        );

        assert!(about(hydrate.floor_ml, 1600.0));
        assert!(about(hydrate.cap_ml, 2400.0));
    }

    #[test]
    fn a_day_drunk_under_its_band_reads_low_inside_it_in_band_and_over_it_high() {
        let records = having_drunk(vec![
            (1, drunk("2026-10-09", "Tea", 350.0)),
            (2, drunk("2026-10-10", "Tea", 1200.0)),
            (3, drunk("2026-10-10", "Tea", 800.0)),
            (4, drunk("2026-10-11", "Tea", 2650.0)),
        ]);
        let verdict_on = |date| {
            hydrate_on(
                day(date),
                &records,
                &GatherRecords::default(),
                &BurnRecords::default(),
                &FuelRecords::default(),
            )
            .verdict
        };

        assert_eq!(
            [
                verdict_on("2026-10-09"),
                verdict_on("2026-10-10"),
                verdict_on("2026-10-11")
            ],
            [Verdict::Low, Verdict::InBand, Verdict::High]
        );
    }
}
