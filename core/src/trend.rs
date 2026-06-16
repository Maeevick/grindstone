use chrono::NaiveDate;

pub fn seven_day_trend(readings: &[(NaiveDate, f64)]) -> Vec<(NaiveDate, f64)> {
    readings.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn reads_about(trend: &[(NaiveDate, f64)], expected: &[(NaiveDate, f64)]) -> bool {
        trend.len() == expected.len()
            && trend
                .iter()
                .zip(expected)
                .all(|((day, mean), (expected_day, expected_mean))| {
                    day == expected_day && (mean - expected_mean).abs() < 1e-9
                })
    }

    #[test]
    fn a_days_trend_averages_the_readings_of_the_days_up_to_it() {
        let readings = [
            (day("2026-10-01"), 73.0),
            (day("2026-10-02"), 72.6),
            (day("2026-10-04"), 72.2),
        ];

        assert!(reads_about(
            &seven_day_trend(&readings),
            &[
                (day("2026-10-01"), 73.0),
                (day("2026-10-02"), 72.8),
                (day("2026-10-04"), 72.6),
            ]
        ));
    }

    #[test]
    fn a_lone_reading_is_its_own_trend() {
        assert_eq!(
            seven_day_trend(&[(day("2026-10-01"), 72.4)]),
            vec![(day("2026-10-01"), 72.4)]
        );
    }
}
