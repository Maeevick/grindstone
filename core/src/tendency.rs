use serde::{Deserialize, Serialize};

use crate::goals::Goals;
use crate::ledger::Day;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heading {
    Toward,
    Away,
    Steady,
    Unaimed,
    /// Falling, yet still above its floor.
    AboveItsFloor,
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

fn ends(days: &[Day], measure: fn(&Day) -> Option<f64>) -> Option<(f64, f64)> {
    let mut measured: Vec<_> = days
        .iter()
        .filter_map(|day| measure(day).map(|value| (day.day, value)))
        .collect();
    measured.sort_by_key(|(day, _)| *day);
    match measured.as_slice() {
        [(_, first), .., (_, last)] => Some((*first, *last)),
        _ => None,
    }
}

fn closing_on(target: f64, (first, last): (f64, f64)) -> Heading {
    if (last - target).abs() < (first - target).abs() {
        Heading::Toward
    } else {
        Heading::Away
    }
}

fn change((first, last): (f64, f64), shown_per_unit: f64, heading: Heading) -> Change {
    let by = last - first;
    Change {
        by,
        heading: if (by * shown_per_unit * 10.0).round() == 0.0 {
            Heading::Steady
        } else {
            heading
        },
    }
}

/// How each measure moved from its first day to its last, and whether that heads for the goals.
/// A change that shows as nothing at the tenth is steady. The waist has no goal of its own: it
/// heads the way the fat ratio must go to reach its goal.
pub fn tendency_of(days: &[Day], goals: Option<&Goals>) -> Tendency {
    let mass = ends(days, |day| Some(day.body_mass_kg));
    let waist = ends(days, |day| day.waist_cm);
    let fat = ends(days, |day| day.composition.map(|split| split.fat_ratio));
    let lean = ends(days, |day| day.composition.map(|split| split.lean_mass_kg));
    let aimed = |target: Option<f64>, ends: (f64, f64)| {
        target.map_or(Heading::Unaimed, |target| closing_on(target, ends))
    };
    let waist_heading = |(first, last): (f64, f64)| match (goals, fat) {
        (Some(goals), Some((fat_first, _)))
            if (goals.fat_ratio - fat_first) * (last - first) > 0.0 =>
        {
            Heading::Toward
        }
        (Some(_), Some(_)) => Heading::Away,
        _ => Heading::Unaimed,
    };
    Tendency {
        body_mass: mass.map(|ends| {
            change(
                ends,
                1.0,
                aimed(goals.map(|goals| goals.body_mass_kg), ends),
            )
        }),
        waist: waist.map(|ends| change(ends, 1.0, waist_heading(ends))),
        fat_ratio: fat
            .map(|ends| change(ends, 100.0, aimed(goals.map(|goals| goals.fat_ratio), ends))),
        lean_mass: lean.map(|ends| {
            change(
                ends,
                1.0,
                aimed(
                    goals.map(|goals| goals.body_mass_kg * (1.0 - goals.fat_ratio)),
                    ends,
                ),
            )
        }),
    }
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

    #[test]
    fn a_measure_ending_as_far_past_its_goal_as_it_began_short_of_it_heads_away() {
        let week = [
            measured("2026-10-05", 76.0, 90.0, 0.2, 64.0),
            measured("2026-10-11", 74.0, 90.0, 0.2, 64.0),
        ];

        assert_eq!(
            moved(tendency_of(&week, Some(&aiming_at(75.0, 0.15))).body_mass),
            Some((-2.0, Heading::Away))
        );
    }

    #[test]
    fn a_waist_moving_while_the_fat_ratio_sits_on_its_goal_heads_away() {
        let week = [
            measured("2026-10-05", 80.0, 90.0, 0.15, 68.0),
            measured("2026-10-11", 80.0, 89.0, 0.15, 68.0),
        ];

        assert_eq!(
            moved(tendency_of(&week, Some(&aiming_at(75.0, 0.15))).waist),
            Some((-1.0, Heading::Away))
        );
    }
}
