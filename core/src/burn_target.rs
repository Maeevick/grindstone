use chrono::NaiveDate;

use crate::kept::{InForce, Kept, in_force};
use crate::workout::activity_induced_energy_expenditure_from_compendium;

const MODERATE_MINUTES_A_WEEK: f64 = 150.0;
const MODERATE_MET: f64 = 4.0;
const DAYS_A_WEEK: f64 = 7.0;
/// From this day on, the recommendation is a flat 75 kcal a day: the user's choice.
const FLAT_FROM: Option<NaiveDate> = NaiveDate::from_ymd_opt(2026, 11, 15);
const FLAT_KCAL: f64 = 75.0;

/// The net kcal a day's workouts aim to burn: the target kept from the latest day up to it, its own kcal or
/// the recommendation. Before any, and wherever the recommendation is kept: from 2026-11-15, 75 kcal a day, the
/// user's choice; before it, the World Health Organization's 150 minutes a week of moderate activity, spread over
/// the days at 4 METs, less the rest it replaces, on the body mass in force, priced as a workout is, and without a
/// body mass, nothing to price a workout on [20].
pub fn target_on(
    day: NaiveDate,
    targets: &[(NaiveDate, Kept<f64>)],
    body_mass_kg: Option<f64>,
) -> Option<InForce<f64>> {
    in_force(
        targets
            .iter()
            .filter(|(from, _)| *from <= day)
            .max_by_key(|(from, _)| *from),
        || {
            if FLAT_FROM.is_some_and(|flat| day >= flat) {
                return Some(FLAT_KCAL);
            }
            body_mass_kg.map(|kg| {
                activity_induced_energy_expenditure_from_compendium(
                    kg,
                    MODERATE_MET,
                    MODERATE_MINUTES_A_WEEK / DAYS_A_WEEK,
                    0.0,
                )
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn about(actual: Option<InForce<f64>>, expected: f64) -> bool {
        actual.is_some_and(|actual| (actual.values - expected).abs() < 1e-9)
    }

    #[test]
    fn without_a_target_kept_a_day_aims_for_whos_150_minutes_a_week_of_moderate_activity_priced_as_a_workout()
     {
        assert!(about(
            target_on(day("2026-10-20"), &[], Some(72.0)),
            3.0 * 3.5 * 72.0 / 200.0 * 150.0 / 7.0
        ));
        assert_eq!(target_on(day("2026-10-20"), &[], None), None);
    }

    #[test]
    fn from_2026_11_15_without_a_target_kept_a_day_aims_for_75_kcal_weighed_or_not() {
        let aims = [
            target_on(day("2026-11-15"), &[], Some(72.0)),
            target_on(day("2026-11-20"), &[], None),
        ]
        .map(|target| target.map(|target| target.values));

        assert_eq!(aims, [Some(75.0), Some(75.0)]);
    }

    #[test]
    fn a_target_kept_from_a_day_stands_from_that_day_and_the_default_before_it() {
        let kept = [(day("2026-10-15"), Kept::Own(150.0))];

        assert_eq!(
            target_on(day("2026-10-20"), &kept, None),
            Some(InForce {
                values: 150.0,
                recommended: false,
                from: Some(day("2026-10-15")),
            })
        );
        assert!(about(
            target_on(day("2026-10-10"), &kept, Some(72.0)),
            3.0 * 3.5 * 72.0 / 200.0 * 150.0 / 7.0
        ));
    }
}
