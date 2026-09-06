//! Health recommendations to start from, never a prescription: every figure here is meant
//! to be checked and adjusted as the logs show what one body actually does.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::age::age_on;
use crate::fuel_records::FuelTargets;
use crate::gender::Gender;
use crate::hydrate_base::default_base_ml;
use crate::profile::Profile;
use crate::volitional::Volitional;

/// How a usual week runs, at work, in training and in hobbies. It sets the protein and
/// the usual day off the clock, never an activity multiplier on energy: the band reads
/// the burn from the workouts actually logged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rhythm {
    Sedentary,
    LightlyActive,
    ModeratelyActive,
    VeryActive,
    ExtremelyActive,
}

pub const RHYTHMS: [Rhythm; 5] = [
    Rhythm::Sedentary,
    Rhythm::LightlyActive,
    Rhythm::ModeratelyActive,
    Rhythm::VeryActive,
    Rhythm::ExtremelyActive,
];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recommendations {
    pub healthy_mass_kg: (f64, f64),
    pub goal_mass_kg: f64,
    pub healthy_fat_ratio: (f64, f64),
    pub goal_fat_ratio: f64,
    pub fuel: FuelTargets,
    pub hydrate_base_ml: f64,
    pub usual_day: Volitional,
}

/// The recommendations for a profile on a day, at a rhythm.
///
/// - Healthy mass: the World Health Organization's normal body mass index, 18.5 to 24.9,
///   at the profile's height; the goal sits at its middle, 21.7.
///   WHO (2000), *Obesity: preventing and managing the global epidemic*, Technical Report Series 894.
/// - Healthy fat: the percentage body fat predicted at a BMI of 18.5 and 25 for the age and gender,
///   Gallagher D. et al. (2000), *Am J Clin Nutr* 72(3):694–701, table 4; the goal sits at its middle.
/// - Protein: 0.83 g/kg for a sedentary rhythm, the safe level of WHO/FAO/UNU (2007), Technical
///   Report Series 935, rising to the 1.4 to 2.0 g/kg of Jäger R. et al. (2017),
///   *J Int Soc Sports Nutr* 14:20, for active ones.
/// - Fat: 0.8 g/kg as a practical floor; the day's band also asks 20 to 33.3% of the energy eaten,
///   within EFSA's 20 to 35% (2010) and near WHO's 30% at most (2023).
/// - Fibre: 25 g a day, WHO (2023), *Carbohydrate intake for adults and children*, spread per kilo.
/// - Water from drinks: EFSA's adequate intake of total water (2010), four fifths of it from drinks.
/// - The usual day: steps after Tudor-Locke C. and Bassett D. R. (2004), *Sports Med* 34(1):1–8,
///   at about 0.75 m a step; the sitting and standing minutes are estimates to adjust.
pub fn recommended(
    profile: &Profile,
    day: NaiveDate,
    body_mass_kg: Option<f64>,
    rhythm: Rhythm,
) -> Recommendations {
    let height_m = profile.height_cm / 100.0;
    let squared = height_m * height_m;
    let goal_mass_kg = MIDDLE_OF_NORMAL_BMI * squared;
    let healthy_fat_ratio = healthy_fat_ratio(profile.gender, age_on(profile.birthday, day));
    let (protein_g_per_kg, steps, (sitting_min, standing_min)) = by_rhythm(rhythm);
    Recommendations {
        healthy_mass_kg: (LOWEST_NORMAL_BMI * squared, HIGHEST_NORMAL_BMI * squared),
        goal_mass_kg,
        healthy_fat_ratio,
        goal_fat_ratio: to_the_thousandth((healthy_fat_ratio.0 + healthy_fat_ratio.1) / 2.0),
        fuel: FuelTargets {
            protein_g_per_kg,
            fat_g_per_kg: FAT_FLOOR_G_PER_KG,
            fibre_g_per_kg: to_the_hundredth(FIBRE_G_A_DAY / body_mass_kg.unwrap_or(goal_mass_kg)),
        },
        hydrate_base_ml: default_base_ml(Some(profile.gender)),
        usual_day: Volitional {
            sitting_min,
            standing_min,
            moving_km: to_the_tenth(steps * METRES_A_STEP / 1000.0),
        },
    }
}

const LOWEST_NORMAL_BMI: f64 = 18.5;
const HIGHEST_NORMAL_BMI: f64 = 24.9;
const MIDDLE_OF_NORMAL_BMI: f64 = 21.7;
const FAT_FLOOR_G_PER_KG: f64 = 0.8;
const FIBRE_G_A_DAY: f64 = 25.0;
const METRES_A_STEP: f64 = 0.75;

fn healthy_fat_ratio(gender: Gender, age: u32) -> (f64, f64) {
    match (gender, age) {
        (Gender::Male, ..=39) => (0.08, 0.20),
        (Gender::Male, 40..=59) => (0.11, 0.22),
        (Gender::Male, _) => (0.13, 0.25),
        (Gender::Female, ..=39) => (0.21, 0.33),
        (Gender::Female, 40..=59) => (0.23, 0.34),
        (Gender::Female, _) => (0.24, 0.36),
    }
}

