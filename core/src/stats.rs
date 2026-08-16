use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::figures::Figure;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub first: f64,
    pub last: f64,
    pub mean: f64,
    pub lowest: f64,
    pub highest: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Measure {
    BodyMass,
    Waist,
    FatRatio,
    LeanMass,
    Eaten,
    Balance,
    Protein,
    Carbs,
    Fat,
    Fibre,
    Burnt,
}

pub const MEASURES: [Measure; 11] = [
    Measure::BodyMass,
    Measure::Waist,
    Measure::FatRatio,
    Measure::LeanMass,
    Measure::Eaten,
    Measure::Balance,
    Measure::Protein,
    Measure::Carbs,
    Measure::Fat,
    Measure::Fibre,
    Measure::Burnt,
];

pub fn figure_of(measure: Measure, value: f64) -> Figure {
    match measure {
        Measure::BodyMass | Measure::LeanMass => Figure::Mass(value),
        Measure::Waist => Figure::Length(value),
        Measure::FatRatio => Figure::Ratio(value),
        Measure::Eaten | Measure::Balance | Measure::Burnt => Figure::Energy(value),
        Measure::Protein | Measure::Carbs | Measure::Fat | Measure::Fibre => Figure::Grams(value),
    }
}

const FEWEST_PAIRS_TO_CORRELATE: usize = 3;

pub fn summary(points: &[(NaiveDate, f64)]) -> Option<Summary> {
    let (_, first) = *points.first()?;
    let (_, last) = *points.last()?;
    let values = points.iter().map(|(_, value)| *value);
    Some(Summary {
        first,
        last,
        mean: values.clone().sum::<f64>() / points.len() as f64,
        lowest: values.clone().fold(f64::INFINITY, f64::min),
        highest: values.fold(f64::NEG_INFINITY, f64::max),
    })
}

pub fn paired(one: &[(NaiveDate, f64)], other: &[(NaiveDate, f64)]) -> Vec<(f64, f64)> {
    one.iter()
        .filter_map(|(day, value)| {
            other
                .iter()
                .find(|(other_day, _)| other_day == day)
                .map(|(_, other_value)| (*value, *other_value))
        })
        .collect()
}

pub fn correlation(pairs: &[(f64, f64)]) -> Option<f64> {
    if pairs.len() < FEWEST_PAIRS_TO_CORRELATE {
        return None;
    }
    let count = pairs.len() as f64;
    let mean_x = pairs.iter().map(|(x, _)| x).sum::<f64>() / count;
    let mean_y = pairs.iter().map(|(_, y)| y).sum::<f64>() / count;
    let deviations: Vec<(f64, f64)> = pairs
        .iter()
        .map(|(x, y)| (x - mean_x, y - mean_y))
        .collect();
    let covariance: f64 = deviations.iter().map(|(dx, dy)| dx * dy).sum();
    let spread_x: f64 = deviations.iter().map(|(dx, _)| dx * dx).sum();
    let spread_y: f64 = deviations.iter().map(|(_, dy)| dy * dy).sum();
    let spread = (spread_x * spread_y).sqrt();
    (spread > 0.0).then(|| covariance / spread)
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

    #[test]
    fn each_measure_shows_in_its_own_figure() {
        let figures: Vec<Figure> = MEASURES
            .iter()
            .map(|measure| figure_of(*measure, 1.0))
            .collect();

        assert_eq!(
            figures,
            vec![
                Figure::Mass(1.0),
                Figure::Length(1.0),
                Figure::Ratio(1.0),
                Figure::Mass(1.0),
                Figure::Energy(1.0),
                Figure::Energy(1.0),
                Figure::Grams(1.0),
                Figure::Grams(1.0),
                Figure::Grams(1.0),
                Figure::Grams(1.0),
                Figure::Energy(1.0),
            ]
        );
    }
}
