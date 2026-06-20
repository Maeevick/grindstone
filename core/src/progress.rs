use chrono::{Days, NaiveDate};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Progress {
    pub per_week: f64,
    pub share_per_week: f64,
    pub reached_on: Option<NaiveDate>,
}

fn least_squares_slope_per_day(trend: &[(NaiveDate, f64)]) -> f64 {
    let first_day = trend.first().map_or(NaiveDate::MIN, |(day, _)| *day);
    let points: Vec<(f64, f64)> = trend
        .iter()
        .map(|(day, value)| ((*day - first_day).num_days() as f64, *value))
        .collect();
    let count = points.len() as f64;
    let sum_x: f64 = points.iter().map(|(x, _)| x).sum();
    let sum_y: f64 = points.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
    let sum_xx: f64 = points.iter().map(|(x, _)| x * x).sum();
    (count * sum_xy - sum_x * sum_y) / (count * sum_xx - sum_x * sum_x)
}

fn day_the_line_crosses(
    target: f64,
    (latest_day, latest): (NaiveDate, f64),
    per_day: f64,
) -> Option<NaiveDate> {
    let days_to_go = (target - latest) / per_day;
    (per_day != 0.0 && days_to_go >= 0.0)
        .then(|| latest_day.checked_add_days(Days::new(days_to_go.ceil() as u64)))
        .flatten()
}

pub fn progress_towards(target: f64, trend: &[(NaiveDate, f64)]) -> Option<Progress> {
    if trend.len() < 2 {
        return None;
    }
    let latest = *trend.last()?;
    let per_day = least_squares_slope_per_day(trend);
    Some(Progress {
        per_week: per_day * 7.0,
        share_per_week: per_day * 7.0 / latest.1,
        reached_on: day_the_line_crosses(target, latest, per_day),
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
