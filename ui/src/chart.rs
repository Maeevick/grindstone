use chrono::NaiveDate;
use grind_core::for_reviewing_progress::GatherCard;

#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    pub masses: Vec<(f64, f64)>,
    pub trend: String,
    pub target: Option<f64>,
    pub highest_kg: f64,
    pub lowest_kg: f64,
}

pub fn chart_of(card: &GatherCard, width: f64, height: f64) -> Option<Chart> {
    let first_day = card.masses.first()?.0;
    let last_day = card.masses.last()?.0;
    let days_drawn = (last_day - first_day).num_days() as f64;
    let values = card
        .masses
        .iter()
        .chain(&card.trend)
        .map(|(_, kg)| *kg)
        .chain(card.target_kg);
    let highest_kg = values.clone().fold(f64::MIN, f64::max) + 0.5;
    let lowest_kg = values.fold(f64::MAX, f64::min) - 0.5;
    let height_of = |kg: f64| (highest_kg - kg) / (highest_kg - lowest_kg) * height;
    let along_of = |day: NaiveDate| {
        if days_drawn == 0.0 {
            width / 2.0
        } else {
            (day - first_day).num_days() as f64 / days_drawn * width
        }
    };
    let point_of = |(day, kg): &(NaiveDate, f64)| (along_of(*day), height_of(*kg));
    Some(Chart {
        masses: card.masses.iter().map(point_of).collect(),
        trend: card
            .trend
            .iter()
            .map(point_of)
            .map(|(x, y)| format!("{x:.1},{y:.1}"))
            .collect::<Vec<_>>()
            .join(" "),
        target: card.target_kg.map(height_of),
        highest_kg,
        lowest_kg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn a_card(masses: &[(&str, f64)]) -> GatherCard {
        GatherCard {
            masses: masses.iter().map(|(date, kg)| (day(date), *kg)).collect(),
            ..GatherCard::default()
        }
    }

    #[test]
    fn a_card_without_masses_draws_no_chart() {
        assert_eq!(chart_of(&GatherCard::default(), 360.0, 180.0), None);
    }

    #[test]
    fn the_earliest_mass_sits_at_the_left_edge_and_the_latest_at_the_right() {
        let chart = chart_of(
            &a_card(&[("2026-10-01", 80.0), ("2026-10-11", 79.0)]),
            360.0,
            180.0,
        );

        let xs: Option<Vec<f64>> =
            chart.map(|chart| chart.masses.iter().map(|(x, _)| *x).collect());
        assert_eq!(xs, Some(vec![0.0, 360.0]));
    }

    #[test]
    fn the_masses_fill_the_height_with_half_a_kilo_to_spare_above_and_below() {
        let chart = chart_of(
            &a_card(&[("2026-10-01", 80.0), ("2026-10-11", 79.0)]),
            360.0,
            180.0,
        )
        .expect("a chart");

        let ys: Vec<f64> = chart.masses.iter().map(|(_, y)| *y).collect();
        assert_eq!(ys, vec![45.0, 135.0]);
        assert_eq!((chart.highest_kg, chart.lowest_kg), (80.5, 78.5));
    }

    #[test]
    fn a_target_beyond_the_masses_widens_the_frame_to_show_its_line() {
        let card = GatherCard {
            target_kg: Some(70.0),
            ..a_card(&[("2026-10-01", 80.0), ("2026-10-11", 79.0)])
        };

        let chart = chart_of(&card, 360.0, 180.0).expect("a chart");

        assert_eq!(chart.lowest_kg, 69.5);
        assert!(
            chart
                .target
                .is_some_and(|y| (y - 10.5 / 11.0 * 180.0).abs() < 1e-9)
        );
    }

    #[test]
    fn the_trend_runs_as_one_line_through_its_points() {
        let card = GatherCard {
            trend: vec![(day("2026-10-01"), 79.8), (day("2026-10-11"), 79.2)],
            ..a_card(&[("2026-10-01", 80.0), ("2026-10-11", 79.0)])
        };

        let chart = chart_of(&card, 360.0, 180.0).expect("a chart");

        assert_eq!(chart.trend, "0.0,63.0 360.0,117.0");
    }

    #[test]
    fn a_single_day_sits_in_the_middle() {
        let chart = chart_of(&a_card(&[("2026-10-10", 80.0)]), 360.0, 180.0).expect("a chart");

        assert_eq!(chart.masses, vec![(180.0, 90.0)]);
    }
}
