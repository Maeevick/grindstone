use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::gender::Gender;
use crate::goals::{Goals, goals};
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
    let from = day(&form.from)?;
    let goals = goals(form.body_mass_kg, form.fat_percentage).map_err(Failure::Refused)?;
    records.save_goals(from, &goals)?;
    Ok(goals)
}

pub fn goals_on(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Option<Goals>, Failure> {
    let day = day(date)?;
    let gathered = records.gather_records()?;
    Ok(version_in_force_on(&gathered.goals, day).copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{BrokenGatherRecords, GatherRecordsInMemory};

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

    #[test]
    fn a_refused_profile_or_refused_goals_leave_what_was_set_as_it_was() {
        let records = GatherRecordsInMemory::default();
        let refused_profile = ProfileForm {
            height_cm: 0.0,
            ..a_mans_form("2026-10-10")
        };
        let refused_goals = GoalsForm {
            from: "2026-10-10".into(),
            body_mass_kg: 70.0,
            fat_percentage: 100.0,
        };

        assert_eq!(
            set_profile(&records, &refused_profile),
            Err(Failure::Refused(Refusal::NotAHeight))
        );
        assert_eq!(
            set_goals(&records, &refused_goals),
            Err(Failure::Refused(Refusal::NotARatio))
        );
        assert_eq!(profile_on(&records, "2026-10-12"), Ok(None));
        assert_eq!(goals_on(&records, "2026-10-12"), Ok(None));
    }

    #[test]
    fn a_day_the_calendar_lacks_is_refused_whether_setting_or_reading() {
        let records = GatherRecordsInMemory::default();
        let not_a_date = Failure::Refused(Refusal::NotADate);

        assert_eq!(
            set_profile(&records, &a_mans_form("2026-02-30")),
            Err(not_a_date.clone())
        );
        assert_eq!(profile_on(&records, "2026-02-30"), Err(not_a_date.clone()));
        assert_eq!(goals_on(&records, "2026-02-30"), Err(not_a_date));
    }

    #[test]
    fn records_that_cannot_be_written_or_read_come_back_broken() {
        let broken = Failure::Broken("the disk is full".into());
        let goals_form = GoalsForm {
            from: "2026-10-10".into(),
            body_mass_kg: 70.0,
            fat_percentage: 10.0,
        };

        assert_eq!(
            set_profile(&BrokenGatherRecords, &a_mans_form("2026-10-10")),
            Err(broken.clone())
        );
        assert_eq!(
            profile_on(&BrokenGatherRecords, "2026-10-10"),
            Err(broken.clone())
        );
        assert_eq!(
            set_goals(&BrokenGatherRecords, &goals_form),
            Err(broken.clone())
        );
        assert_eq!(goals_on(&BrokenGatherRecords, "2026-10-10"), Err(broken));
    }
}
