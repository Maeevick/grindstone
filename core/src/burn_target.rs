use chrono::NaiveDate;

pub fn target_on(
    _day: NaiveDate,
    _targets: &[(NaiveDate, f64)],
    _body_mass_kg: Option<f64>,
) -> Option<f64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn about(actual: Option<f64>, expected: f64) -> bool {
        actual.is_some_and(|actual| (actual - expected).abs() < 1e-9)
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
}
