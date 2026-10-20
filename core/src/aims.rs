use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::gather_records::GatherRecords;
use crate::heading::Aim;
use crate::kept::Kept;
use crate::recommendations::recommended;
use crate::version::version_in_force_on;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aims {
    pub body_mass_kg: f64,
    pub lean_mass_kg: f64,
    pub recommended: bool,
}

impl Aims {
    /// The body mass to reach.
    pub fn body_mass(&self) -> Aim {
        Aim::Reach(self.body_mass_kg)
    }

    /// The lean mass to stay above: a floor, not a value to reach.
    pub fn lean_mass(&self) -> Aim {
        Aim::Floor(self.lean_mass_kg)
    }

    /// The fat mass the aims leave: the body mass less the lean.
    pub fn fat_mass_kg(&self) -> f64 {
        self.body_mass_kg - self.lean_mass_kg
    }

    /// The fat ratio the aims leave: the share of the body mass the lean does not take.
    pub fn fat_ratio(&self) -> f64 {
        1.0 - self.lean_mass_kg / self.body_mass_kg
    }
}

/// The body mass and the lean mass a day aims for: the goals in force, else the healthy weight and fat
/// recommended for the profile in force (WHO's body mass index, Gallagher's fat), the lean mass being what
/// their fat ratio leaves of their body mass.
pub fn aims_on(day: NaiveDate, gathered: &GatherRecords) -> Option<Aims> {
    let aiming = |body_mass_kg: f64, fat_ratio: f64, recommended: bool| Aims {
        body_mass_kg,
        lean_mass_kg: body_mass_kg * (1.0 - fat_ratio),
        recommended,
    };
    version_in_force_on(&gathered.goals, day)
        .and_then(Kept::own)
        .map(|goals| aiming(goals.body_mass_kg, goals.fat_ratio, false))
        .or_else(|| {
            version_in_force_on(&gathered.profiles, day).map(|profile| {
                let advised = recommended(profile, day, None);
                aiming(advised.goal_mass_kg, advised.goal_fat_ratio, true)
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;
    use crate::goals::Goals;
    use crate::profile::Profile;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn a_man() -> GatherRecords {
        GatherRecords {
            profiles: vec![(
                day("2026-03-01"),
                Profile {
                    birthday: day("1985-04-12"),
                    gender: Gender::Male,
                    height_cm: 180.0,
                    neck_cm: 38.5,
                    hip_cm: None,
                },
            )],
            ..GatherRecords::default()
        }
    }

    #[test]
    fn a_day_aims_for_the_goals_in_force_and_the_lean_mass_they_leave() {
        let gathered = GatherRecords {
            goals: vec![(
                day("2026-03-01"),
                Kept::Own(Goals {
                    body_mass_kg: 70.0,
                    fat_ratio: 0.12,
                }),
            )],
            ..a_man()
        };

        let aims = aims_on(day("2026-10-10"), &gathered).expect("aims");

        assert!((aims.body_mass_kg - 70.0).abs() < 1e-9);
        assert!((aims.lean_mass_kg - 70.0 * 0.88).abs() < 1e-9);
        assert!(!aims.recommended);
    }

    #[test]
    fn the_aims_leave_a_fat_mass_and_a_fat_ratio() {
        let aims = Aims {
            body_mass_kg: 70.0,
            lean_mass_kg: 61.6,
            recommended: false,
        };

        assert!((aims.fat_mass_kg() - 8.4).abs() < 1e-9);
        assert!((aims.fat_ratio() - 0.12).abs() < 1e-9);
    }

    #[test]
    fn without_goals_a_day_aims_for_whos_healthy_weight_and_fat_for_its_profile_and_nothing_without_one()
     {
        let advised =
            crate::recommendations::recommended(&a_man().profiles[0].1, day("2026-10-10"), None);

        assert_eq!(
            (
                aims_on(day("2026-10-10"), &a_man()),
                aims_on(day("2026-10-10"), &GatherRecords::default())
            ),
            (
                Some(Aims {
                    body_mass_kg: advised.goal_mass_kg,
                    lean_mass_kg: advised.goal_mass_kg * (1.0 - advised.goal_fat_ratio),
                    recommended: true,
                }),
                None
            )
        );
    }
}
