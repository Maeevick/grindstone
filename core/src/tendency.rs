use serde::{Deserialize, Serialize};

use crate::goals::Goals;
use crate::ledger::Day;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heading {
    Toward,
    Away,
    Steady,
    Unaimed,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub by: f64,
    pub heading: Heading,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Tendency {
    pub body_mass: Option<Change>,
    pub waist: Option<Change>,
    pub fat_ratio: Option<Change>,
    pub lean_mass: Option<Change>,
}

pub fn tendency_of(_days: &[Day], _goals: Option<&Goals>) -> Tendency {
    Tendency::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::Composition;
    use chrono::NaiveDate;

    fn measured(
        date: &str,
        body_mass_kg: f64,
        waist_cm: f64,
        fat_ratio: f64,
        lean_mass_kg: f64,
    ) -> Day {
        Day {
            day: NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day"),
            body_mass_kg,
            waist_cm: Some(waist_cm),
            composition: Some(Composition {
                fat_ratio,
                fat_mass_kg: body_mass_kg - lean_mass_kg,
                lean_mass_kg,
            }),
            age: Some(41),
        }
    }

    fn aiming_at(body_mass_kg: f64, fat_ratio: f64) -> Goals {
        Goals {
            body_mass_kg,
            fat_ratio,
        }
    }

    fn moved(change: Option<Change>) -> Option<(f64, Heading)> {
        change.map(|change| ((change.by * 1000.0).round() / 1000.0, change.heading))
    }

    #[test]
    fn a_measure_closing_on_its_goal_heads_toward_it_and_one_widening_the_gap_away() {
        let week = [
            measured("2026-10-11", 79.0, 89.0, 0.185, 64.4),
            measured("2026-10-05", 80.0, 90.0, 0.2, 64.0),
        ];

        let tendency = tendency_of(&week, Some(&aiming_at(75.0, 0.15)));

        assert_eq!(moved(tendency.body_mass), Some((-1.0, Heading::Toward)));
        assert_eq!(moved(tendency.fat_ratio), Some((-0.015, Heading::Toward)));
        assert_eq!(moved(tendency.lean_mass), Some((0.4, Heading::Away)));
    }

    #[test]
    fn the_waist_heads_the_way_the_fat_ratio_must_go_to_reach_its_goal() {
        let shrinking = [
            measured("2026-10-05", 80.0, 90.0, 0.2, 64.0),
            measured("2026-10-11", 80.0, 89.0, 0.2, 64.0),
        ];

        assert_eq!(
            moved(tendency_of(&shrinking, Some(&aiming_at(75.0, 0.15))).waist),
            Some((-1.0, Heading::Toward))
        );
        assert_eq!(
            moved(tendency_of(&shrinking, Some(&aiming_at(85.0, 0.25))).waist),
            Some((-1.0, Heading::Away))
        );
    }

    #[test]
    fn a_change_that_shows_as_nothing_at_the_tenth_is_steady_goal_or_not() {
        let week = [
            measured("2026-10-05", 80.0, 90.0, 0.2, 64.0),
            measured("2026-10-11", 80.04, 90.0, 0.2004, 63.96),
        ];

        let tendency = tendency_of(&week, Some(&aiming_at(75.0, 0.15)));

        assert_eq!(moved(tendency.body_mass), Some((0.04, Heading::Steady)));
        assert_eq!(moved(tendency.waist), Some((0.0, Heading::Steady)));
        assert_eq!(moved(tendency.fat_ratio), Some((0.0, Heading::Steady)));
        assert_eq!(moved(tendency.lean_mass), Some((-0.04, Heading::Steady)));
    }

    #[test]
    fn without_goals_a_change_aims_nowhere_and_a_single_day_has_no_tendency() {
        let week = [
            measured("2026-10-05", 80.0, 90.0, 0.2, 64.0),
            measured("2026-10-11", 79.0, 89.0, 0.19, 64.0),
        ];

        let aimless = tendency_of(&week, None);
        let alone = tendency_of(&week[..1], Some(&aiming_at(75.0, 0.15)));

        assert_eq!(moved(aimless.body_mass), Some((-1.0, Heading::Unaimed)));
        assert_eq!(moved(aimless.waist), Some((-1.0, Heading::Unaimed)));
        assert_eq!(alone, Tendency::default());
    }

    #[test]
    fn days_without_a_waist_or_a_composition_are_skipped_for_those_measures() {
        let week = [
            measured("2026-10-05", 80.0, 90.0, 0.2, 64.0),
            Day {
                waist_cm: None,
                composition: None,
                ..measured("2026-10-11", 79.0, 0.0, 0.0, 0.0)
            },
        ];

        let tendency = tendency_of(&week, Some(&aiming_at(75.0, 0.15)));

        assert_eq!(moved(tendency.body_mass), Some((-1.0, Heading::Toward)));
        assert_eq!(
            (tendency.waist, tendency.fat_ratio, tendency.lean_mass),
            (None, None, None)
        );
    }
}
