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
    if gender == Gender::Female && !hip_cm.is_some_and(|cm| cm.is_finite() && cm > 0.0) {
        return Err(Refusal::NotAHip);
    }
    Ok(Profile {
        birthday,
        gender,
        height_cm,
        neck_cm,
        hip_cm: hip_cm.filter(|_| gender == Gender::Female),
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

    #[test]
    fn a_womans_profile_without_a_hip_or_with_one_no_body_could_have_is_refused() {
        for hip_cm in [None, Some(0.0), Some(-98.0), Some(f64::NAN)] {
            assert_eq!(
                profile("1990-06-01", Gender::Female, 165.0, 33.0, hip_cm),
                Err(Refusal::NotAHip),
                "{hip_cm:?}"
            );
        }
    }

    #[test]
    fn a_mans_profile_leaves_out_any_hip_his_formula_never_reads() {
        assert_eq!(
            profile("1985-04-12", Gender::Male, 180.0, 38.5, Some(98.0)).map(|kept| kept.hip_cm),
            Ok(None)
        );
    }
}
