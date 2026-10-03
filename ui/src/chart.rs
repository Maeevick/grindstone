use chrono::NaiveDate;

fn placed(value: f64, (lowest, highest): (f64, f64), length: f64) -> f64 {
    if highest > lowest {
        (value - lowest) / (highest - lowest) * length
    } else {
        length / 2.0
    }
}

fn range_of(values: impl Iterator<Item = f64> + Clone) -> (f64, f64) {
    (
        values.clone().fold(f64::INFINITY, f64::min),
        values.fold(f64::NEG_INFINITY, f64::max),
    )
}

pub fn timeline_of(
    points: &[(NaiveDate, f64)],
    (first, last): (NaiveDate, NaiveDate),
    width: f64,
    height: f64,
) -> String {
    let days = (first, last);
    let along = |day: NaiveDate| {
        let (from, to) = days;
        placed(
            (day - from).num_days() as f64,
            (0.0, (to - from).num_days() as f64),
            width,
        )
    };
    let values = range_of(points.iter().map(|(_, value)| *value));
    points
        .iter()
        .map(|(day, value)| (along(*day), height - placed(*value, values, height)))
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn scatter_of(pairs: &[(f64, f64)], width: f64, height: f64) -> Vec<(f64, f64)> {
    let xs = range_of(pairs.iter().map(|(x, _)| *x));
    let ys = range_of(pairs.iter().map(|(_, y)| *y));
    pairs
        .iter()
        .map(|(x, y)| (placed(*x, xs, width), height - placed(*y, ys, height)))
        .collect()
}

pub fn dots_of(
    pairs: &[(f64, f64)],
    xs: (f64, f64),
    ys: (f64, f64),
    width: f64,
    height: f64,
) -> Vec<(f64, f64)> {
    pairs
        .iter()
        .map(|(x, y)| (placed(*x, xs, width), height_of(*y, ys, height)))
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gauge {
    pub band_from: f64,
    pub band_to: f64,
    pub optimal_at: f64,
    pub amount_at: f64,
}

pub fn gauge_of(amount: f64, (floor, optimal, cap): (f64, f64, f64), width: f64) -> Gauge {
    let scale = (0.0, (cap * 1.25).max(amount));
    Gauge {
        band_from: placed(floor, scale, width),
        band_to: placed(cap, scale, width),
        optimal_at: placed(optimal, scale, width),
        amount_at: placed(amount, scale, width),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyGauge {
    pub minimum_at: f64,
    pub floor_at: f64,
    pub cap_at: f64,
    pub eaten_at: f64,
}

pub fn energy_gauge_of(
    eaten: f64,
    (minimum, floor, cap): (f64, f64, f64),
    width: f64,
) -> EnergyGauge {
    let spare = 0.1 * (cap - minimum).max(0.1 * cap);
    let scale = (minimum - spare, cap + spare);
    EnergyGauge {
        minimum_at: placed(minimum, scale, width),
        floor_at: placed(floor, scale, width),
        cap_at: placed(cap, scale, width),
        eaten_at: placed(eaten, scale, width).clamp(0.0, width),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis {
    pub lowest: f64,
    pub highest: f64,
    pub step: f64,
}

impl Axis {
    pub fn scale(&self) -> (f64, f64) {
        (self.lowest, self.highest)
    }

    pub fn ticks(&self) -> Vec<f64> {
        let steps = ((self.highest - self.lowest) / self.step).round() as usize;
        (0..=steps)
            .map(|step| self.lowest + step as f64 * self.step)
            .collect()
    }
}

/// An axis over every value, on round ends: about a third of the span, rounded up to 1, 2, 2.5
/// or 5 times a power of ten, makes its step, so one to three ticks fall between its ends.
/// A lone value gets a unit either side, never below nothing when it is not.
pub fn axis_of(values: impl Iterator<Item = f64> + Clone) -> Option<Axis> {
    let (lowest, highest) = range_of(values);
    if !lowest.is_finite() {
        return None;
    }
    let (lowest, highest) = if highest > lowest {
        (lowest, highest)
    } else if lowest >= 0.0 {
        ((lowest - 1.0).max(0.0), highest + 1.0)
    } else {
        (lowest - 1.0, highest + 1.0)
    };
    let rough = (highest - lowest) / 3.0;
    let magnitude = 10_f64.powf(rough.log10().floor());
    let step = [1.0, 2.0, 2.5, 5.0, 10.0]
        .into_iter()
        .find(|factor| factor * magnitude >= rough)
        .unwrap_or(10.0)
        * magnitude;
    Some(Axis {
        lowest: (lowest / step).floor() * step,
        highest: (highest / step).ceil() * step,
        step,
    })
}

pub fn scale_of(values: impl Iterator<Item = f64> + Clone) -> Option<(f64, f64)> {
    let (lowest, highest) = range_of(values);
    if !lowest.is_finite() {
        return None;
    }
    let spare = ((highest - lowest) / 20.0).max(1.0);
    Some((lowest - spare, highest + spare))
}

pub fn height_of(value: f64, scale: (f64, f64), height: f64) -> f64 {
    height - placed(value, scale, height)
}

pub fn line_of(
    points: &[(NaiveDate, f64)],
    (first, last): (NaiveDate, NaiveDate),
    scale: (f64, f64),
    width: f64,
    height: f64,
) -> String {
    let span = (0.0, (last - first).num_days() as f64);
    points
        .iter()
        .map(|(day, value)| {
            (
                placed((*day - first).num_days() as f64, span, width),
                height_of(*value, scale, height),
            )
        })
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn spread(heights: &[f64], gap: f64, height: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..heights.len()).collect();
    order.sort_by(|one, other| heights[*one].total_cmp(&heights[*other]));
    let mut placed_at = heights.to_vec();
    for pair in order.windows(2) {
        placed_at[pair[1]] = placed_at[pair[1]].max(placed_at[pair[0]] + gap);
    }
    if let Some(lowest) = order.last() {
        placed_at[*lowest] = placed_at[*lowest].min(height);
    }
    for pair in order.windows(2).rev() {
        placed_at[pair[0]] = placed_at[pair[0]].min(placed_at[pair[1]] - gap);
    }
    placed_at
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    #[test]
    fn a_timeline_spreads_its_days_over_the_span_and_its_values_over_their_own_range() {
        let points = [(day("2026-10-01"), 80.0), (day("2026-10-11"), 78.0)];

        assert_eq!(
            timeline_of(
                &points,
                (day("2026-10-01"), day("2026-10-11")),
                360.0,
                180.0
            ),
            "0.0,0.0 360.0,180.0"
        );
    }

    #[test]
    fn a_flat_timeline_or_a_single_day_runs_through_the_middle() {
        let flat = [(day("2026-10-01"), 80.0), (day("2026-10-11"), 80.0)];
        let single = [(day("2026-10-05"), 80.0)];

        assert_eq!(
            timeline_of(&flat, (day("2026-10-01"), day("2026-10-11")), 360.0, 180.0),
            "0.0,90.0 360.0,90.0"
        );
        assert_eq!(
            timeline_of(
                &single,
                (day("2026-10-05"), day("2026-10-05")),
                360.0,
                180.0
            ),
            "180.0,90.0"
        );
    }

    #[test]
    fn a_scatter_spreads_its_pairs_over_both_ranges_and_a_flat_side_sits_in_the_middle() {
        let pairs = [(1.0, 10.0), (3.0, 30.0), (2.0, 10.0)];

        assert_eq!(
            scatter_of(&pairs, 360.0, 180.0),
            vec![(0.0, 180.0), (360.0, 0.0), (180.0, 180.0)]
        );
        assert_eq!(scatter_of(&[(1.0, 5.0)], 360.0, 180.0), vec![(180.0, 90.0)]);
    }

    #[test]
    fn a_gauge_runs_from_nothing_to_a_quarter_past_the_cap_and_places_the_band_the_optimal_and_the_amount()
     {
        assert_eq!(
            gauge_of(1000.0, (1800.0, 2000.0, 2200.0), 275.0),
            Gauge {
                band_from: 180.0,
                band_to: 220.0,
                optimal_at: 200.0,
                amount_at: 100.0,
            }
        );
    }

    #[test]
    fn an_amount_beyond_the_scale_stretches_it_to_stay_in_sight() {
        let gauge = gauge_of(4000.0, (1800.0, 2000.0, 2200.0), 400.0);

        assert_eq!((gauge.amount_at, gauge.optimal_at), (400.0, 200.0));
    }

    #[test]
    fn a_scale_spans_every_value_with_a_twentieth_to_spare_either_side() {
        assert_eq!(scale_of([60.0, 80.0, 70.0].into_iter()), Some((59.0, 81.0)));
        assert_eq!(scale_of([70.0].into_iter()), Some((69.0, 71.0)));
        assert_eq!(scale_of(std::iter::empty()), None);
    }

    #[test]
    fn a_line_runs_from_the_first_day_to_the_last_on_its_scale() {
        let points = [(day("2026-10-01"), 81.0), (day("2026-10-11"), 59.0)];

        assert_eq!(
            line_of(
                &points,
                (day("2026-10-01"), day("2026-10-11")),
                (59.0, 81.0),
                360.0,
                180.0
            ),
            "0.0,0.0 360.0,180.0"
        );
        assert_eq!(height_of(70.0, (59.0, 81.0), 180.0), 90.0);
    }

    #[test]
    fn labels_too_close_are_spread_apart_and_kept_inside_the_height() {
        assert_eq!(
            spread(&[100.0, 95.0, 30.0, 178.0], 12.0, 180.0),
            vec![107.0, 95.0, 30.0, 178.0]
        );
        assert_eq!(spread(&[175.0, 178.0], 12.0, 180.0), vec![168.0, 180.0]);
    }

    #[test]
    fn an_energy_gauge_runs_a_tenth_of_its_band_past_the_minimum_and_the_cap() {
        let gauge = energy_gauge_of(2200.0, (1600.0, 2000.0, 2400.0), 960.0);

        for (placed, expected) in [
            (gauge.minimum_at, 80.0),
            (gauge.floor_at, 480.0),
            (gauge.cap_at, 880.0),
            (gauge.eaten_at, 680.0),
        ] {
            assert!((placed - expected).abs() < 1e-9, "{gauge:?}");
        }
    }

    #[test]
    fn what_was_eaten_beyond_the_gauge_stays_on_its_edge() {
        let band = (1600.0, 2000.0, 2400.0);

        assert_eq!(energy_gauge_of(100.0, band, 960.0).eaten_at, 0.0);
        assert_eq!(energy_gauge_of(5000.0, band, 960.0).eaten_at, 960.0);
    }

    #[test]
    fn a_band_closed_on_its_minimum_sits_in_the_middle_of_a_gauge_a_tenth_of_it_wide() {
        let closed = energy_gauge_of(1800.0, (1800.0, 1800.0, 1800.0), 960.0);

        assert_eq!(
            (closed.minimum_at, closed.cap_at, closed.eaten_at),
            (480.0, 480.0, 480.0)
        );
    }

    fn axis(values: &[f64]) -> Option<Axis> {
        axis_of(values.iter().copied())
    }

    #[test]
    fn an_axis_spans_every_value_on_round_ends_with_one_to_three_round_ticks_between() {
        let masses = axis(&[63.0, 80.2, 70.0]).expect("an axis");
        let energies = axis(&[1900.0, 2650.0, 2199.0]).expect("an axis");
        let burns = axis(&[0.0, 120.0, 252.0]).expect("an axis");

        assert_eq!(masses.ticks(), vec![60.0, 70.0, 80.0, 90.0]);
        assert_eq!(
            energies.ticks(),
            vec![1750.0, 2000.0, 2250.0, 2500.0, 2750.0]
        );
        assert_eq!(burns.ticks(), vec![0.0, 100.0, 200.0, 300.0]);
        assert_eq!(burns.scale(), (0.0, 300.0));
    }

    #[test]
    fn a_single_value_sits_in_the_middle_of_a_narrow_axis_and_no_value_draws_none() {
        let single = axis(&[70.0]).expect("an axis");

        assert_eq!(single.ticks().first(), Some(&69.0));
        assert_eq!(single.ticks().last(), Some(&71.0));
        assert_eq!(axis(&[]), None);
    }

    #[test]
    fn a_lone_value_below_nothing_gets_a_unit_either_side_and_one_at_nothing_none_below_it() {
        assert_eq!(
            axis(&[-0.5]).map(|negative| negative.ticks()),
            Some(vec![-2.0, -1.0, 0.0, 1.0])
        );
        assert_eq!(
            axis(&[0.0]).map(|nothing| nothing.ticks()),
            Some(vec![0.0, 0.5, 1.0])
        );
    }

    #[test]
    fn a_scatters_dots_sit_on_the_scales_of_their_two_axes() {
        assert_eq!(
            dots_of(
                &[(80.0, 2400.0), (80.5, 2600.0)],
                (80.0, 81.0),
                (2400.0, 2800.0),
                360.0,
                180.0
            ),
            vec![(0.0, 180.0), (180.0, 90.0)]
        );
    }
}
