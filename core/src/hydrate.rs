use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn::Burns;
use crate::burn_records::BurnRecords;
use crate::drink::DrinkKind;
use crate::drinking::Drinking;
use crate::gather_records::GatherRecords;
use crate::hydrate_base::default_base_ml;
use crate::hydrate_records::HydrateRecords;
use crate::kept::{InForce, Kept, in_force};
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

/// The least share of water among a day's drinks, in percent, the rest counting towards the drinks alone.
pub const WATER_SHARE_FLOOR_PERCENT: f64 = 80.0;

impl Hydrate {
    /// What the day drank of water.
    pub fn water_ml(&self) -> f64 {
        self.drinkings
            .iter()
            .filter(|(_, drunk)| drunk.kind == DrinkKind::Water)
            .map(|(_, drunk)| drunk.volume_ml)
            .sum()
    }

    /// The day's share of water among its drinks, in percent; nothing drunk, none.
    pub fn water_share(&self) -> f64 {
        if self.drunk_ml > 0.0 {
            100.0 * self.water_ml() / self.drunk_ml
        } else {
            0.0
        }
    }
}

const ML_PER_KCAL_SPENT: f64 = 1.0;
const BAND_SHARE_EITHER_SIDE: f64 = 0.20;

pub fn hydrate_on(
    day: NaiveDate,
    records: &HydrateRecords,
    gather: &GatherRecords,
    burn: &BurnRecords,
) -> Hydrate {
    hydrated(day, &Drinks::of(records, gather, burn))
}

/// Each day's drinks against its band.
pub fn hydrate_over(
    days: &[NaiveDate],
    records: &HydrateRecords,
    gather: &GatherRecords,
    burn: &BurnRecords,
) -> Vec<Hydrate> {
    let drinks = Drinks::of(records, gather, burn);
    days.iter().map(|day| hydrated(*day, &drinks)).collect()
}

/// A history's drinks read day after day: grouped by day, beside the burn they are judged against, gathered once
/// rather than for each day.
struct Drinks<'records> {
    records: &'records HydrateRecords,
    gather: &'records GatherRecords,
    burns: Burns<'records>,
    drinkings: BTreeMap<NaiveDate, Vec<(i64, Drinking)>>,
}

impl<'records> Drinks<'records> {
    fn of(
        records: &'records HydrateRecords,
        gather: &'records GatherRecords,
        burn: &'records BurnRecords,
    ) -> Self {
        Drinks {
            records,
            gather,
            burns: Burns::of(burn, gather),
            drinkings: records
                .drinkings
                .iter()
                .fold(BTreeMap::new(), |mut by_day, (id, drunk)| {
                    by_day
                        .entry(drunk.day)
                        .or_insert_with(Vec::new)
                        .push((*id, drunk.clone()));
                    by_day
                }),
        }
    }
}

/// The water a day recommended before the burn: EFSA's drinks for the gender, four fifths of its adequate intake of
/// total water [28], are its floor, a fifth under it (the user's rule).
pub(crate) fn recommended_water_ml(day: NaiveDate, gather: &GatherRecords) -> f64 {
    default_base_ml(version_in_force_on(&gather.profiles, day).map(|profile| profile.gender))
        / (1.0 - BAND_SHARE_EITHER_SIDE)
}

/// The water target in force on a day: kept from the latest day up to it (the earliest before any), its own
/// base or the recommendation; never kept, the recommendation.
pub(crate) fn water_target_in_force(
    day: NaiveDate,
    records: &HydrateRecords,
    gather: &GatherRecords,
) -> Option<InForce<f64>> {
    in_force(dated_version_in_force_on(&records.targets, day), || {
        Some(recommended_water_ml(day, gather))
    })
}

/// The day's drinks against its band: the optimal, a water target of one's own or the recommended one, plus a
/// millilitre per kcal burnt, within the 0.84 to 1.46 mL lost per kcal spent [30]; the band a fifth either side of
/// it (the user's rule).
fn hydrated(day: NaiveDate, drinks: &Drinks) -> Hydrate {
    let Drinks {
        records, gather, ..
    } = drinks;
    let drinkings = drinks.drinkings.get(&day).cloned().unwrap_or_default();
    let drunk_ml: f64 = drinkings.iter().map(|(_, drunk)| drunk.volume_ml).sum();
    let burnt_kcal = drinks.burns.on(day).burnt_kcal.unwrap_or(0.0);
    let base_ml = version_in_force_on(&records.targets, day)
        .and_then(Kept::own)
        .copied()
        .unwrap_or_else(|| recommended_water_ml(day, gather));
    let target_ml = base_ml + burnt_kcal * ML_PER_KCAL_SPENT;
    let floor_ml = target_ml * (1.0 - BAND_SHARE_EITHER_SIDE);
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
    use crate::measurement::Measurement;
    use crate::profile::Profile;
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
        }
    }

    #[test]
    fn a_day_without_its_own_target_aims_where_efsas_drinks_stand_a_fifth_under_plus_a_millilitre_per_kcal_burnt_twenty_percent_either_side()
     {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &a_man_measured_with_his_waist(),
            &walked_an_hour("2026-10-10"),
        );

        let optimal_ml = 2000.0 / 0.8 + 252.0;
        assert!(about(hydrate.target_ml, optimal_ml));
        assert!(about(hydrate.floor_ml, 0.8 * optimal_ml));
        assert!(about(hydrate.cap_ml, 1.2 * optimal_ml));
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
    fn a_burn_of_unknown_size_without_any_weigh_in_adds_nothing_to_the_target() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &GatherRecords::default(),
            &walked_an_hour("2026-10-10"),
        );

        assert!(about(hydrate.target_ml, 2500.0));
    }

    #[test]
    fn with_no_profile_yet_a_day_floors_at_a_mans_efsa_drinks() {
        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &GatherRecords::default(),
            &BurnRecords::default(),
        );

        assert!(about(hydrate.floor_ml, 2000.0));
        assert!(about(hydrate.cap_ml, 3000.0));
    }

    #[test]
    fn a_womans_day_floors_at_her_efsa_drinks() {
        let a_woman = GatherRecords {
            profiles: vec![(
                day("2026-03-01"),
                Profile {
                    birthday: day("1985-04-12"),
                    gender: Gender::Female,
                    height_cm: 165.0,
                    neck_cm: 32.0,
                    hip_cm: Some(95.0),
                },
            )],
            ..GatherRecords::default()
        };

        let hydrate = hydrate_on(
            day("2026-10-10"),
            &HydrateRecords::default(),
            &a_woman,
            &BurnRecords::default(),
        );

        assert!(about(hydrate.floor_ml, 1600.0));
        assert!(about(hydrate.target_ml, 2000.0));
    }

    #[test]
    fn a_day_drunk_under_its_band_reads_low_inside_it_in_band_and_over_it_high() {
        let records = having_drunk(vec![
            (1, drunk("2026-10-09", "Tea", 350.0)),
            (2, drunk("2026-10-10", "Tea", 1200.0)),
            (3, drunk("2026-10-10", "Tea", 1300.0)),
            (4, drunk("2026-10-11", "Tea", 3100.0)),
        ]);
        let verdict_on = |date| {
            hydrate_on(
                day(date),
                &records,
                &GatherRecords::default(),
                &BurnRecords::default(),
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
