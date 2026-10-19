use serde::{Deserialize, Serialize};

use crate::heading::STABLE_SHARE_A_WEEK;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Judgement {
    Holding,
    VeryGood,
    Good,
    Stable,
    WrongWay,
    GainingTooFast,
    LosingTooFast,
    UnderHealthyMass,
    UnderHealthyFat,
}

const GAIN_SHARE_A_WEEK_FROM: f64 = 0.0025;
const AT_THE_AIM_SHARE: f64 = 0.01;
const GAIN_SHARE_A_WEEK_AT_MOST: f64 = 0.005;

/// A week of the body mass, as it is judged.
pub struct Week {
    pub per_week: f64,
    pub body_mass_kg: f64,
    pub fat_ratio: Option<f64>,
    pub healthy_mass_from_kg: Option<f64>,
    pub healthy_fat_ratio_from: Option<f64>,
    pub safe_loss_kg: f64,
    pub aim_kg: Option<f64>,
}

/// Health first: a loss under the healthy mass or fat, or past the safe loss, is dangerous whatever the aim.
/// Towards a higher aim, a quarter to half a percent of the body mass a week builds more than it stores:
/// Iraki J. et al. (2019), *Nutrition recommendations for bodybuilders in the off-season*, Sports 7(7):154.
pub fn judgement_of(week: &Week) -> Judgement {
    let losing = week.per_week < -STABLE_SHARE_A_WEEK * week.body_mass_kg;
    let gaining = week.per_week > STABLE_SHARE_A_WEEK * week.body_mass_kg;
    let towards_a_lower_aim = week.aim_kg.is_some_and(|aim| aim < week.body_mass_kg);
    let towards_a_higher_aim = week.aim_kg.is_some_and(|aim| aim > week.body_mass_kg);
    let gained_share = week.per_week / week.body_mass_kg;
    let at_the_aim = week
        .aim_kg
        .is_some_and(|aim| (week.body_mass_kg - aim).abs() <= AT_THE_AIM_SHARE * aim);
    if losing
        && week
            .healthy_mass_from_kg
            .is_some_and(|healthy| week.body_mass_kg < healthy)
    {
        Judgement::UnderHealthyMass
    } else if losing
        && week
            .healthy_fat_ratio_from
            .zip(week.fat_ratio)
            .is_some_and(|(healthy, fat_ratio)| fat_ratio < healthy)
    {
        Judgement::UnderHealthyFat
    } else if -week.per_week > week.safe_loss_kg {
        Judgement::LosingTooFast
    } else if towards_a_lower_aim && -week.per_week >= week.safe_loss_kg / 2.0 {
        Judgement::VeryGood
    } else if towards_a_lower_aim && losing {
        Judgement::Good
    } else if towards_a_lower_aim && gaining || towards_a_higher_aim && losing {
        Judgement::WrongWay
    } else if towards_a_higher_aim && gained_share > GAIN_SHARE_A_WEEK_AT_MOST {
        Judgement::GainingTooFast
    } else if towards_a_higher_aim && gained_share >= GAIN_SHARE_A_WEEK_FROM {
        Judgement::VeryGood
    } else if towards_a_higher_aim && gaining {
        Judgement::Good
    } else if at_the_aim && !losing && !gaining {
        Judgement::Holding
    } else {
        Judgement::Stable
    }
}
