use serde::{Deserialize, Serialize};

use crate::verdict::{Verdict, verdict_of};

/// A logged day's amount of a measure, and the band it was judged against.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Banded {
    pub amount: f64,
    pub floor: f64,
    pub ceiling: f64,
}

/// A measure over a week's logged days: what a day took on average, against the average of their bands, and
/// how many of those days stood in theirs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Averaged {
    pub per_day: f64,
    pub floor: f64,
    pub ceiling: f64,
    pub in_band: usize,
}

impl Averaged {
    pub fn verdict(&self) -> Verdict {
        verdict_of(self.per_day, self.floor, self.ceiling)
    }

    /// How far the average runs from its band, a day: under its floor short, past its ceiling over, nothing
    /// within it.
    pub fn gap(&self) -> Option<f64> {
        match self.verdict() {
            Verdict::Low => Some(self.per_day - self.floor),
            Verdict::High => Some(self.per_day - self.ceiling),
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
        ceiling: mean(|day| day.ceiling),
        in_band: days
            .iter()
            .filter(|day| verdict_of(day.amount, day.floor, day.ceiling) == Verdict::InBand)
            .count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn banded(amount: f64) -> Banded {
        Banded {
            amount,
            floor: 1800.0,
            ceiling: 2400.0,
        }
    }

    fn a_day_of(per_day: f64) -> Averaged {
        Averaged {
            per_day,
            floor: 1800.0,
            ceiling: 2400.0,
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
                ceiling: 2400.0,
                in_band: 1,
            })
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
