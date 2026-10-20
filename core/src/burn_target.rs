use chrono::NaiveDate;

use crate::kept::{InForce, Kept, in_force};

const MODERATE_MINUTES_A_WEEK: f64 = 150.0;
const MODERATE_MET: f64 = 4.0;
const DAYS_A_WEEK: f64 = 7.0;
const MINUTES_AN_HOUR: f64 = 60.0;

/// The net kcal a day's workouts aim to burn: the target kept from the latest day up to it, its own kcal or
/// the recommendation. Before any, and wherever the recommendation is kept, the World Health Organization's
/// 150 minutes a week of moderate activity, spread over the days at 4 METs, less the rest it replaces, on the
/// body mass in force; without a body mass either, nothing to price a workout on.
///
/// Bull F. C. et al. (2020), *World Health Organization 2020 guidelines on physical activity and sedentary
/// behaviour*, Br J Sports Med 54(24):1451-1462.
pub fn target_on(
    day: NaiveDate,
    targets: &[(NaiveDate, Kept<f64>)],
    body_mass_kg: Option<f64>,
) -> Option<InForce<f64>> {
    in_force(
        targets
            .iter()
            .filter(|(from, _)| *from <= day)
            .max_by_key(|(from, _)| *from)
            .map(|(_, kept)| kept),
        || {
            body_mass_kg.map(|kg| {
                (MODERATE_MET - 1.0) * kg * MODERATE_MINUTES_A_WEEK / DAYS_A_WEEK / MINUTES_AN_HOUR
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
    fn without_a_target_kept_a_day_aims_for_whos_150_minutes_a_week_of_moderate_activity_on_its_body_mass()
     {
        assert!(about(
            target_on(day("2026-10-20"), &[], Some(72.0)),
            3.0 * 72.0 * 150.0 / 7.0 / 60.0
        ));
        assert_eq!(target_on(day("2026-10-20"), &[], None), None);
    }

    #[test]
    fn a_target_kept_from_a_day_stands_from_that_day_and_the_default_before_it() {
        let kept = [(day("2026-10-15"), Kept::Own(150.0))];

        assert_eq!(
            target_on(day("2026-10-20"), &kept, None),
            Some(InForce {
                values: 150.0,
                recommended: false
            })
        );
        assert!(about(
            target_on(day("2026-10-10"), &kept, Some(72.0)),
            3.0 * 72.0 * 150.0 / 7.0 / 60.0
        ));
    }
}
