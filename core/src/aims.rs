use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::gather_records::GatherRecords;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aims {
    pub body_mass_kg: f64,
    pub lean_mass_kg: f64,
    pub recommended: bool,
}

pub fn aims_on(_day: NaiveDate, _gathered: &GatherRecords) -> Option<Aims> {
    None
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
                Goals {
                    body_mass_kg: 70.0,
                    fat_ratio: 0.12,
                },
            )],
            ..a_man()
        };

        let aims = aims_on(day("2026-10-10"), &gathered).expect("aims");

        assert!((aims.body_mass_kg - 70.0).abs() < 1e-9);
        assert!((aims.lean_mass_kg - 70.0 * 0.88).abs() < 1e-9);
        assert!(!aims.recommended);
    }
}
