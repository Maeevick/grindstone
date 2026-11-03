use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::verdict::{Verdict, verdict_of};

/// A logged day's amount of a measure, and the band it was judged against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Banded {
    pub amount: f64,
    pub floor: f64,
    /// None when nothing is too much.
    pub ceiling: Option<f64>,
    /// The goal painted for the day, none until one is.
    pub goal: Option<f64>,
}

/// A measure over a week's logged days: what a day took on average, against the average of their bands, and
/// how many of those days stood in theirs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Averaged {
    pub per_day: f64,
    pub floor: f64,
    /// None when nothing is too much.
    pub ceiling: Option<f64>,
    /// The goals painted for those days, on average; none when none was.
    pub goal: Option<f64>,
    pub in_band: usize,
}

impl Averaged {
    pub fn verdict(&self) -> Verdict {
        verdict_of(
            self.per_day,
            self.floor,
            self.ceiling.unwrap_or(f64::INFINITY),
        )
    }

    /// How far the average runs from its band, a day: under its floor short, past its ceiling over, nothing
    /// within it.
    pub fn gap(&self) -> Option<f64> {
        match self.verdict() {
            Verdict::Low => Some(self.per_day - self.floor),
            Verdict::High => self.ceiling.map(|ceiling| self.per_day - ceiling),
            Verdict::InBand => None,
        }
    }
}

/// The logged days' amounts and bands, averaged; nothing without a day.
pub fn averaged(days: &[Banded]) -> Option<Averaged> {
    let logged = days.len() as f64;
    let mean = |of: fn(&Banded) -> f64| days.iter().map(of).sum::<f64>() / logged;
    (!days.is_empty()).then(|| Averaged {
        per_day: mean(|day| day.amount),
        floor: mean(|day| day.floor),
        ceiling: days
            .iter()
            .map(|day| day.ceiling)
            .sum::<Option<f64>>()
            .map(|sum| sum / logged),
        goal: None,
        in_band: days
            .iter()
            .filter(|day| {
                verdict_of(day.amount, day.floor, day.ceiling.unwrap_or(f64::INFINITY))
                    == Verdict::InBand
            })
            .count(),
    })
}

/// Every day from the first to the last, oldest first.
pub fn the_days_between(first: NaiveDate, last: NaiveDate) -> Vec<NaiveDate> {
    first.iter_days().take_while(|day| *day <= last).collect()
}

/// The days logged from the first to the last, each once and oldest first, leaving out those from a day on:
/// today, while it is still under way.
pub fn logged_between(
    first: NaiveDate,
    last: NaiveDate,
    left_out_from: NaiveDate,
    logged: impl Iterator<Item = NaiveDate>,
) -> Vec<NaiveDate> {
    let mut week: Vec<NaiveDate> = logged
        .filter(|day| (first..=last).contains(day) && *day < left_out_from)
        .collect();
    week.sort();
    week.dedup();
    week
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn banded(amount: f64) -> Banded {
        Banded {
            amount,
            floor: 1800.0,
            ceiling: Some(2400.0),
            goal: None,
        }
    }

    fn a_day_of(per_day: f64) -> Averaged {
        Averaged {
            per_day,
            floor: 1800.0,
            ceiling: Some(2400.0),
            goal: None,
            in_band: 0,
        }
    }

    #[test]
    fn a_week_averages_its_logged_days_and_their_bands_and_counts_the_days_in_theirs() {
        let week = averaged(&[banded(2000.0), banded(1700.0), banded(2600.0)]);

        assert_eq!(
            week,
            Some(Averaged {
                per_day: 2100.0,
                floor: 1800.0,
                ceiling: Some(2400.0),
                goal: None,
                in_band: 1,
            })
        );
    }

    #[test]
    fn the_days_logged_from_the_first_to_the_last_come_once_oldest_first_those_from_today_on_left_out()
     {
        let logged = [
            "2026-10-10",
            "2026-10-03",
            "2026-10-06",
            "2026-10-04",
            "2026-10-06",
            "2026-10-11",
        ];

        let week = logged_between(
            day("2026-10-04"),
            day("2026-10-10"),
            day("2026-10-10"),
            logged.into_iter().map(day),
        );

        assert_eq!(week, [day("2026-10-04"), day("2026-10-06")]);
    }

    #[test]
    fn the_days_from_the_first_to_the_last_run_oldest_first() {
        assert_eq!(
            the_days_between(day("2026-09-26"), day("2026-10-02")),
            [
                "2026-09-26",
                "2026-09-27",
                "2026-09-28",
                "2026-09-29",
                "2026-09-30",
                "2026-10-01",
                "2026-10-02"
            ]
            .map(day)
        );
    }

    #[test]
    fn a_week_with_nothing_logged_averages_nothing() {
        assert_eq!(averaged(&[]), None);
    }

    #[test]
    fn an_average_under_its_floor_runs_short_past_its_ceiling_over_and_within_its_band_on_course() {
        assert_eq!(
            [
                a_day_of(1700.0).gap(),
                a_day_of(2500.0).gap(),
                a_day_of(2000.0).gap()
            ],
            [Some(-100.0), Some(100.0), None]
        );
    }
}
