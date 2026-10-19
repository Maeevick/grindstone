use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::body::body_on;
use crate::fuel::SPARED_KCAL_PER_KG_OF_FAT;
use crate::gather_records::GatherRecords;
use crate::ledger::ledger;
use crate::progress::progress_towards;
use crate::recommendations::recommended;
use crate::tendency::{Change, Heading};
use crate::trend::seven_day_trend;
use crate::version::version_in_force_on;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Judgement {
    Holding,
    VeryGood,
    Good,
    Stable,
    WrongWay,
    GainingTooFast,
    LosingTooFast,
    UnderHealthyMass,
    UnderHealthyFat,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pace {
    pub body_mass: Change,
    pub lean_mass: Option<Change>,
    pub fat_mass: Option<Change>,
    pub fat_ratio: Option<Change>,
    pub judgement: Judgement,
    pub safe_loss_kg: f64,
}

const DAYS_OF_THE_SLOPE: u64 = 13;
const DAYS_OF_THE_AVERAGE: u64 = 6;
const DAYS_A_WEEK: f64 = 7.0;
const LOSS_SHARE_A_WEEK_AT_MOST: f64 = 0.01;
const LOSS_KG_A_WEEK_AT_MOST: f64 = 1.0;
const KCAL_PER_KG_OF_FAT_TISSUE: f64 = 7700.0;
const STABLE_SHARE_A_WEEK: f64 = 0.0015;

/// The most a body can lose in a week without eating into its lean mass: 1% of its mass, 1 kg at most, and
/// no more than its fat store can spare.
///
/// - 0.5 to 1% of the body mass a week keeps the muscle: Helms E. R., Aragon A. A., Fitschen P. J. (2014),
///   J Int Soc Sports Nutr 11:20; 0.7% kept more lean mass than 1.4%: Garthe I. et al. (2011), Int J Sport Nutr
///   Exerc Metab 21(2):97-104.
/// - No more than 0.5 to 1 kg a week: NICE (2025), *Overweight and obesity management*, NG246.
/// - What the fat store can spare, about 48.5 kcal per kilo of fat a day: Alpert S. S. (2005), J Theor Biol
///   233(1):1-13; about 7,700 kcal a kilo of fat tissue: Wishnofsky M. (1958), Am J Clin Nutr 6(5):542-546.
fn safe_loss_kg(body_mass_kg: f64, fat_kg: Option<f64>) -> f64 {
    let spared = fat_kg.map_or(f64::INFINITY, |fat_kg| {
        SPARED_KCAL_PER_KG_OF_FAT * fat_kg * DAYS_A_WEEK / KCAL_PER_KG_OF_FAT_TISSUE
    });
    (LOSS_SHARE_A_WEEK_AT_MOST * body_mass_kg)
        .min(LOSS_KG_A_WEEK_AT_MOST)
        .min(spared)
}

/// The week's pace on a day: the slope, per week, of the seven-day average of the body mass over the
/// fourteen days ending on it.
pub fn pace_on(day: NaiveDate, gathered: &GatherRecords) -> Option<Pace> {
    let readings: Vec<(NaiveDate, f64)> = gathered
        .measurements
        .iter()
        .filter(|measured| {
            measured.day <= day
                && measured.day + Days::new(DAYS_OF_THE_SLOPE + DAYS_OF_THE_AVERAGE) >= day
        })
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    let trend: Vec<(NaiveDate, f64)> = seven_day_trend(&readings)
        .into_iter()
        .filter(|(averaged, _)| *averaged + Days::new(DAYS_OF_THE_SLOPE) >= day)
        .collect();
    let progress = progress_towards(None, &trend)?;
    let (_, body_mass_kg) = *trend.last()?;
    let body = body_on(
        day,
        &ledger(&gathered.measurements, &gathered.profiles),
        &gathered.profiles,
    );
    let safe_loss_kg = safe_loss_kg(body_mass_kg, body.as_ref().map(|body| body.fat_kg));
    let healthy = version_in_force_on(&gathered.profiles, day)
        .map(|profile| recommended(profile, day, Some(body_mass_kg)));
    let fat_ratio = body.as_ref().map(|body| body.fat_kg / body.mass_kg);
    let losing = progress.per_week < -STABLE_SHARE_A_WEEK * body_mass_kg;
    let judgement =
        if losing && healthy.is_some_and(|healthy| body_mass_kg < healthy.healthy_mass_kg.0) {
            Judgement::UnderHealthyMass
        } else if losing
            && healthy
                .zip(fat_ratio)
                .is_some_and(|(healthy, fat_ratio)| fat_ratio < healthy.healthy_fat_ratio.0)
        {
            Judgement::UnderHealthyFat
        } else if -progress.per_week > safe_loss_kg {
            Judgement::LosingTooFast
        } else {
            Judgement::Stable
        };
    Some(Pace {
        body_mass: Change {
            by: progress.per_week,
            heading: Heading::Unaimed,
        },
        lean_mass: None,
        fat_mass: None,
        fat_ratio: None,
        judgement,
        safe_loss_kg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::gender::Gender;
    use crate::measurement::Measurement;
    use crate::profile::Profile;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn a_man_weighed(from_kg: f64, kg_a_day: f64) -> GatherRecords {
        let last = day("2026-10-10");
        GatherRecords {
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
            measurements: (0..21)
                .map(|at| Measurement {
                    day: last - Days::new(20 - at),
                    body_mass_kg: from_kg + kg_a_day * at as f64,
                    waist_cm: Some(82.0),
                })
                .collect(),
            goals: vec![],
        }
    }

    #[test]
    fn a_weeks_pace_is_the_slope_of_the_seven_day_average_over_the_last_fourteen_days() {
        let pace = pace_on(day("2026-10-10"), &a_man_weighed(80.0, -0.05)).expect("a pace");

        assert!(
            (pace.body_mass.by - -0.35).abs() < 1e-9,
            "{}",
            pace.body_mass.by
        );
    }

    fn aiming_at(body_mass_kg: f64, gathered: GatherRecords) -> GatherRecords {
        GatherRecords {
            goals: vec![(
                day("2026-03-01"),
                crate::goals::Goals {
                    body_mass_kg,
                    fat_ratio: 0.12,
                },
            )],
            ..gathered
        }
    }

    fn fat_kg_of(gathered: &GatherRecords) -> f64 {
        crate::ledger::ledger(&gathered.measurements, &gathered.profiles)
            .last()
            .and_then(|measured| measured.composition)
            .map(|composition| composition.fat_mass_kg)
            .expect("a fat mass")
    }

    #[test]
    fn losing_faster_than_the_fat_store_can_spare_a_week_is_dangerous() {
        let losing_a_kilo_a_week = aiming_at(70.0, a_man_weighed(81.0, -0.14));

        let pace = pace_on(day("2026-10-10"), &losing_a_kilo_a_week).expect("a pace");

        assert_eq!(pace.judgement, Judgement::LosingTooFast);
        assert!(
            (pace.safe_loss_kg - 48.5 * fat_kg_of(&losing_a_kilo_a_week) * 7.0 / 7700.0).abs()
                < 1e-9
        );
    }

    fn judged(gathered: &GatherRecords) -> Judgement {
        pace_on(day("2026-10-10"), gathered)
            .expect("a pace")
            .judgement
    }

    fn a_man_with_a_waist_weighed(waist_cm: f64, from_kg: f64, kg_a_day: f64) -> GatherRecords {
        let weighed = a_man_weighed(from_kg, kg_a_day);
        GatherRecords {
            measurements: weighed
                .measurements
                .iter()
                .map(|measured| Measurement {
                    waist_cm: Some(waist_cm),
                    ..*measured
                })
                .collect(),
            ..weighed
        }
    }

    #[test]
    fn losing_under_the_healthy_mass_or_fat_is_dangerous_whatever_the_aim() {
        let under_the_healthy_mass = aiming_at(55.0, a_man_with_a_waist_weighed(72.0, 59.0, -0.02));
        let under_the_healthy_fat = aiming_at(70.0, a_man_with_a_waist_weighed(70.0, 76.0, -0.02));

        assert_eq!(
            (
                judged(&under_the_healthy_mass),
                judged(&under_the_healthy_fat)
            ),
            (Judgement::UnderHealthyMass, Judgement::UnderHealthyFat)
        );
    }
}
