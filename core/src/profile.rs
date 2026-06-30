use chrono::NaiveDate;

use crate::gender::Gender;
use crate::refusal::Refusal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    pub birthday: NaiveDate,
    pub gender: Gender,
    pub height_cm: f64,
    pub neck_cm: f64,
    pub hip_cm: Option<f64>,
}

pub fn profile(
    birthday: &str,
    gender: Gender,
    height_cm: f64,
    neck_cm: f64,
    hip_cm: Option<f64>,
) -> Result<Profile, Refusal> {
    Ok(Profile {
        birthday: NaiveDate::MIN,
        gender,
        height_cm,
        neck_cm,
        hip_cm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_birthday_the_calendar_does_not_have_is_refused() {
        assert_eq!(
            profile("1985-02-30", Gender::Male, 180.0, 38.5, None),
            Err(Refusal::NotADate)
        );
    }
}
