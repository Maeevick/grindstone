use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub first: f64,
    pub last: f64,
    pub mean: f64,
    pub lowest: f64,
    pub highest: f64,
}

pub fn summary(points: &[(NaiveDate, f64)]) -> Option<Summary> {
    let _ = points;
    None
}

pub fn paired(one: &[(NaiveDate, f64)], other: &[(NaiveDate, f64)]) -> Vec<(f64, f64)> {
    let _ = (one, other);
    vec![]
}

pub fn correlation(pairs: &[(f64, f64)]) -> Option<f64> {
    let _ = pairs;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    #[test]
    fn a_series_sums_up_into_its_first_last_mean_lowest_and_highest() {
        let points = [
            (day("2026-10-01"), 80.0),
            (day("2026-10-02"), 78.0),
            (day("2026-10-03"), 79.0),
        ];

        assert_eq!(
            summary(&points),
            Some(Summary {
                first: 80.0,
                last: 79.0,
                mean: 79.0,
                lowest: 78.0,
                highest: 80.0,
            })
        );
        assert_eq!(summary(&[]), None);
    }

    #[test]
    fn two_series_pair_up_on_the_days_they_share() {
        let masses = [(day("2026-10-01"), 80.0), (day("2026-10-02"), 79.5)];
        let eaten = [(day("2026-10-02"), 2400.0), (day("2026-10-03"), 2100.0)];

        assert_eq!(paired(&masses, &eaten), vec![(79.5, 2400.0)]);
    }

    #[test]
    fn pairs_moving_together_correlate_at_one_and_against_each_other_at_minus_one() {
        let together = [(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];
        let against = [(1.0, 6.0), (2.0, 4.0), (3.0, 2.0)];

        assert!(correlation(&together).is_some_and(|r| (r - 1.0).abs() < 1e-9));
        assert!(correlation(&against).is_some_and(|r| (r + 1.0).abs() < 1e-9));
    }

    #[test]
    fn fewer_than_three_pairs_or_a_flat_series_correlate_with_nothing() {
        assert_eq!(correlation(&[(1.0, 2.0), (2.0, 4.0)]), None);
        assert_eq!(correlation(&[(1.0, 2.0), (2.0, 2.0), (3.0, 2.0)]), None);
    }
}