fn by_rhythm(rhythm: Rhythm) -> (f64, f64, (f64, f64)) {
    match rhythm {
        Rhythm::Sedentary => (0.83, 4000.0, (600.0, 60.0)),
        Rhythm::LightlyActive => (1.1, 6250.0, (480.0, 120.0)),
        Rhythm::ModeratelyActive => (1.4, 8750.0, (360.0, 240.0)),
        Rhythm::VeryActive => (1.7, 11250.0, (240.0, 360.0)),
        Rhythm::ExtremelyActive => (2.0, 14000.0, (120.0, 480.0)),
    }
}

fn to_the_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn to_the_hundredth(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn to_the_thousandth(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn born(birthday: &str, gender: Gender, height_cm: f64) -> Profile {
        Profile {
            birthday: day(birthday),
            gender,
            height_cm,
            neck_cm: 38.0,
            hip_cm: (gender == Gender::Female).then_some(98.0),
        }
    }

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn a_healthy_mass_spans_the_who_normal_bmi_at_the_height_its_goal_at_the_middle() {
        let advised = recommended(
            &born("1985-04-12", Gender::Male, 180.0),
            day("2026-10-11"),
            Some(71.7),
            Rhythm::ModeratelyActive,
        );

        assert!(about(advised.healthy_mass_kg.0, 18.5 * 1.8 * 1.8));
        assert!(about(advised.healthy_mass_kg.1, 24.9 * 1.8 * 1.8));
        assert!(about(advised.goal_mass_kg, 21.7 * 1.8 * 1.8));
    }

    #[test]
    fn a_healthy_fat_follows_gallagher_for_the_age_and_gender_its_goal_at_the_middle() {
        let fat_of = |birthday, gender| {
            let advised = recommended(
                &born(birthday, gender, 170.0),
                day("2026-10-11"),
                None,
                Rhythm::Sedentary,
            );
            (advised.healthy_fat_ratio, advised.goal_fat_ratio)
        };

        assert_eq!(fat_of("1985-04-12", Gender::Male), ((0.11, 0.22), 0.165));
        assert_eq!(fat_of("2001-06-01", Gender::Female), ((0.21, 0.33), 0.27));
        assert_eq!(fat_of("1960-01-01", Gender::Male), ((0.13, 0.25), 0.19));
        assert_eq!(fat_of("2010-01-01", Gender::Male), ((0.08, 0.20), 0.14));
    }

    #[test]
    fn protein_rises_with_the_rhythm_from_the_who_safe_level_to_the_issn_upper_bound() {
        let proteins: Vec<f64> = RHYTHMS
            .iter()
            .map(|rhythm| {
                recommended(
                    &born("1985-04-12", Gender::Male, 180.0),
                    day("2026-10-11"),
                    Some(80.0),
                    *rhythm,
                )
                .fuel
                .protein_g_per_kg
            })
            .collect();

        assert_eq!(proteins, vec![0.83, 1.1, 1.4, 1.7, 2.0]);
    }

    #[test]
    fn fat_has_a_practical_floor_and_fibre_spreads_25_g_over_the_body_mass() {
        let advised = recommended(
            &born("1985-04-12", Gender::Male, 180.0),
            day("2026-10-11"),
            Some(80.0),
            Rhythm::Sedentary,
        );
        let unweighed = recommended(
            &born("1985-04-12", Gender::Male, 180.0),
            day("2026-10-11"),
            None,
            Rhythm::Sedentary,
        );

        assert_eq!(advised.fuel.fat_g_per_kg, 0.8);
        assert_eq!(advised.fuel.fibre_g_per_kg, 0.31);
        assert_eq!(unweighed.fuel.fibre_g_per_kg, 0.36);
    }

    #[test]
    fn drinks_follow_efsa_for_the_gender() {
        let drinks_of = |gender| {
            recommended(
                &born("1985-04-12", gender, 170.0),
                day("2026-10-11"),
                None,
                Rhythm::Sedentary,
            )
            .hydrate_base_ml
        };

        assert_eq!(
            (drinks_of(Gender::Male), drinks_of(Gender::Female)),
            (2000.0, 1600.0)
        );
    }

    #[test]
    fn the_usual_day_sits_less_and_moves_more_as_the_rhythm_rises() {
        let days: Vec<Volitional> = RHYTHMS
            .iter()
            .map(|rhythm| {
                recommended(
                    &born("1985-04-12", Gender::Male, 180.0),
                    day("2026-10-11"),
                    None,
                    *rhythm,
                )
                .usual_day
            })
            .collect();

        assert_eq!(
            days,
            vec![
                Volitional {
                    sitting_min: 600.0,
                    standing_min: 60.0,
                    moving_km: 3.0
                },
                Volitional {
                    sitting_min: 480.0,
                    standing_min: 120.0,
                    moving_km: 4.7
                },
                Volitional {
                    sitting_min: 360.0,
                    standing_min: 240.0,
                    moving_km: 6.6
                },
                Volitional {
                    sitting_min: 240.0,
                    standing_min: 360.0,
                    moving_km: 8.4
                },
                Volitional {
                    sitting_min: 120.0,
                    standing_min: 480.0,
                    moving_km: 10.5
                },
            ]
        );
    }
}
