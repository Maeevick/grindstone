use chrono::NaiveDate;

use crate::age::age_on;
use crate::gender::Gender;
use crate::ledger::Day;
use crate::profile::Profile;
use crate::version::version_in_force_on;

pub struct Body {
    pub mass_kg: f64,
    pub lean_kg: f64,
    pub fat_kg: f64,
    pub age: u32,
    pub gender: Gender,
}

/// The body on a day: its last weigh-in and its last lean mass read from the tapes, the first of each
/// standing in before the records begin, under the profile in force. The ledger is in day order, as `ledger`
/// builds it.
pub fn body_on(day: NaiveDate, ledger: &[Day], profiles: &[(NaiveDate, Profile)]) -> Option<Body> {
    let profile = version_in_force_on(profiles, day)?;
    let so_far = &ledger[..ledger.partition_point(|measured| measured.day <= day)];
    let weighed = so_far.last().or(ledger.first())?;
    let lean_kg = so_far
        .iter()
        .rev()
        .find_map(|measured| measured.composition)
        .or_else(|| ledger.iter().find_map(|measured| measured.composition))?
        .lean_mass_kg;
    Some(Body {
        mass_kg: weighed.body_mass_kg,
        lean_kg,
        fat_kg: (weighed.body_mass_kg - lean_kg).max(0.0),
        age: age_on(profile.birthday, day),
        gender: profile.gender,
    })
}
