use chrono::{Days, NaiveDate};

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// Each reading's day with the mean of the readings in the week ending on it, that day and the six before.
pub fn seven_day_trend(readings: &[(NaiveDate, f64)]) -> Vec<(NaiveDate, f64)> {
    let mut in_order = readings.to_vec();
    in_order.sort_by_key(|(day, _)| *day);
    in_order
        .iter()
        .map(|(day, _)| {
            let from = in_order.partition_point(|(read_on, _)| *read_on + Days::new(6) < *day);
            let to = in_order.partition_point(|(read_on, _)| read_on <= day);
            let the_weeks_readings: Vec<f64> =
                in_order[from..to].iter().map(|(_, value)| *value).collect();
            (*day, mean(&the_weeks_readings))
        })
        .collect()
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
    fn a_reading_more_than_seven_days_old_no_longer_weighs_on_the_trend() {
        let readings = [(day("2026-10-01"), 74.0), (day("2026-10-08"), 72.0)];

        assert!(reads_about(
            &seven_day_trend(&readings),
            &[(day("2026-10-01"), 74.0), (day("2026-10-08"), 72.0)]
        ));
    }

    #[test]
    fn a_reading_six_days_old_still_weighs_on_the_trend() {
        let readings = [(day("2026-10-01"), 74.0), (day("2026-10-07"), 72.0)];

        assert!(reads_about(
            &seven_day_trend(&readings),
            &[(day("2026-10-01"), 74.0), (day("2026-10-07"), 73.0)]
        ));
    }

    #[test]
    fn the_trend_follows_the_days_whatever_the_order_of_the_readings() {
        let readings = [(day("2026-10-02"), 72.6), (day("2026-10-01"), 73.0)];

        assert!(reads_about(
            &seven_day_trend(&readings),
            &[(day("2026-10-01"), 73.0), (day("2026-10-02"), 72.8)]
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
