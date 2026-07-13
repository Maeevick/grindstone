use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::gender::Gender;
use crate::refusal::Refusal;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub birthday: NaiveDate,
    pub gender: Gender,
    pub height_cm: f64,
    pub neck_cm: f64,
    pub hip_cm: Option<f64>,
}

fn is_a_figure_above_nothing(figure: f64) -> bool {
    figure.is_finite() && figure > 0.0
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
    if !is_a_figure_above_nothing(height_cm) {
        return Err(Refusal::NotAHeight);
    }
    if !is_a_figure_above_nothing(neck_cm) {
        return Err(Refusal::NotANeck);
    }
    if gender == Gender::Female && !hip_cm.is_some_and(is_a_figure_above_nothing) {
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

    #[test]
    fn a_womans_profile_keeps_her_birthday_height_neck_and_hip() {
        assert_eq!(
            profile("1990-06-01", Gender::Female, 165.0, 33.0, Some(98.0)),
            Ok(Profile {
                birthday: NaiveDate::from_ymd_opt(1990, 6, 1).expect("a real day"),
                gender: Gender::Female,
                height_cm: 165.0,
                neck_cm: 33.0,
                hip_cm: Some(98.0),
            })
        );
    }
}
