use chrono::NaiveDate;
use grind_core::heading::Aim;
use grind_core::judgement::AT_THE_AIM_SHARE;

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

const DAYS_IN_A_WEEK: i64 = 7;

/// The days to date an x axis with: every one over a week at most, else the oldest, the newest
/// and three between, each with where it stands along the width, months instead of days past a year.
pub fn days_along((first, last): (NaiveDate, NaiveDate), width: f64) -> Vec<(f64, String)> {
    let days = (last - first).num_days();
    let format = if days > 365 { "%Y-%m" } else { "%m-%d" };
    let mut offsets: Vec<i64> = if days < DAYS_IN_A_WEEK {
        (0..=days).collect()
    } else {
        (0..=4)
            .map(|quarter| (quarter as f64 * days as f64 / 4.0).round() as i64)
            .collect()
    };
    offsets.dedup();
    offsets
        .into_iter()
        .map(|offset| {
            (
                placed(offset as f64, (0.0, days as f64), width),
                (first + chrono::Days::new(offset as u64))
                    .format(format)
                    .to_string(),
            )
        })
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
    pub ghost_at: Option<f64>,
}

pub fn gauge_of(
    amount: f64,
    (floor, optimal, cap): (f64, f64, f64),
    ghost: Option<f64>,
    width: f64,
) -> Gauge {
    let scale = (0.0, (cap * 1.25).max(amount).max(ghost.unwrap_or(0.0)));
    Gauge {
        band_from: placed(floor, scale, width),
        band_to: placed(cap, scale, width),
        optimal_at: placed(optimal, scale, width),
        amount_at: placed(amount, scale, width),
        ghost_at: ghost.map(|ml| placed(ml, scale, width)),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BurnGauge {
    pub band_from: f64,
    pub target_at: f64,
    pub burnt_at: f64,
    pub ghost_at: Option<f64>,
    pub up_to_kcal: f64,
}

const BURN_GAUGE_KCAL: f64 = 200.0;
const ROOM_PAST_THE_LARGER: f64 = 1.2;
const ROUNDED_UP_TO_KCAL: f64 = 50.0;

/// A burn on a line from nothing: 200 kcal by default, stretched a fifth past the largest of the target, the
/// burn and its ghost, rounded up to 50; green from the target on, with no end to it.
pub fn burn_gauge_of(burnt: f64, target: f64, ghost: Option<f64>, width: f64) -> BurnGauge {
    let larger = target.max(burnt).max(ghost.unwrap_or(0.0));
    let stretched =
        (ROOM_PAST_THE_LARGER * larger / ROUNDED_UP_TO_KCAL).ceil() * ROUNDED_UP_TO_KCAL;
    let scale = (0.0, stretched.max(BURN_GAUGE_KCAL));
    BurnGauge {
        band_from: placed(target, scale, width),
        target_at: placed(target, scale, width),
        burnt_at: placed(burnt, scale, width),
        ghost_at: ghost.map(|kcal| placed(kcal, scale, width)),
        up_to_kcal: scale.1,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimGauge {
    pub band_from: f64,
    pub band_to: f64,
    pub aim_at: f64,
    pub amount_at: f64,
    pub from: f64,
    pub up_to: f64,
}

const AIM_GAUGE_SPAN: f64 = 0.10;

/// A line a tenth either side of the aim, stretched to the amount: green where a value to reach stands at its
/// aim, and from a floor's tick to the end.
pub fn aim_gauge_of(amount: f64, aim: Aim, width: f64) -> AimGauge {
    let (Aim::Reach(value) | Aim::Floor(value)) = aim;
    let scale = (
        ((1.0 - AIM_GAUGE_SPAN) * value).min(amount),
        ((1.0 + AIM_GAUGE_SPAN) * value).max(amount),
    );
    let (band_from, band_to) = match aim {
        Aim::Reach(_) => (
            placed((1.0 - AT_THE_AIM_SHARE) * value, scale, width),
            placed((1.0 + AT_THE_AIM_SHARE) * value, scale, width),
        ),
        Aim::Floor(_) => (placed(value, scale, width), width),
    };
    AimGauge {
        band_from,
        band_to,
        aim_at: placed(value, scale, width),
        amount_at: placed(amount, scale, width),
        from: scale.0,
        up_to: scale.1,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyGauge {
    pub minimum_at: f64,
    pub floor_at: f64,
    pub maintenance_at: f64,
    pub cap_at: f64,
    pub eaten_at: f64,
    pub ghost: Option<GhostEnergy>,
}

/// What the day would eat and where its band would run, once what is about to be eaten is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GhostEnergy {
    pub eaten_at: f64,
    pub floor_at: f64,
    pub maintenance_at: f64,
    pub cap_at: f64,
}

pub fn energy_gauge_of(
    eaten: f64,
    (minimum, floor, maintenance, cap): (f64, f64, f64, f64),
    ghost: Option<(f64, (f64, f64, f64))>,
    width: f64,
) -> EnergyGauge {
    let top = ghost.map_or(cap, |(_, (_, _, ghost_cap))| cap.max(ghost_cap));
    let spare = 0.1 * (top - minimum).max(0.1 * top);
    let scale = (minimum - spare, top + spare);
    let at = |kcal| placed(kcal, scale, width);
    EnergyGauge {
        minimum_at: at(minimum),
        floor_at: at(floor),
        maintenance_at: at(maintenance),
        cap_at: at(cap),
        eaten_at: at(eaten).clamp(0.0, width),
        ghost: ghost.map(|(eaten, (floor, maintenance, cap))| GhostEnergy {
            eaten_at: at(eaten).clamp(0.0, width),
            floor_at: at(floor),
            maintenance_at: at(maintenance),
            cap_at: at(cap),
        }),
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
    fn a_gauge_runs_from_nothing_to_a_quarter_past_the_cap_and_places_the_band_the_optimal_and_the_amount()
     {
        assert_eq!(
            gauge_of(1000.0, (1800.0, 2000.0, 2200.0), None, 275.0),
            Gauge {
                band_from: 180.0,
                band_to: 220.0,
                optimal_at: 200.0,
                amount_at: 100.0,
                ghost_at: None,
            }
        );
    }

    #[test]
    fn a_gauge_stretches_to_hold_a_ghost_past_its_end() {
        let gauge = gauge_of(1500.0, (1800.0, 2000.0, 2200.0), Some(3000.0), 300.0);

        assert_eq!((gauge.amount_at, gauge.ghost_at), (150.0, Some(300.0)));
    }

    #[test]
    fn a_burn_gauge_runs_from_nothing_to_200_kcal_and_turns_green_at_its_target() {
        assert_eq!(
            burn_gauge_of(60.0, 80.0, None, 400.0),
            BurnGauge {
                band_from: 160.0,
                target_at: 160.0,
                burnt_at: 120.0,
                ghost_at: None,
                up_to_kcal: 200.0,
            }
        );
    }

    #[test]
    fn a_ghost_sits_on_the_burn_gauge_stretching_it_as_a_burn_would() {
        assert_eq!(
            burn_gauge_of(60.0, 80.0, Some(300.0), 400.0),
            BurnGauge {
                band_from: 80.0,
                target_at: 80.0,
                burnt_at: 60.0,
                ghost_at: Some(300.0),
                up_to_kcal: 400.0,
            }
        );
    }

    #[test]
    fn a_target_or_a_burn_past_200_kcal_stretches_the_gauge_a_fifth_beyond_rounded_up_to_50() {
        assert_eq!(
            (
                burn_gauge_of(300.0, 150.0, None, 400.0).up_to_kcal,
                burn_gauge_of(20.0, 180.0, None, 400.0).up_to_kcal
            ),
            (400.0, 250.0)
        );
    }

    #[test]
    fn an_aim_gauge_spans_a_tenth_either_side_of_the_aim_green_a_hundredth_either_side() {
        let gauge = aim_gauge_of(72.0, Aim::Reach(70.0), 280.0);

        for (placed, expected) in [
            (gauge.band_from, 126.0),
            (gauge.band_to, 154.0),
            (gauge.aim_at, 140.0),
            (gauge.amount_at, 180.0),
            (gauge.from, 63.0),
            (gauge.up_to, 77.0),
        ] {
            assert!((placed - expected).abs() < 1e-9, "{gauge:?}");
        }
    }

    #[test]
    fn a_floors_gauge_runs_green_from_its_tick_to_the_end_of_its_line() {
        assert_eq!(
            aim_gauge_of(72.0, Aim::Floor(70.0), 280.0),
            AimGauge {
                band_from: 140.0,
                band_to: 280.0,
                aim_at: 140.0,
                amount_at: 180.0,
                from: 63.0,
                up_to: 77.0,
            }
        );
    }

    #[test]
    fn an_amount_further_than_a_tenth_from_its_aim_stretches_the_aim_gauge_to_it() {
        let above = aim_gauge_of(84.0, Aim::Reach(70.0), 280.0);
        let below = aim_gauge_of(56.0, Aim::Reach(70.0), 280.0);

        assert_eq!((above.amount_at, below.amount_at), (280.0, 0.0));
        assert!((above.aim_at - 280.0 * 7.0 / 21.0).abs() < 1e-9);
    }

    #[test]
    fn an_amount_beyond_the_scale_stretches_it_to_stay_in_sight() {
        let gauge = gauge_of(4000.0, (1800.0, 2000.0, 2200.0), None, 400.0);

        assert_eq!((gauge.amount_at, gauge.optimal_at), (400.0, 200.0));
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
        let gauge = energy_gauge_of(2200.0, (1600.0, 2000.0, 2400.0, 2400.0), None, 960.0);

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
        let band = (1600.0, 2000.0, 2400.0, 2400.0);

        assert_eq!(energy_gauge_of(100.0, band, None, 960.0).eaten_at, 0.0);
        assert_eq!(energy_gauge_of(5000.0, band, None, 960.0).eaten_at, 960.0);
    }

    #[test]
    fn a_band_closed_on_its_minimum_sits_in_the_middle_of_a_gauge_a_tenth_of_it_wide() {
        let closed = energy_gauge_of(1800.0, (1800.0, 1800.0, 1800.0, 1800.0), None, 960.0);

        assert_eq!(
            (closed.minimum_at, closed.cap_at, closed.eaten_at),
            (480.0, 480.0, 480.0)
        );
    }

    #[test]
    fn an_energy_gauge_stretches_to_hold_the_band_previewed_and_places_it_on_the_same_line() {
        let previewed = energy_gauge_of(
            2200.0,
            (1600.0, 2000.0, 2400.0, 2400.0),
            Some((2300.0, (2200.0, 2600.0, 2600.0))),
            1200.0,
        );

        assert_eq!(
            (previewed.cap_at, previewed.ghost),
            (
                900.0,
                Some(GhostEnergy {
                    eaten_at: 800.0,
                    floor_at: 700.0,
                    maintenance_at: 1100.0,
                    cap_at: 1100.0,
                })
            )
        );
    }

    #[test]
    fn an_energy_gauge_ticks_maintenance_where_it_falls_in_its_band() {
        let gaining = energy_gauge_of(2200.0, (1600.0, 2000.0, 2100.0, 2400.0), None, 960.0);

        assert!((gaining.maintenance_at - 580.0).abs() < 1e-9, "{gaining:?}");
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

    #[test]
    fn a_line_of_a_single_day_stands_in_the_middle_of_the_width() {
        let points = [(day("2026-10-10"), 70.0)];

        assert_eq!(
            line_of(
                &points,
                (day("2026-10-10"), day("2026-10-10")),
                (60.0, 80.0),
                360.0,
                180.0
            ),
            "180.0,90.0"
        );
    }

    #[test]
    fn an_x_axis_dates_its_oldest_day_its_newest_and_three_between() {
        assert_eq!(
            days_along((day("2026-09-10"), day("2026-10-10")), 360.0),
            vec![
                (0.0, "09-10".to_string()),
                (96.0, "09-18".to_string()),
                (180.0, "09-25".to_string()),
                (276.0, "10-03".to_string()),
                (360.0, "10-10".to_string()),
            ]
        );
    }

    #[test]
    fn a_short_span_dates_each_day_once_and_a_long_one_its_months() {
        assert_eq!(
            days_along((day("2026-10-09"), day("2026-10-10")), 360.0),
            vec![(0.0, "10-09".to_string()), (360.0, "10-10".to_string())]
        );
        assert_eq!(
            days_along((day("2026-10-10"), day("2026-10-10")), 360.0),
            vec![(180.0, "10-10".to_string())]
        );
        assert_eq!(
            days_along((day("2025-06-01"), day("2026-10-10")), 360.0)
                .into_iter()
                .map(|(_, label)| label)
                .collect::<Vec<_>>(),
            ["2025-06", "2025-10", "2026-02", "2026-06", "2026-10"]
        );
    }

    #[test]
    fn a_week_dates_each_of_its_seven_days() {
        assert_eq!(
            days_along((day("2026-10-04"), day("2026-10-10")), 360.0)
                .into_iter()
                .map(|(_, label)| label)
                .collect::<Vec<_>>(),
            [
                "10-04", "10-05", "10-06", "10-07", "10-08", "10-09", "10-10"
            ]
        );
    }
}
