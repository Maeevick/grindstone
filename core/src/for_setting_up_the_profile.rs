use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::gender::Gender;
use crate::goals::Goals;
use crate::profile::{Profile, profile};
use crate::refusal::Refusal;
use crate::version::version_in_force_on;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileForm {
    pub from: String,
    pub birthday: String,
    pub gender: Gender,
    pub height_cm: f64,
    pub neck_cm: f64,
    pub hip_cm: Option<f64>,
}

fn day(date: &str) -> Result<NaiveDate, Failure> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Failure::Refused(Refusal::NotADate))
}

pub fn set_profile(
    records: &impl ForStoringGatherRecords,
    form: &ProfileForm,
) -> Result<Profile, Failure> {
    let from = day(&form.from)?;
    let profile = profile(
        &form.birthday,
        form.gender,
        form.height_cm,
        form.neck_cm,
        form.hip_cm,
    )
    .map_err(Failure::Refused)?;
    records.save_profile(from, &profile)?;
    Ok(profile)
}

pub fn profile_on(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Option<Profile>, Failure> {
    let day = day(date)?;
    let gathered = records.gather_records()?;
    Ok(version_in_force_on(&gathered.profiles, day).copied())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoalsForm {
    pub from: String,
    pub body_mass_kg: f64,
    pub fat_percentage: f64,
}

pub fn set_goals(
    records: &impl ForStoringGatherRecords,
    form: &GoalsForm,
) -> Result<Goals, Failure> {
    Err(Failure::Refused(Refusal::NotADate))
}

pub fn goals_on(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Option<Goals>, Failure> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::GatherRecordsInMemory;

    fn a_mans_form(from: &str) -> ProfileForm {
        ProfileForm {
            from: from.into(),
            birthday: "1985-04-12".into(),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm: 38.5,
            hip_cm: None,
        }
    }

    #[test]
    fn a_profile_set_from_a_day_is_in_force_from_that_day() {
        let records = GatherRecordsInMemory::default();

        let set = set_profile(&records, &a_mans_form("2026-10-10"));

        assert_eq!(profile_on(&records, "2026-10-12"), set.map(Some));
    }

    #[test]
    fn goals_set_from_a_day_are_in_force_from_that_day() {
        let records = GatherRecordsInMemory::default();

        let set = set_goals(
            &records,
            &GoalsForm {
                from: "2026-10-10".into(),
                body_mass_kg: 70.0,
                fat_percentage: 10.0,
            },
        );

        assert_eq!(goals_on(&records, "2026-10-12"), set.map(Some));
    }
}
