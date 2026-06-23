use chrono::NaiveDate;

use crate::age::age_on;
use crate::composition::{Composition, Tapes, body_composition};
use crate::gender::Gender;
use crate::measurement::Measurement;
use crate::profile::Profile;
use crate::version::version_in_force_on;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Day {
    pub day: NaiveDate,
    pub body_mass_kg: f64,
    pub waist_cm: Option<f64>,
    pub composition: Option<Composition>,
    pub age: Option<u32>,
}

fn composition_carrying(lean_mass_kg: f64, body_mass_kg: f64) -> Option<Composition> {
    let fat_mass_kg = body_mass_kg - lean_mass_kg;
    (fat_mass_kg > 0.0).then_some(Composition {
        fat_ratio: fat_mass_kg / body_mass_kg,
        fat_mass_kg,
        lean_mass_kg,
    })
}

fn tapes(profile: &Profile, waist_cm: f64) -> Option<Tapes> {
    match profile.gender {
        Gender::Male => Some(Tapes::Male {
            neck_cm: profile.neck_cm,
            waist_cm,
        }),
        Gender::Female => profile.hip_cm.map(|hip_cm| Tapes::Female {
            neck_cm: profile.neck_cm,
            waist_cm,
            hip_cm,
        }),
    }
}

fn composition_read_from_the_tapes(
    measured: &Measurement,
    profiles: &[(NaiveDate, Profile)],
) -> Option<Composition> {
    let profile = version_in_force_on(profiles, measured.day)?;
    body_composition(
        measured.body_mass_kg,
        profile.height_cm,
        tapes(profile, measured.waist_cm?)?,
    )
}

pub fn ledger(measurements: &[Measurement], profiles: &[(NaiveDate, Profile)]) -> Vec<Day> {
    let mut measurements = measurements.to_vec();
    measurements.sort_by_key(|measured| measured.day);
    let earliest_lean_mass_kg = measurements
        .iter()
        .find_map(|measured| composition_read_from_the_tapes(measured, profiles))
        .map(|composition| composition.lean_mass_kg);
    measurements
        .iter()
        .scan(earliest_lean_mass_kg, |last_lean_mass_kg, measured| {
            let read_from_the_tapes = composition_read_from_the_tapes(measured, profiles);
            if let Some(composition) = read_from_the_tapes {
                *last_lean_mass_kg = Some(composition.lean_mass_kg);
            }
            let composition = read_from_the_tapes.or_else(|| {
                last_lean_mass_kg.and_then(|lean_mass_kg| {
                    composition_carrying(lean_mass_kg, measured.body_mass_kg)
                })
            });
            Some(Day {
                day: measured.day,
                body_mass_kg: measured.body_mass_kg,
                waist_cm: measured.waist_cm,
                composition,
                age: version_in_force_on(profiles, measured.day)
                    .map(|profile| age_on(profile.birthday, measured.day)),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn a_profile_changed_from_a_date_leaves_the_days_before_it_as_they_were() {
        let profiles = [
            (day("2026-03-01"), a_man(38.5)),
            (day("2026-10-05"), a_man(36.0)),
        ];

        let days = ledger(
            &[
                measured("2026-10-01", 80.0, Some(80.0)),
                measured("2026-10-10", 80.0, Some(80.0)),
            ],
            &profiles,
        );

        assert!(
            days.first()
                .is_some_and(|day| fat_ratio_reads_about(day, 0.116245))
        );
        assert!(
            days.last()
                .is_some_and(|day| !fat_ratio_reads_about(day, 0.116245))
        );
    }

    fn lean_mass_of(day: &Day) -> Option<f64> {
        day.composition.map(|composition| composition.lean_mass_kg)
    }

    #[test]
    fn a_day_without_a_waist_carries_the_last_lean_mass() {
        let days = ledger(
            &[
                measured("2026-10-01", 80.0, Some(80.0)),
                measured("2026-10-02", 79.5, None),
            ],
            &[(day("2026-03-01"), a_man(38.5))],
        );

        let carried = days
            .last()
            .and_then(|day| day.composition)
            .expect("a carried composition");
        assert_eq!(
            Some(carried.lean_mass_kg),
            days.first().and_then(lean_mass_of)
        );
        assert!((carried.fat_mass_kg + carried.lean_mass_kg - 79.5).abs() < 1e-9);
        assert!((carried.fat_ratio - carried.fat_mass_kg / 79.5).abs() < 1e-9);
    }

    #[test]
    fn the_days_before_the_first_waist_carry_the_earliest_lean_mass() {
        let days = ledger(
            &[
                measured("2026-09-30", 80.2, None),
                measured("2026-10-01", 80.0, Some(80.0)),
            ],
            &[(day("2026-03-01"), a_man(38.5))],
        );

        assert!(days.first().and_then(lean_mass_of).is_some());
        assert_eq!(
            days.first().and_then(lean_mass_of),
            days.last().and_then(lean_mass_of)
        );
    }

    #[test]
    fn a_mass_below_the_carried_lean_mass_reads_no_composition_rather_than_a_negative_fat() {
        let days = ledger(
            &[
                measured("2026-10-01", 80.0, Some(80.0)),
                measured("2026-10-02", 70.0, None),
            ],
            &[(day("2026-03-01"), a_man(38.5))],
        );

        assert_eq!(days.last().and_then(|day| day.composition), None);
    }

    #[test]
    fn a_day_reads_its_age_from_the_birthday_in_its_profile() {
        let days = ledger(
            &[measured("2026-10-10", 80.0, None)],
            &[(day("2026-03-01"), a_man(38.5))],
        );

        assert_eq!(days.first().and_then(|day| day.age), Some(41));
    }

    #[test]
    fn a_womans_day_reads_her_hip_from_her_profile() {
        let a_woman = Profile {
            birthday: day("1990-06-01"),
            gender: Gender::Female,
            height_cm: 165.0,
            neck_cm: 33.0,
            hip_cm: Some(98.0),
        };

        let days = ledger(
            &[measured("2026-10-10", 65.0, Some(75.0))],
            &[(day("2026-03-01"), a_woman)],
        );

        assert!(
            days.first()
                .is_some_and(|day| fat_ratio_reads_about(day, 0.284350))
        );
    }

    #[test]
    fn the_ledger_lists_its_days_in_their_order_whatever_the_order_measured() {
        let days = ledger(
            &[
                measured("2026-10-02", 79.5, None),
                measured("2026-10-01", 80.0, Some(80.0)),
            ],
            &[(day("2026-03-01"), a_man(38.5))],
        );

        let listed: Vec<NaiveDate> = days.iter().map(|day| day.day).collect();
        assert_eq!(listed, vec![day("2026-10-01"), day("2026-10-02")]);
    }
}
