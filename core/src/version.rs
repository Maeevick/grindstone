use chrono::NaiveDate;

pub fn version_in_force_on<V>(versions: &[(NaiveDate, V)], day: NaiveDate) -> Option<&V> {
    versions
        .iter()
        .filter(|(from, _)| *from <= day)
        .max_by_key(|(from, _)| *from)
        .map(|(_, version)| version)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn heights() -> Vec<(NaiveDate, f64)> {
        vec![
            (day("2026-06-22"), 180.5),
            (day("2026-03-01"), 180.0),
            (day("2026-09-01"), 181.0),
        ]
    }

    #[test]
    fn a_version_is_in_force_from_its_first_day_until_the_next_one() {
        assert_eq!(version_in_force_on(&heights(), day("2026-06-22")), Some(&180.5));
        assert_eq!(version_in_force_on(&heights(), day("2026-08-31")), Some(&180.5));
    }
}
