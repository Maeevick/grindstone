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

    #[test]
    fn a_lone_reading_is_its_own_trend() {
        assert_eq!(
            seven_day_trend(&[(day("2026-10-01"), 72.4)]),
            vec![(day("2026-10-01"), 72.4)]
        );
    }
}
