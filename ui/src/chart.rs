use grind_core::for_reviewing_progress::GatherCard;

#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    pub masses: Vec<(f64, f64)>,
    pub trend: String,
    pub target: Option<f64>,
    pub highest_kg: f64,
    pub lowest_kg: f64,
}

pub fn chart_of(card: &GatherCard, width: f64, _height: f64) -> Option<Chart> {
    let first_day = card.masses.first()?.0;
    let last_day = card.masses.last()?.0;
    let days_drawn = (last_day - first_day).num_days() as f64;
    Some(Chart {
        masses: card
            .masses
            .iter()
            .map(|(day, _)| {
                (
                    (*day - first_day).num_days() as f64 / days_drawn * width,
                    0.0,
                )
            })
            .collect(),
        trend: String::new(),
        target: None,
        highest_kg: 0.0,
        lowest_kg: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

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
}
