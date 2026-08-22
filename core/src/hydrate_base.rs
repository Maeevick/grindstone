use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::profile::Profile;
use crate::sources::Source;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HydrateBase {
    pub base_ml: f64,
    pub is_default: bool,
}

pub const DEFAULT_BASE_SOURCE: Source = Source {
    authors: "EFSA Panel on Dietetic Products, Nutrition and Allergies",
    year: 2010,
    title: "Scientific Opinion on Dietary Reference Values for water",
    link: "https://doi.org/10.2903/j.efsa.2010.1459",
};

pub fn base_on(
    day: NaiveDate,
    bases: &[(NaiveDate, f64)],
    profiles: &[(NaiveDate, Profile)],
) -> HydrateBase {
    let _ = (day, bases, profiles);
    HydrateBase {
        base_ml: 0.0,
        is_default: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn a_man() -> Profile {
        Profile {
            birthday: day("1985-04-12"),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm: 38.5,
            hip_cm: None,
        }
    }

    fn a_woman() -> Profile {
        Profile {
            birthday: day("1990-06-01"),
            gender: Gender::Female,
            height_cm: 165.0,
            neck_cm: 33.0,
            hip_cm: Some(98.0),
        }
    }

    #[test]
    fn a_base_set_from_a_day_is_in_force_from_that_day_rather_than_the_default() {
        let bases = [(day("2026-09-01"), 2200.0)];

        let base = base_on(day("2026-10-10"), &bases, &[(day("2026-03-01"), a_man())]);

        assert_eq!(
            base,
            HydrateBase {
                base_ml: 2200.0,
                is_default: false,
            }
        );
    }

    #[test]
    fn with_no_base_set_a_man_drinks_four_fifths_of_his_adequate_intake_of_total_water() {
        let base = base_on(day("2026-10-10"), &[], &[(day("2026-03-01"), a_man())]);

        assert_eq!(
            base,
            HydrateBase {
                base_ml: 2000.0,
                is_default: true,
            }
        );
    }

    #[test]
    fn with_no_base_set_a_woman_drinks_four_fifths_of_her_adequate_intake_of_total_water() {
        let base = base_on(day("2026-10-10"), &[], &[(day("2026-03-01"), a_woman())]);

        assert_eq!(
            base,
            HydrateBase {
                base_ml: 1600.0,
                is_default: true,
            }
        );
    }

    #[test]
    fn with_no_base_set_and_no_profile_yet_the_default_is_two_litres() {
        assert_eq!(
            base_on(day("2026-10-10"), &[], &[]),
            HydrateBase {
                base_ml: 2000.0,
                is_default: true,
            }
        );
    }
}
