use chrono::NaiveDate;

use crate::composition::{Composition, Tapes, body_composition};
use crate::measurement::Measurement;
use crate::profile::Profile;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Day {
    pub day: NaiveDate,
    pub body_mass_kg: f64,
    pub waist_cm: Option<f64>,
    pub composition: Option<Composition>,
    pub age: Option<u32>,
}

pub fn ledger(measurements: &[Measurement], profiles: &[(NaiveDate, Profile)]) -> Vec<Day> {
    measurements
        .iter()
        .map(|measured| {
            let profile = profiles.first().map(|(_, profile)| profile);
            let composition = profile
                .zip(measured.waist_cm)
                .and_then(|(profile, waist_cm)| {
                    body_composition(
                        measured.body_mass_kg,
                        profile.height_cm,
                        Tapes::Male {
                            neck_cm: profile.neck_cm,
                            waist_cm,
                        },
                    )
                });
            Day {
                day: measured.day,
                body_mass_kg: measured.body_mass_kg,
                waist_cm: measured.waist_cm,
                composition,
                age: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn a_man(neck_cm: f64) -> Profile {
        Profile {
            birthday: day("1985-04-12"),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm,
            hip_cm: None,
        }
    }

    fn measured(date: &str, body_mass_kg: f64, waist_cm: Option<f64>) -> Measurement {
        Measurement {
            day: day(date),
            body_mass_kg,
            waist_cm,
        }
    }

    fn fat_ratio_reads_about(day: &Day, expected: f64) -> bool {
        day.composition
            .is_some_and(|composition| (composition.fat_ratio - expected).abs() < 1e-6)
    }

    #[test]
    fn a_day_with_a_waist_reads_its_composition_from_the_profile_in_force() {
        let days = ledger(
            &[measured("2026-10-10", 80.0, Some(80.0))],
            &[(day("2026-03-01"), a_man(38.5))],
        );

        assert!(
            days.first()
                .is_some_and(|day| fat_ratio_reads_about(day, 0.116245))
        );
    }
}
