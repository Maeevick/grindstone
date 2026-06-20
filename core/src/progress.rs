use chrono::{Days, NaiveDate};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
    pub per_week: f64,
    pub share_per_week: f64,
    pub reached_on: Option<NaiveDate>,
}

pub fn progress_towards(target: f64, trend: &[(NaiveDate, f64)]) -> Option<Progress> {
    if trend.len() < 2 {
        return None;
    }
    let first_day = trend.first()?.0;
    let (latest_day, latest) = *trend.last()?;
    let points: Vec<(f64, f64)> = trend
        .iter()
        .map(|(day, value)| ((*day - first_day).num_days() as f64, *value))
        .collect();
    let count = points.len() as f64;
    let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
    let sum_xx: f64 = points.iter().map(|(x, _)| x * x).sum();
    let per_day = (count * sum_xy - sum_x * sum_y) / (count * sum_xx - sum_x * sum_x);
    Some(Progress {
        per_week: per_day * 7.0,
        share_per_week: per_day * 7.0 / latest,
        reached_on: (per_day != 0.0 && (target - latest) / per_day >= 0.0)
            .then(|| {
                latest_day.checked_add_days(Days::new(((target - latest) / per_day).ceil() as u64))
            })
            .flatten(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn losing_half_a_kilo_a_week() -> Vec<(NaiveDate, f64)> {
        vec![
            (day("2026-09-01"), 80.0),
            (day("2026-09-08"), 79.5),
            (day("2026-09-15"), 79.0),
        ]
    }

    fn about(actual: Option<f64>, expected: f64) -> bool {
        actual.is_some_and(|actual| (actual - expected).abs() < 1e-9)
    }

    #[test]
    fn a_steady_loss_of_half_a_kilo_a_week_reads_minus_half_a_kilo_a_week() {
        let progress = progress_towards(75.0, &losing_half_a_kilo_a_week());

        assert!(about(progress.map(|progress| progress.per_week), -0.5));
    }

    #[test]
    fn the_weekly_share_is_the_rate_over_the_latest_trend() {
        let progress = progress_towards(75.0, &losing_half_a_kilo_a_week());

        assert!(about(
            progress.map(|progress| progress.share_per_week),
            -0.5 / 79.0
        ));
    }

    #[test]
    fn a_target_ahead_of_the_trend_is_reached_where_its_line_crosses_it() {
        let progress = progress_towards(78.0, &losing_half_a_kilo_a_week());

        assert_eq!(
            progress.and_then(|progress| progress.reached_on),
            Some(day("2026-09-29"))
        );
    }

    #[test]
    fn a_flat_trend_reaches_no_target() {
        let flat = [
            (day("2026-09-01"), 79.0),
            (day("2026-09-08"), 79.0),
            (day("2026-09-15"), 79.0),
        ];

        assert_eq!(
            progress_towards(78.0, &flat).map(|progress| progress.reached_on),
            Some(None)
        );
    }

    #[test]
    fn a_trend_moving_away_from_its_target_reaches_none() {
        let progress = progress_towards(81.0, &losing_half_a_kilo_a_week());

        assert_eq!(progress.map(|progress| progress.reached_on), Some(None));
    }

    #[test]
    fn a_single_point_gives_no_progress() {
        assert_eq!(progress_towards(78.0, &[(day("2026-09-15"), 79.0)]), None);
    }
}
