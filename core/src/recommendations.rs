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

/// The recommendations for a profile on a day.
///
/// - Healthy mass: the World Health Organization's normal body mass index, 18.5 to 24.9,
///   at the profile's height; the goal sits at its middle, 21.7.
///   WHO (2000), *Obesity: preventing and managing the global epidemic*, Technical Report Series 894.
/// - Healthy fat: the percentage body fat predicted at a BMI of 18.5 and 25 for the age and gender,
///   Gallagher D. et al. (2000), *Am J Clin Nutr* 72(3):694–701, table 4; the goal sits at its middle.
/// - Protein: 1.6 g/kg, where the gains in lean mass from training level off: Morton R. W. et al.
///   (2018), *Br J Sports Med* 52(6):376–384, 1.62 g/kg (95% CI 1.03 to 2.20); within the
///   1.4 to 2.0 g/kg of Jäger R. et al. (2017), *J Int Soc Sports Nutr* 14:20.
/// - Fat: 0.8 g/kg as a practical floor; the day's band also asks 20 to 33.3% of the energy eaten,
///   within EFSA's 20 to 35% (2010) and near WHO's 30% at most (2023).
/// - Fibre: 25 g a day, WHO (2023), *Carbohydrate intake for adults and children*, spread per kilo.
/// - Water from drinks: EFSA's adequate intake of total water (2010), four fifths of it from drinks.
/// - The usual day: sitting 8 hours at most, past which the least active die sooner, Ekelund U.
///   et al. (2016), *Lancet* 388(10051):1302–1310; standing 2 hours, the first step of
///   Buckley J. P. et al. (2015), *Br J Sports Med* 49(21):1357–1362; and the steps a day past
///   which the risk of dying stops falling, 8000 under 60 and 6000 from 60, Paluch A. E. et al.
///   (2022), *Lancet Public Health* 7(3):e219–e228, at about 0.75 m a step.
pub fn recommended(
    profile: &Profile,
    day: NaiveDate,
    body_mass_kg: Option<f64>,
) -> Recommendations {
    let height_m = profile.height_cm / 100.0;
    let squared = height_m * height_m;
    let goal_mass_kg = MIDDLE_OF_NORMAL_BMI * squared;
    let age = age_on(profile.birthday, day);
    let healthy_fat_ratio = healthy_fat_ratio(profile.gender, age);
    Recommendations {
        healthy_mass_kg: (LOWEST_NORMAL_BMI * squared, HIGHEST_NORMAL_BMI * squared),
        goal_mass_kg,
        healthy_fat_ratio,
        goal_fat_ratio: to_the_thousandth((healthy_fat_ratio.0 + healthy_fat_ratio.1) / 2.0),
        fuel: FuelTargets {
            protein_g_per_kg: PROTEIN_G_PER_KG,
            fat_g_per_kg: FAT_FLOOR_G_PER_KG,
            fibre_g_per_kg: to_the_hundredth(FIBRE_G_A_DAY / body_mass_kg.unwrap_or(goal_mass_kg)),
        },
        hydrate_base_ml: default_base_ml(Some(profile.gender)),
        usual_day: Volitional {
            sitting_min: SITTING_MIN,
            standing_min: STANDING_MIN,
            moving_km: to_the_tenth(steps_a_day(age) * METRES_A_STEP / 1000.0),
        },
    }
}

const LOWEST_NORMAL_BMI: f64 = 18.5;
const HIGHEST_NORMAL_BMI: f64 = 24.9;
const MIDDLE_OF_NORMAL_BMI: f64 = 21.7;
const PROTEIN_G_PER_KG: f64 = 1.6;
const FAT_FLOOR_G_PER_KG: f64 = 0.8;
const FIBRE_G_A_DAY: f64 = 25.0;
const SITTING_MIN: f64 = 480.0;
const STANDING_MIN: f64 = 120.0;
const METRES_A_STEP: f64 = 0.75;

fn steps_a_day(age: u32) -> f64 {
    if age < 60 { 8000.0 } else { 6000.0 }
}

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
        );

        assert!(about(advised.healthy_mass_kg.0, 18.5 * 1.8 * 1.8));
        assert!(about(advised.healthy_mass_kg.1, 24.9 * 1.8 * 1.8));
        assert!(about(advised.goal_mass_kg, 21.7 * 1.8 * 1.8));
    }

    #[test]
    fn a_healthy_fat_follows_gallagher_for_the_age_and_gender_its_goal_at_the_middle() {
        let fat_of = |birthday, gender| {
            let advised = recommended(&born(birthday, gender, 170.0), day("2026-10-11"), None);
            (advised.healthy_fat_ratio, advised.goal_fat_ratio)
        };

        assert_eq!(fat_of("1985-04-12", Gender::Male), ((0.11, 0.22), 0.165));
        assert_eq!(fat_of("2001-06-01", Gender::Female), ((0.21, 0.33), 0.27));
        assert_eq!(fat_of("1960-01-01", Gender::Male), ((0.13, 0.25), 0.19));
        assert_eq!(fat_of("2010-01-01", Gender::Male), ((0.08, 0.20), 0.14));
        assert_eq!(fat_of("1980-01-01", Gender::Female), ((0.23, 0.34), 0.285));
        assert_eq!(fat_of("1950-01-01", Gender::Female), ((0.24, 0.36), 0.3));
    }

    #[test]
    fn protein_sits_where_the_gains_in_lean_mass_level_off_for_those_who_train() {
        let advised = recommended(
            &born("1985-04-12", Gender::Male, 180.0),
            day("2026-10-11"),
            Some(80.0),
        );

        assert_eq!(advised.fuel.protein_g_per_kg, 1.6);
    }

    #[test]
    fn fat_has_a_practical_floor_and_fibre_spreads_25_g_over_the_body_mass() {
        let advised = recommended(
            &born("1985-04-12", Gender::Male, 180.0),
            day("2026-10-11"),
            Some(80.0),
        );
        let unweighed = recommended(
            &born("1985-04-12", Gender::Male, 180.0),
            day("2026-10-11"),
            None,
        );

        assert_eq!(advised.fuel.fat_g_per_kg, 0.8);
        assert_eq!(advised.fuel.fibre_g_per_kg, 0.31);
        assert_eq!(unweighed.fuel.fibre_g_per_kg, 0.36);
    }

    #[test]
    fn drinks_follow_efsa_for_the_gender() {
        let drinks_of = |gender| {
            recommended(&born("1985-04-12", gender, 170.0), day("2026-10-11"), None).hydrate_base_ml
        };

        assert_eq!(
            (drinks_of(Gender::Male), drinks_of(Gender::Female)),
            (2000.0, 1600.0)
        );
    }

    #[test]
    fn the_usual_day_sits_eight_hours_at_most_stands_two_and_walks_the_steps_that_pay_off_at_its_age()
     {
        let usual_day_of = |birthday| {
            recommended(
                &born(birthday, Gender::Male, 180.0),
                day("2026-10-11"),
                None,
            )
            .usual_day
        };

        assert_eq!(
            usual_day_of("1985-04-12"),
            Volitional {
                sitting_min: 480.0,
                standing_min: 120.0,
                moving_km: 6.0
            }
        );
        assert_eq!(
            usual_day_of("1960-01-01"),
            Volitional {
                sitting_min: 480.0,
                standing_min: 120.0,
                moving_km: 4.5
            }
        );
    }
}
