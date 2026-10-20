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
/// Within 1% of its aim, a body mass stands at it.
pub const AT_THE_AIM_SHARE: f64 = 0.01;
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

#[cfg(test)]
mod tests {
    use super::*;

    const A_BODY_KG: f64 = 64.0;

    fn a_week(per_week: f64, aim_kg: Option<f64>) -> Week {
        Week {
            per_week,
            body_mass_kg: A_BODY_KG,
            fat_ratio: None,
            healthy_mass_from_kg: None,
            healthy_fat_ratio_from: None,
            safe_loss_kg: 10.0,
            aim_kg,
        }
    }

    #[test]
    fn a_loss_or_a_gain_of_exactly_015_percent_a_week_is_no_move_yet() {
        let a_lower_aim = Some(50.0);

        assert_eq!(
            [
                judgement_of(&a_week(-STABLE_SHARE_A_WEEK * A_BODY_KG, a_lower_aim)),
                judgement_of(&a_week(STABLE_SHARE_A_WEEK * A_BODY_KG, a_lower_aim)),
            ],
            [Judgement::Stable, Judgement::Stable]
        );
    }

    #[test]
    fn a_body_at_its_aim_aims_neither_lower_nor_higher_whichever_way_it_moves() {
        let a_move = 2.0 * STABLE_SHARE_A_WEEK * A_BODY_KG;

        assert_eq!(
            [
                judgement_of(&a_week(a_move, Some(A_BODY_KG))),
                judgement_of(&a_week(-a_move, Some(A_BODY_KG))),
            ],
            [Judgement::Stable, Judgement::Stable]
        );
    }

    #[test]
    fn a_loss_under_the_healthy_mass_or_fat_is_judged_by_the_aim_the_healthy_ranges_being_the_profiles_word()
     {
        let losing_towards_a_lower_aim = Week {
            safe_loss_kg: 0.47,
            ..a_week(-0.3, Some(60.0))
        };

        assert_eq!(
            [
                judgement_of(&Week {
                    healthy_mass_from_kg: Some(A_BODY_KG + 1.0),
                    ..losing_towards_a_lower_aim
                }),
                judgement_of(&Week {
                    fat_ratio: Some(0.107),
                    healthy_fat_ratio_from: Some(0.11),
                    ..losing_towards_a_lower_aim
                }),
            ],
            [Judgement::VeryGood, Judgement::VeryGood]
        );
    }

    #[test]
    fn losing_right_at_the_healthy_mass_or_fat_is_not_yet_under_them() {
        let losing = a_week(-1.0, None);

        assert_eq!(
            [
                judgement_of(&Week {
                    healthy_mass_from_kg: Some(A_BODY_KG),
                    ..losing
                }),
                judgement_of(&Week {
                    fat_ratio: Some(0.125),
                    healthy_fat_ratio_from: Some(0.125),
                    ..losing
                }),
            ],
            [Judgement::Stable, Judgement::Stable]
        );
    }

    #[test]
    fn losing_right_the_safe_loss_towards_a_lower_aim_is_very_good_not_yet_too_fast() {
        assert_eq!(
            judgement_of(&Week {
                safe_loss_kg: 0.5,
                ..a_week(-0.5, Some(50.0))
            }),
            Judgement::VeryGood
        );
    }

    #[test]
    fn gaining_right_half_a_percent_a_week_towards_a_higher_aim_is_very_good_not_yet_too_fast() {
        assert_eq!(
            judgement_of(&a_week(GAIN_SHARE_A_WEEK_AT_MOST * A_BODY_KG, Some(80.0))),
            Judgement::VeryGood
        );
    }
}
