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
    let birthday =
        NaiveDate::parse_from_str(birthday, "%Y-%m-%d").map_err(|_| Refusal::NotADate)?;
    if !(height_cm.is_finite() && height_cm > 0.0) {
        return Err(Refusal::NotAHeight);
    }
    if !(neck_cm.is_finite() && neck_cm > 0.0) {
        return Err(Refusal::NotANeck);
    }
    Ok(Profile {
        birthday,
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

    #[test]
    fn a_height_of_nothing_or_no_number_at_all_is_refused() {
        for cm in [0.0, -180.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                profile("1985-04-12", Gender::Male, cm, 38.5, None),
                Err(Refusal::NotAHeight),
                "{cm}"
            );
        }
    }

    #[test]
    fn a_neck_of_nothing_or_no_number_at_all_is_refused() {
        for cm in [0.0, -38.5, f64::NAN, f64::INFINITY] {
            assert_eq!(
                profile("1985-04-12", Gender::Male, 180.0, cm, None),
                Err(Refusal::NotANeck),
                "{cm}"
            );
        }
    }
}
