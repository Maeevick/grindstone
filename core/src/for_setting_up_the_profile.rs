use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::gender::Gender;
use crate::profile::Profile;
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileForm {
    pub from: String,
    pub birthday: String,
    pub gender: Gender,
    pub height_cm: f64,
    pub neck_cm: f64,
    pub hip_cm: Option<f64>,
}

pub fn set_profile(
    records: &impl ForStoringGatherRecords,
    form: &ProfileForm,
) -> Result<Profile, Failure> {
    Err(Failure::Refused(Refusal::NotADate))
}

pub fn profile_on(
    records: &impl ForStoringGatherRecords,
    day: &str,
) -> Result<Option<Profile>, Failure> {
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
}
