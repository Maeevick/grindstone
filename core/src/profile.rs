use chrono::NaiveDate;

use crate::gender::Gender;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    pub birthday: NaiveDate,
    pub gender: Gender,
    pub height_cm: f64,
    pub neck_cm: f64,
    pub hip_cm: Option<f64>,
}
